use crate::data::*;
use crate::translation_provider::TranslationProvider;
use gtk::{GtkListStoreExtManual, StaticType, TreeModelExt};
use log::debug;

pub const BACKGROUND_COLOR_NORMAL: &str = "#ffffff";
pub const BACKGROUND_COLOR_IS_DEFAULT: &str = "#ddddee";
pub const BACKGROUND_COLOR_WRONG_BUDGET_CATEGORY: &str = "#eedddd";
pub const BACKGROUND_COLOR_RENAMED_BUDGET_CATEGORY: &str = "#ddeedd";
pub const BACKGROUND_COLOR_DEBIT: &str = "#f00";
pub const BACKGROUND_COLOR_CREDIT: &str = "#0f0";
pub const BACKGROUND_COLOR_NULL: &str = "#ff0";

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
    Name = 0,
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
    Name = 0,
    Amount = 1,
    Balance = 2,
    NameBackgroundColor = 3,
    AmountBackgroundColor = 4,
    BalanceBackgroundColor = 5,
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
    CategoryName = 1,
    Amount = 2,
    Day = 3,

    NameBackgroundColor = 4,
    CategoryNameBackgroundColor = 5,
    AmountBackgroundColor = 6,
    DayBackgroundColor = 7,
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
    translation_provider: &TranslationProvider,
) -> gtk::ListStore {
    use BudgetCategoriesListStoreIds::*;

    let list = gtk::ListStore::new(&[
        // name
        String::static_type(),
        // amount
        String::static_type(),
        // balance
        String::static_type(),
        // name color
        String::static_type(),
        // amount color
        String::static_type(),
        // balance color
        String::static_type(),
    ]);

    // collect for each budget_category how much was spent and all
    let mut balance_per_budget = std::collections::HashMap::new();
    for spending in &monthly_budget.spendings.0 {
        *balance_per_budget
            .entry(spending.budget_category_id)
            .or_insert(0) += spending.amount.to_i32();
    }

    for (budget_category_id, budget_category) in &budget_categories.0 {
        let budget_category_amount = monthly_budget
            .budgets
            .get(&budget_category_id)
            .unwrap_or(&BudgetAmount(0));
        let amount = MoneyAmount::from_i32(budget_category_amount.0);
        let formatted_amount = translation_provider
            .format_money(amount.sign(), amount.whole(), amount.cents_padded())
            .expect("Could not format the input in the budget_category_amount fn!");

        let balance =
            budget_category_amount.0 + balance_per_budget.get(&budget_category_id).unwrap_or(&0);
        let balance_amount = MoneyAmount::from_i32(balance);
        let formatted_balance = translation_provider
            .format_money(
                balance_amount.sign(),
                balance_amount.whole(),
                balance_amount.cents_padded(),
            )
            .expect("Could not format the input in the budget_category_amount fn!");
        let balance_cell_color = amount_to_color(balance);

        list.insert_with_values(
            None,
            &[
                Name.into(),
                Amount.into(),
                Balance.into(),
                NameBackgroundColor.into(),
                AmountBackgroundColor.into(),
                BalanceBackgroundColor.into(),
            ],
            &[
                &budget_category.0,
                &formatted_amount,
                &formatted_balance,
                &BACKGROUND_COLOR_NORMAL,
                &BACKGROUND_COLOR_NORMAL,
                &balance_cell_color,
            ],
        );
    }
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
        let list = gtk::ListStore::new(&[String::static_type()]);
        for budget_category in self.0.values() {
            list.insert_with_values(None, &[Name.into()], &[&budget_category.0]);
        }
        list
    }
}

