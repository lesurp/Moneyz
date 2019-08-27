use crate::data::*;
use gtk::{StaticType, GtkListStoreExtManual};

impl Into<gtk::ListStore> for BudgetCategories {
    fn into(self) -> gtk::ListStore {
        let list = gtk::ListStore::new(&[String::static_type()]);
        let mut i = 0;
        for budget_category in self.0 {
            list.insert_with_values(Some(i), &[0], &[&budget_category.0]);
            i += 1;
        }
        list
    }
}
