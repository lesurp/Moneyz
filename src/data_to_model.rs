use crate::data::*;
use gtk::{GtkListStoreExtManual, StaticType, TreeModelExt};
use log::debug;

pub const BACKGROUND_COLOR_NORMAL: &str = "#ffffff";
pub const BACKGROUND_COLOR_IS_DEFAULT: &str = "#ddddee";
pub const BACKGROUND_COLOR_WRONG_BUDGET_CATEGORY: &str = "#eedddd";
pub const BACKGROUND_COLOR_RENAMED_BUDGET_CATEGORY: &str = "#ddeedd";

pub enum SpendingDayComboBoxIds {
    Day = 0,
}

impl Into<i32> for SpendingDayComboBoxIds {
    fn into(self) -> i32 {
        self as i32
    }
}

impl Into<u32> for SpendingDayComboBoxIds {
    fn into(self) -> u32 {
        self as u32
    }
}

pub enum BudgetCategoryComboBoxIds {
    Id = 0,
    Name = 1,
}

impl Into<i32> for BudgetCategoryComboBoxIds {
    fn into(self) -> i32 {
        self as i32
    }
}

impl Into<u32> for BudgetCategoryComboBoxIds {
    fn into(self) -> u32 {
        self as u32
    }
}

pub enum BudgetCategoriesListStoreIds {
    Id = 0,
    Name = 1,
    Amount = 2,
    Surplus = 3,
    IsDefault = 4,
    BackgroundColor = 5,
}

impl Into<i32> for BudgetCategoriesListStoreIds {
    fn into(self) -> i32 {
        self as i32
    }
}

impl Into<u32> for BudgetCategoriesListStoreIds {
    fn into(self) -> u32 {
        self as u32
    }
}

pub enum SpendingsGtkModelIds {
    Name = 0,
    CategoryId = 1,
    CategoryName = 2,
    Amount = 3,
    Day = 4,
    BackgroundColor = 5,
    IsDefault = 6,
}

impl Into<i32> for SpendingsGtkModelIds {
    fn into(self) -> i32 {
        self as i32
    }
}

impl Into<u32> for SpendingsGtkModelIds {
    fn into(self) -> u32 {
        self as u32
    }
}

pub fn get_model_from_budget_categories_and_monthly_budget(
    budget_categories: &BudgetCategories,
    monthly_budget: &MonthlyBudget,
) -> gtk::ListStore {
    use BudgetCategoriesListStoreIds::*;

    let list = gtk::ListStore::new(&[
        u32::static_type(),
        String::static_type(),
        i32::static_type(),
        i32::static_type(),
        bool::static_type(),
        String::static_type(),
    ]);

    // collect for each budget_category how much was spent and all
    let mut spendings_per_budget = std::collections::HashMap::new();
    for spending in &monthly_budget.spendings.0 {
        *spendings_per_budget
            .entry(&spending.budget_category)
            .or_insert(0) += spending.amount;
    }

    for budget_category in &budget_categories.0 {
        let budget_category_amount = monthly_budget
            .budgets
            .get(&budget_category.id())
            .unwrap_or(&BudgetAmount(0));
        list.insert_with_values(
            None,
            &[
                Id.into(),
                Name.into(),
                Amount.into(),
                Surplus.into(),
                IsDefault.into(),
                BackgroundColor.into(),
            ],
            &[
                &budget_category.id().0,
                &budget_category.name(),
                &budget_category_amount.0,
                &(budget_category_amount.0
                    - spendings_per_budget.get(&budget_category).unwrap_or(&0)),
                &false,
                &BACKGROUND_COLOR_NORMAL,
            ],
        );
    }

    // get the maximum id and increments it by one, or initialize first id at 0
    let max_category_id = budget_categories
        .0
        .iter()
        .last()
        .map(|max_category_id| max_category_id.id().0)
        .unwrap_or(0);

    add_default_budget_category(&list, max_category_id);
    list
}

impl Into<gtk::ListStore> for BudgetCategories {
    fn into(self) -> gtk::ListStore {
        (&self).into()
    }
}

impl Into<gtk::ListStore> for &BudgetCategories {
    fn into(self) -> gtk::ListStore {
        use BudgetCategoryComboBoxIds::*;
        let list = gtk::ListStore::new(&[u32::static_type(), String::static_type()]);
        for budget_category in &self.0 {
            list.insert_with_values(
                None,
                &[Id.into(), Name.into()],
                &[&budget_category.id().0, &budget_category.name()],
            );
        }
        list
    }
}

pub fn get_spendings_model(
    monthly_budget: &MonthlyBudget,
    budget_categories: &BudgetCategories,
    today: Day,
) -> gtk::ListStore {
    use SpendingsGtkModelIds::*;
    let spendings_list = gtk::ListStore::new(&[
        // name
        String::static_type(),
        // category id
        u32::static_type(),
        // category display string
        String::static_type(),
        // amount
        i32::static_type(),
        // day
        i32::static_type(),
        // background color (when category doesn't exist anymore)
        String::static_type(),
        // is_default
        bool::static_type(),
    ]);

    for spending in &monthly_budget.spendings.0 {
        let (category_color, category_name) =
            match budget_categories.0.get(&spending.budget_category) {
                Some(budget_category) => {
                    if budget_category.name() == spending.budget_category.name() {
                        // Id and display string are the same - nothing special
                        (BACKGROUND_COLOR_NORMAL, spending.budget_category.name())
                    } else {
                        debug!(
                            "Id exists but names re different. Old name: {}",
                            spending.budget_category.name()
                        );
                        debug!("New name: {}", budget_category.name());
                        // The id exists, but the category has been renamed - display the new one with a green background
                        (
                            BACKGROUND_COLOR_RENAMED_BUDGET_CATEGORY,
                            budget_category.name(),
                        )
                    }
                }
                // Id doesn't exist - the category has been deleted
                // We still show it, with a redbackground
                None => (
                    BACKGROUND_COLOR_WRONG_BUDGET_CATEGORY,
                    spending.budget_category.name(),
                ),
            };

        spendings_list.insert_with_values(
            None,
            &[
                Name.into(),
                CategoryId.into(),
                CategoryName.into(),
                Amount.into(),
                Day.into(),
                BackgroundColor.into(),
                IsDefault.into(),
            ],
            &[
                &spending.name,
                &spending.budget_category.id().0,
                &category_name,
                &spending.amount,
                &spending.day.0,
                &category_color,
                &false,
            ],
        );
    }
    order_spendings_by_day(&spendings_list);
    add_default_spending(&spendings_list, today);
    spendings_list
}