pub fn get_spendings_model(
    monthly_budget: &MonthlyBudget,
    budget_categories: &BudgetCategories,
    translation_provider: &TranslationProvider,
) -> gtk::ListStore {
    use SpendingsGtkModelIds::*;
    let spendings_list = gtk::ListStore::new(&[
        // name
        String::static_type(),
        // category display string
        String::static_type(),
        // amount
        String::static_type(),
        // day
        i32::static_type(),
        // background
        String::static_type(),
        // background
        String::static_type(),
        // background
        String::static_type(),
        // background
        String::static_type(),
    ]);

    for spending in &monthly_budget.spendings.0 {
        let (category_color, category_name) =
            match budget_categories.0.get(&spending.budget_category_id) {
                Some(name) => {
                    if name.0 == spending.budget_category_name.0 {
                        // Id and display string are the same - nothing special
                        (
                            BACKGROUND_COLOR_NORMAL,
                            spending.budget_category_name.clone(),
                        )
                    } else {
                        debug!(
                            "Id exists but names re different. Old name: {}",
                            spending.budget_category_name.0
                        );
                        debug!("New name: {}", name.0);
                        // The id exists, but the category has been renamed - display the new one with a green background
                        (BACKGROUND_COLOR_RENAMED_BUDGET_CATEGORY, name.clone())
                    }
                }
                // Id doesn't exist - the category has been deleted
                // We still show it, with a redbackground
                None => (
                    BACKGROUND_COLOR_WRONG_BUDGET_CATEGORY,
                    spending.budget_category_name.clone(),
                ),
            };

        let amount_cell_background_color = amount_to_color(spending.amount.to_i32());
        let formatted_amount = translation_provider
            .format_money(
                spending.amount.sign(),
                spending.amount.whole(),
                spending.amount.cents_padded(),
            )
            .expect("Could not format the input in the budget_category_amount fn!");

        spendings_list.insert_with_values(
            None,
            &[
                Name.into(),
                CategoryName.into(),
                Amount.into(),
                Day.into(),
                NameBackgroundColor.into(),
                CategoryNameBackgroundColor.into(),
                AmountBackgroundColor.into(),
                DayBackgroundColor.into(),
            ],
            &[
                &spending.name,
                &category_name.0,
                &formatted_amount,
                &spending.day.0,
                &BACKGROUND_COLOR_NORMAL,
                &category_color,
                &amount_cell_background_color,
                &BACKGROUND_COLOR_NORMAL,
            ],
        );
    }
    order_spendings_by_day(&spendings_list);
    spendings_list
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
            CategoryName.into(),
            Amount.into(),
            Day.into(),
            NameBackgroundColor.into(),
            CategoryNameBackgroundColor.into(),
            AmountBackgroundColor.into(),
            DayBackgroundColor.into(),
        ],
        &[
            &"New spending",
            // TODO: see comment on the Spendings declaration on why we do that
            // note: using an option would be better...
            &"",
            &"",
            &today.0,
            &BACKGROUND_COLOR_IS_DEFAULT,
            &BACKGROUND_COLOR_IS_DEFAULT,
            &BACKGROUND_COLOR_IS_DEFAULT,
            &BACKGROUND_COLOR_IS_DEFAULT,
        ],
    );
}

pub fn add_default_budget_category(model: &gtk::ListStore) {
    use BudgetCategoriesListStoreIds::*;
    model.insert_with_values(
        None,
        &[
            Name.into(),
            Amount.into(),
            Balance.into(),
            NameBackgroundColor.into(),
            AmountBackgroundColor.into(),
            BalanceBackgroundColor.into(),
        ],
        &[
            &"New category",
            &"",
            &"",
            &BACKGROUND_COLOR_IS_DEFAULT,
            &BACKGROUND_COLOR_IS_DEFAULT,
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

pub fn amount_to_color(amount: i32) -> &'static str {
    match amount.cmp(&0) {
        std::cmp::Ordering::Greater => BACKGROUND_COLOR_CREDIT,
        std::cmp::Ordering::Equal => BACKGROUND_COLOR_NULL,
        std::cmp::Ordering::Less => BACKGROUND_COLOR_DEBIT,
    }
}
