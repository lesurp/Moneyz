use crate::data::*;
use gtk::{GtkListStoreExtManual, StaticType};

pub fn get_model_from_budget_categories_and_monthly_budget(
    budget_categories: &BudgetCategories,
    monthly_budget: &MonthlyBudget,
) -> gtk::ListStore {
    let list = gtk::ListStore::new(&[String::static_type(), i32::static_type()]);
    for budget_category in &budget_categories.0 {
        let budget_category_amount = monthly_budget
            .budgets
            .get(&budget_category)
            .unwrap_or(&BudgetAmount(0));
        list.insert_with_values(
            None,
            &[0, 1],
            &[&budget_category.0, &budget_category_amount.0],
        );
    }
    list
}

impl Into<gtk::ListStore> for BudgetCategories {
    fn into(self) -> gtk::ListStore {
        let list = gtk::ListStore::new(&[String::static_type()]);
        let mut i = 0;
        for budget_category in self.0 {
            list.insert_with_values(Some(i), &[0], &[&budget_category.0]);
            i += 1;
        }
        list.insert_with_values(None, &[0], &[&"ASDA"]);
        list
    }
}

pub fn get_spendings_model(
    monthly_budget: &MonthlyBudget,
    budget_categories: &BudgetCategories,
) -> gtk::ListStore {
    let spendings_list = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        i32::static_type(),
        i32::static_type(),
        String::static_type(),
    ]);

    for spending in &monthly_budget.spendings.0 {
        let category_color = if budget_categories.0.contains(&spending.budget_category) {
            "ffffff"
        } else {
            "aa0000"
        };
        spendings_list.insert_with_values(
            None,
            &[0, 1, 2, 3],
            &[
                &spending.name,
                &spending.budget_category.0,
                &spending.amount,
                &spending.day.0,
                &category_color,
            ],
        );
    }
    spendings_list.insert_with_values(
        None,
        &[0, 1, 2, 3, 4],
        &[&"asdsadsad", &"35y", &123, &1, &"#eedddd"],
    );
    spendings_list
}
