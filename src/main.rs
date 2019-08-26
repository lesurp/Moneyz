use self::Msg::*;
use chrono::Datelike;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::*;
//use gtk::{ComboBoxTextExt, ComboBoxExt, GtkListStoreExtManual, Inhibit, OrientableExt, StaticType, WidgetExt};
use data::Month;
use relm::{connect, connect_stream, Widget};
use relm_derive::{widget, Msg};

mod data;
mod file_loader;

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> () {
        ()
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="config_box"]
                gtk::Box {
                    orientation: Horizontal,
                    #[name="month_combo_box"]
                    gtk::ComboBox {
                    },
                    #[name="year_combo_box"]
                    gtk::ComboBox {
                },
                    #[name="config_button"]
                    gtk::Button {
                        label: "Configuration"
                    }
                },
                gtk::Box {
                    orientation: Horizontal,
                    #[name="entries_scrolled_window"]
                    gtk::ScrolledWindow {
                        #[name="entries_tree_view"]
                        gtk::TreeView {

                        },
                    },
                    #[name="categories_scrolled_window"]
                    gtk::ScrolledWindow {
                        #[name="categories_tree_view"]
                        gtk::TreeView {

                        },
                    },
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn initialize_month_year_combo_boxex(mcb: &gtk::ComboBox, ycb: &gtk::ComboBox) {
    let local: chrono::DateTime<chrono::Local> = chrono::Local::now();
    let current_month = local.date().month() - 1; // chrono starts counting at 1
    let current_year = local.date().year();

    let cell = gtk::CellRendererText::new();
    let month_model = create_and_fill_month_model();
    mcb.set_model(Some(&month_model));
    mcb.pack_start(&cell, true);
    mcb.add_attribute(&cell, "text", 1);
    mcb.set_id_column(0);
    mcb.set_active_id(Some(&current_month.to_string()));

    let cell = gtk::CellRendererText::new();
    let year_model = create_and_fill_year_model();
    ycb.set_model(Some(&year_model));
    ycb.pack_start(&cell, true);
    ycb.add_attribute(&cell, "text", 1);
    ycb.set_id_column(0);
    ycb.set_active_id(Some(&current_year.to_string()));
}

fn create_and_fill_month_model() -> gtk::ListStore {
    // Single row model
    let model = gtk::ListStore::new(&[String::static_type(), String::static_type()]);

    for m_idx in 0 as u32..12 {
        let m: Month = num_traits::FromPrimitive::from_u32(m_idx).unwrap();
        let m_str = m.to_string();
        model.insert_with_values(
            Some(m_idx),
            &[0, 1],
            &[&m_idx.to_string(), &m_str.to_string()],
        );
    }

    model
}

fn create_and_fill_year_model() -> gtk::ListStore {
    // Single row model
    let model = gtk::ListStore::new(&[String::static_type(), String::static_type()]);

    for year in 2010 as u32..2099 {
        model.insert_with_values(Some(year), &[0, 1], &[&year.to_string(), &year.to_string()]);
    }

    model
}

fn create_and_fill_budget_category_model() -> gtk::ListStore {
    let model = gtk::ListStore::new(&[String::static_type(), String::static_type()]);
    model.insert_with_values(Some(0), &[0, 1], &[&0.to_string(), &"a".to_string()]);
    model.insert_with_values(Some(1), &[0, 1], &[&1.to_string(), &"z".to_string()]);
    model.insert_with_values(Some(2), &[0, 1], &[&2.to_string(), &"e".to_string()]);
    model.insert_with_values(Some(3), &[0, 1], &[&3.to_string(), &"r".to_string()]);
    model.insert_with_values(Some(4), &[0, 1], &[&4.to_string(), &"t".to_string()]);
    model.insert_with_values(Some(5), &[0, 1], &[&5.to_string(), &"y".to_string()]);
    model
}

fn main() {
    let (_component, widgets) = relm::init_test::<Win>(()).expect("test failed");
    let month_combo_box = &widgets.month_combo_box;
    let year_combo_box = &widgets.year_combo_box;
    initialize_month_year_combo_boxex(&month_combo_box, &year_combo_box);

    let entries_tree_view = &widgets.entries_tree_view;
    let entries_scrolled_window = &widgets.entries_scrolled_window;
    entries_scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    let col = gtk::TreeViewColumn::new();
    col.set_title("Name");
    let cell = gtk::CellRendererText::new();
    cell.set_property_editable(true);
    col.pack_start(&cell, true);
    col.add_attribute(&cell, "text", 0);
    entries_tree_view.append_column(&col);

    let col = gtk::TreeViewColumn::new();
    col.set_title("Category");
    let cell = gtk::CellRendererCombo::new();
    let list_store = create_and_fill_budget_category_model();
    let tree_model = list_store.upcast::<gtk::TreeModel>();
    cell.set_property_model(Some(&tree_model));
    cell.set_property_editable(true);
    cell.set_property_has_entry(false);
    cell.set_property_text_column(1);
    col.pack_start(&cell, true);
    col.add_attribute(&cell, "text", 1);
    entries_tree_view.append_column(&col);

    let col = gtk::TreeViewColumn::new();
    col.set_title("Amount");
    let cell = gtk::CellRendererText::new();
    cell.set_property_editable(true);
    col.pack_start(&cell, true);
    col.add_attribute(&cell, "text", 2);
    entries_tree_view.append_column(&col);

    let col = gtk::TreeViewColumn::new();
    col.set_title("Budget surplus");
    let cell = gtk::CellRendererText::new();
    cell.set_property_editable(false);
    col.pack_start(&cell, true);

    let model = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    model.insert_with_values(Some(0), &[0, 1, 2], &[&"asd", &"a", &"asdasda"]);
    model.insert_with_values(Some(0), &[0, 1, 2], &[&"qwe", &"e", &"rgon"]);
    entries_tree_view.set_model(Some(&model));
    gtk::main();
}

/*
#[cfg(test)]
mod tests {
    use gtk::{Cast, Label, LabelExt, NotebookExt};
    use gtk_test::assert_text;

    use relm;

    use super::Win;

    #[test]
    fn root_widget() {
        let (_component, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let month_combo_box = &widgets.month_combo_box;
        let year_combo_box = &widgets.year_combo_box;
    }
}
*/
