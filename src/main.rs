mod data;
mod data_to_model;
mod file_loader;

use chrono::Datelike;
use data::Month;
use file_loader::FileLoader;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::*;
use relm::{connect, connect_stream, Widget};
use relm_derive::{widget, Msg};

const DATA_DIR: &str = "data";

pub struct MoneyzModel {
    file_loader: FileLoader,

    selected_month: data::Month,
    selected_year: data::Year,

    budget_categories: data::BudgetCategories,
    monthly_budget: data::MonthlyBudget,
}

#[derive(Msg)]
pub enum MoneyzMsg {
    Save,
    Quit,
}

#[widget]
impl Widget for Win {
    fn model(_: &relm::Relm<Self>, file_loader: FileLoader) -> MoneyzModel {
        let local: chrono::DateTime<chrono::Local> = chrono::Local::now();
        let current_month = local.date().month() - 1; // chrono starts counting at 1
        let current_year = local.date().year();
        let selected_month: data::Month =
            num_traits::FromPrimitive::from_u32(current_month).unwrap();
        let selected_year = data::Year(current_year);

        let budget_categories = file_loader.load_budget_categories().unwrap();
        let monthly_budget = file_loader
            .load_monthly_budget(selected_month, selected_year)
            .unwrap();

        MoneyzModel {
            file_loader,
            selected_month,
            selected_year,
            budget_categories,
            monthly_budget,
        }
    }

    fn initialize_budget_categories_headers(&self) {
        let col = gtk::TreeViewColumn::new();
        col.set_title("Budget category name");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 0);
        self.budget_categories_tree_view.append_column(&col);

        let col = gtk::TreeViewColumn::new();
        col.set_title("Budget amount");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 0);
        self.budget_categories_tree_view.append_column(&col);
    }

    fn initialize_spendings_tree_view_headers(&self) {
        let col = gtk::TreeViewColumn::new();
        col.set_title("Name");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 0);
        self.spendings_tree_view.append_column(&col);

        let col = gtk::TreeViewColumn::new();
        col.set_title("Category");
        let cell = gtk::CellRendererCombo::new();
        let category_model: gtk::ListStore = self
            .model
            .file_loader
            .load_budget_categories()
            .unwrap()
            .into();
        let tree_model = category_model.upcast::<gtk::TreeModel>();
        cell.set_property_model(Some(&tree_model));
        cell.set_property_editable(true);
        cell.set_property_has_entry(false);
        cell.set_property_text_column(0);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 1);
        self.spendings_tree_view.append_column(&col);

        let col = gtk::TreeViewColumn::new();
        col.set_title("Amount");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 2);
        self.spendings_tree_view.append_column(&col);

        let col = gtk::TreeViewColumn::new();
        col.set_title("Day");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 3);
        self.spendings_tree_view.append_column(&col);
    }

    fn update(&mut self, event: MoneyzMsg) {
        match event {
            MoneyzMsg::Quit => gtk::main_quit(),
            MoneyzMsg::Save => {
                self.model
                    .file_loader
                    .save_budget_categories(&self.model.budget_categories)
                    .unwrap();
                self.model
                    .file_loader
                    .save_monthly_budget(
                        self.model.selected_month,
                        self.model.selected_year,
                        &self.model.monthly_budget,
                    )
                    .unwrap();
            }
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
                    },
                    #[name="save_button"]
                    gtk::Button {
                        label: "Save"
                    }

                },
                gtk::Box {
                    orientation: Horizontal,
                    #[name="spendings_tree_view"]
                    gtk::TreeView {},
                    gtk::Separator { orientation: Vertical },
                    #[name="budget_categories_tree_view"]
                    gtk::TreeView {},
                },
            },
            delete_event(_, _) => (MoneyzMsg::Quit, Inhibit(false)),
        }
    }

    fn init_view(&mut self) {
        let month_combo_box = &self.month_combo_box;
        let year_combo_box = &self.year_combo_box;
        initialize_month_year_combo_boxes(
            self.model.selected_month,
            self.model.selected_year,
            &month_combo_box,
            &year_combo_box,
        );

        self.initialize_spendings_tree_view_headers();
        let (_, spendings_model): (gtk::ListStore, gtk::ListStore) = self
            .model
            .file_loader
            .load_monthly_budget(self.model.selected_month, self.model.selected_year)
            .unwrap()
            .into();
        self.spendings_tree_view.set_model(Some(&spendings_model));

        self.initialize_budget_categories_headers();
        let model: gtk::ListStore = self
            .model
            .file_loader
            .load_budget_categories()
            .unwrap()
            .into();
        self.budget_categories_tree_view.set_model(Some(&model));
    }
}

fn initialize_month_year_combo_boxes(
    month: data::Month,
    year: data::Year,
    mcb: &gtk::ComboBox,
    ycb: &gtk::ComboBox,
) {
    let cell = gtk::CellRendererText::new();
    let month_model = create_and_fill_month_model();
    mcb.set_model(Some(&month_model));
    mcb.pack_start(&cell, true);
    mcb.add_attribute(&cell, "text", 1);
    mcb.set_id_column(0);
    mcb.set_active_id(Some(&month.id_string()));

    let cell = gtk::CellRendererText::new();
    let year_model = create_and_fill_year_model();
    ycb.set_model(Some(&year_model));
    ycb.pack_start(&cell, true);
    ycb.add_attribute(&cell, "text", 0);
    ycb.set_id_column(0);
    ycb.set_active_id(Some(&year.to_string()));
}

fn create_and_fill_month_model() -> gtk::ListStore {
    let model = gtk::ListStore::new(&[String::static_type(), String::static_type()]);
    for m_idx in 0 as u32..12 {
        let m: Month = num_traits::FromPrimitive::from_u32(m_idx).unwrap();
        model.insert_with_values(Some(m_idx), &[0, 1], &[&m.id_string(), &m.display_string()]);
    }
    model
}

fn create_and_fill_year_model() -> gtk::ListStore {
    let model = gtk::ListStore::new(&[String::static_type()]);
    for year in 2010 as u32..2099 {
        model.insert_with_values(None, &[0], &[&year.to_string()]);
    }
    model
}

fn main() {
    let file_loader = FileLoader::new(DATA_DIR);
    Win::run(file_loader).expect("Win::run failed");
}
