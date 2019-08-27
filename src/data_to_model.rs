use crate::data::*;
use gtk::{GtkListStoreExtManual, StaticType};

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

impl Into<(gtk::ListStore, gtk::ListStore)> for MonthlyBudget {
    fn into(self) -> (gtk::ListStore, gtk::ListStore) {
        let budgets_list = gtk::ListStore::new(&[String::static_type(), String::static_type()]);
        for (budget_category, budget_amount) in &self.budgets {
            budgets_list.insert_with_values(
                None,
                &[0, 1],
                &[&budget_category.0, &budget_amount.0.to_string()],
            );
        }
        budgets_list.insert_with_values(None, &[0, 1], &[&"asd", &"123"]);

        let spendings_list = gtk::ListStore::new(&[
            String::static_type(),
            String::static_type(),
            i32::static_type(),
            i32::static_type(),
        ]);
        for spending in self.spendings.0 {
            spendings_list.insert_with_values(
                None,
                &[0, 1, 2, 3],
                &[
                    &spending.name,
                    &spending.budget_category.0,
                    &spending.amount,
                    &spending.day.0,
                ],
            );
        }
        spendings_list.insert_with_values(None, &[0, 1, 2, 3], &[&"asdsadsad", &"35y", &123, &1]);
        (budgets_list, spendings_list)
    }
}