pub fn list_store_to_budget_categories(model: gtk::TreeModel) -> BudgetCategories {
    use BudgetCategoriesListStoreIds::*;
    let mut budget_categories = BudgetCategories::default();
    model.foreach(|m, _, i| {
        if m.get_value(i, IsDefault.into()).get().unwrap() {
            return false;
        }

        let id = m.get_value(i, Id.into()).get().unwrap();
        let name = m.get_value(i, Name.into()).get().unwrap();
        budget_categories
            .0
            .insert(BudgetCategory(BudgetCategoryId(id), name));
        // false = continue; true = break
        false
    });

    budget_categories
}

pub fn list_store_to_monthly_budget(
    spendings_model: gtk::TreeModel,
    budget_categories_model: gtk::TreeModel,
) -> MonthlyBudget {
    let mut spendings = Spendings(Vec::new());
    {
        use SpendingsGtkModelIds::*;
        spendings_model.foreach(|m, _, i| {
            if m.get_value(i, IsDefault.into()).get().unwrap() {
                return false;
            }
            let name = m.get_value(i, Name.into()).get().unwrap();
            let category_id = m.get_value(i, CategoryId.into()).get().unwrap();
            let category_name = m.get_value(i, CategoryName.into()).get().unwrap();
            let amount = m.get_value(i, Amount.into()).get().unwrap();
            let day = crate::data::Day(m.get_value(i, Day.into()).get().unwrap());

            spendings.0.push(Spending {
                name,
                budget_category: BudgetCategory(BudgetCategoryId(category_id), category_name),
                amount,
                day,
            });

            // false = continue; true = break
            false
        });
    }

    let mut budgets = std::collections::HashMap::new();
    {
        use BudgetCategoriesListStoreIds::*;
        budget_categories_model.foreach(|m, _, i| {
            if m.get_value(i, IsDefault.into()).get().unwrap() {
                return false;
            }
            let id = m.get_value(i, Id.into()).get().unwrap();
            let amount = m.get_value(i, Amount.into()).get().unwrap();

            budgets.insert(BudgetCategoryId(id), BudgetAmount(amount));

            // false = continue; true = break
            false
        });
    }

    MonthlyBudget { budgets, spendings }
}

pub fn list_model_from_month_year(m: Month, y: Year) -> gtk::ListStore {
    use SpendingDayComboBoxIds::Day;
    let list = gtk::ListStore::new(&[String::static_type()]);

    let max_day = match m {
        Month::Jan
        | Month::Mar
        | Month::May
        | Month::Jul
        | Month::Aug
        | Month::Oct
        | Month::Dec => 31,
        Month::Apr | Month::Jun | Month::Sep | Month::Nov => 30,

        Month::Feb => {
            if (y.0 % 4 == 0 && y.0 % 100 != 0) || y.0 % 400 == 0 {
                29
            } else {
                28
            }
        }
    };

    for i in 1..=max_day {
        list.insert_with_values(None, &[Day.into()], &[&i.to_string()]);
    }
    list
}

pub fn add_default_spending(model: &gtk::ListStore, today: Day) {
    use SpendingsGtkModelIds::*;
    model.insert_with_values(
        None,
        &[
            Name.into(),
            CategoryId.into(),
            CategoryName.into(),
            Amount.into(),
            Day.into(),
            BackgroundColor.into(),
            IsDefault.into(),
        ],
        &[
            &"New spending",
            // TODO: see comment on the Spendings declaration on why we do that
            // note: using an option would be better...
            &u32::max_value(),
            &"",
            &0,
            &today.0,
            &BACKGROUND_COLOR_IS_DEFAULT,
            &true,
        ],
    );
}

pub fn add_default_budget_category(model: &gtk::ListStore, max_id: u32) {
    use BudgetCategoriesListStoreIds::*;
    model.insert_with_values(
        None,
        &[
            Id.into(),
            Name.into(),
            Amount.into(),
            Surplus.into(),
            IsDefault.into(),
            BackgroundColor.into(),
        ],
        &[
            &(max_id + 1),
            &"New category",
            &0,
            &0,
            &true,
            &BACKGROUND_COLOR_IS_DEFAULT,
        ],
    );
}

pub fn order_spendings_by_day(model: &gtk::ListStore) {
    let mut sorting_by_day_vec = Vec::new();
    let mut i = 0;
    model.foreach(|m, _, iter| {
        let day = m
            .get_value(iter, SpendingsGtkModelIds::Day.into())
            .get::<i32>()
            .unwrap();
        sorting_by_day_vec.push((i, day));
        i += 1;
        false
    });
    sorting_by_day_vec.sort_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap());
    let ordering_array = sorting_by_day_vec
        .iter()
        .map(|(i, _)| *i as u32)
        .collect::<Vec<_>>();
    model.reorder(&ordering_array);
}
