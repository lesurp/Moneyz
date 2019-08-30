use crate::data::*;
use gtk::{GtkListStoreExtManual, StaticType, TreeModelExt};

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
    IsDefault = 3,
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
        bool::static_type(),
    ]);
    for budget_category in &budget_categories.0 {
        let budget_category_amount = monthly_budget
            .budgets
            .get(&budget_category.id())
            .unwrap_or(&BudgetAmount(0));
        list.insert_with_values(
            None,
            &[Id.into(), Name.into(), Amount.into(), IsDefault.into()],
            &[
                &budget_category.id().0,
                &budget_category.name(),
                &budget_category_amount.0,
                &false,
            ],
        );
    }

    // get the maximum id and increments it by one, or initialize first id at 0
    let next_category_id = budget_categories
        .0
        .iter()
        .last()
        .map(|max_id_category| max_id_category.id().0 + 1)
        .unwrap_or(0);

    // manually add an empty row so users can add categories
    list.insert_with_values(
        None,
        &[Id.into(), Name.into(), Amount.into(), IsDefault.into()],
        &[&next_category_id, &"New category", &0, &true],
    );

    list
}

impl Into<gtk::ListStore> for BudgetCategories {
    fn into(self) -> gtk::ListStore {
        use BudgetCategoryComboBoxIds::*;
        let list = gtk::ListStore::new(&[u32::static_type(), String::static_type()]);
        let mut i = 0;
        for budget_category in self.0 {
            list.insert_with_values(
                Some(i),
                &[Id.into(), Name.into()],
                &[&budget_category.id().0, &budget_category.name()],
            );
            i += 1;
        }
        list
    }
}

impl Into<gtk::ListStore> for &BudgetCategories {
    fn into(self) -> gtk::ListStore {
        use BudgetCategoryComboBoxIds::*;
        let list = gtk::ListStore::new(&[u32::static_type(), String::static_type()]);
        let mut i = 0;
        for budget_category in &self.0 {
            list.insert_with_values(
                Some(i),
                &[Id.into(), Name.into()],
                &[&budget_category.id().0, &budget_category.name()],
            );
            i += 1;
        }
        list
    }
}

pub fn get_spendings_model(
    monthly_budget: &MonthlyBudget,
    budget_categories: &BudgetCategories,
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
                        ("#ffffff", spending.budget_category.name())
                    } else {
                        // The id exists, but the category has been renamed - display the new one with a green background
                        ("#ddeedd", budget_category.name())
                    }
                }
                // Id doesn't exist - the catewgory has been deleted
                // We still show it, with a redbackground
                None => ("#eedddd", spending.budget_category.name()),
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
        &[&"New spending", &0, &"", &0, &1, &"#ffffff", &true],
    );
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

    for i in 1..max_day + 1 {
        list.insert_with_values(None, &[Day.into()], &[&i.to_string()]);
    }
    list
}
