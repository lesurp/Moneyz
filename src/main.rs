mod data;
mod data_to_model;
mod file_loader;

use chrono::Datelike;
use data::Month;
use file_loader::FileLoader;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::*;
use log::debug;
use relm::{connect, connect_stream, Widget};
use relm_derive::{widget, Msg};

const DATA_DIR: &str = "./data";
const FIRST_YEAR: u32 = 2000;
const LAST_YEAR: u32 = 2100;

pub struct MoneyzModel {
    file_loader: FileLoader,
    relm: relm::Relm<Win>,

    selected_month: data::Month,
    selected_year: data::Year,

    budget_categories: data::BudgetCategories,
    monthly_budget: data::MonthlyBudget,
}

#[derive(Msg)]
pub enum MoneyzMsg {
    ChangeSelectedDate,
    SpendingCellChanged,
    CategoryNameChanged,
    BudgetAmountChanged,
    Save,
    Quit,
}

#[widget]
impl Widget for Win {
    fn model(relm: &relm::Relm<Self>, file_loader: FileLoader) -> MoneyzModel {
        let local: chrono::DateTime<chrono::Local> = chrono::Local::now();
        let current_month = local.date().month() - 1; // chrono starts counting at 1
        let current_year = local.date().year();
        let selected_month: data::Month =
            num_traits::FromPrimitive::from_u32(current_month).unwrap();
        let selected_year = data::Year(current_year as u32);

        // no need to initialize the model there, because when we set the initial date (to
        // today's), the callback already takes care of loading the model
        let budget_categories = Default::default();
        let monthly_budget = Default::default();

        MoneyzModel {
            file_loader,
            relm: relm.clone(),
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
        col.add_attribute(&cell, "text", 1);
        self.budget_categories_tree_view.append_column(&col);

        let budget_categories_tree_view_ref = self.budget_categories_tree_view.clone();
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            debug!("Category name has been changed: {}", value);
            let tree_model = budget_categories_tree_view_ref.get_model().unwrap();
            let model = tree_model.downcast::<gtk::ListStore>().unwrap();
            let mut does_name_already_exist = false;
            model.foreach(|m, _, i| {
                let val = m.get_value(i, 1);
                does_name_already_exist = val.get() == Some(value);
                does_name_already_exist
            });
            if !does_name_already_exist {
                let iter = model.get_iter(&path).unwrap();
                model.set_value(&iter, 1, &Value::from(&value));
                // if the row was the default one, change it to non-default then add another default one
                if model.get_value(&iter, 3).get().unwrap() {
                    model.set_value(&iter, 3, &Value::from(&false));
                    let latest_id = model.get_value(&iter, 0).get::<u32>().unwrap();
                    model.insert_with_values(
                        None,
                        &[0, 1, 2, 3],
                        &[&(latest_id + 1), &"New category", &0, &true],
                    );
                }
                relm.stream().emit(MoneyzMsg::CategoryNameChanged);
            } else {
                debug!("Selected category name already exists!");
            }
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title("Budget amount");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 2);
        self.budget_categories_tree_view.append_column(&col);

        let budget_categories_tree_view_ref = self.budget_categories_tree_view.clone();
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            debug!("Budget amount modified; new value: {}", value);
            match value.parse::<i32>() {
                Err(_) => debug!("'{}' could NOT be parsed into an amount", value),
                Ok(amount) => {
                    let tree_model = budget_categories_tree_view_ref.get_model().unwrap();
                    let model = tree_model.downcast::<gtk::ListStore>().unwrap();
                    let iter = model.get_iter(&path).unwrap();
                    model.set_value(&iter, 2, &Value::from(&amount));
                    debug!("Parsed amount: {}", amount);
                    relm.stream().emit(MoneyzMsg::BudgetAmountChanged);
                }
            }
        });
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
        cell.set_property_text_column(1);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 2);
        col.add_attribute(&cell, "background", 5);
        self.spendings_tree_view.append_column(&col);

        let col = gtk::TreeViewColumn::new();
        col.set_title("Amount");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 3);
        self.spendings_tree_view.append_column(&col);
        let spendings_tree_view_ref = self.spendings_tree_view.clone();
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            debug!("Amount cell modified; new value: {}", value);
            match value.parse::<i32>() {
                Err(_) => debug!("'{}' could NOT be parsed into an amount", value),
                Ok(amount) => {
                    let tree_model = spendings_tree_view_ref.get_model().unwrap();
                    let model = tree_model.downcast::<gtk::ListStore>().unwrap();
                    let iter = model.get_iter(&path).unwrap();
                    model.set_value(&iter, 3, &Value::from(&amount));
                    debug!("Parsed amount: {}", amount);
                    relm.stream().emit(MoneyzMsg::SpendingCellChanged);
                }
            }
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title("Day");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", 4);
        self.spendings_tree_view.append_column(&col);
        //let spendings_tree_view_ref = self.spendings_tree_view.clone();
        //let relm = self.model.relm.clone();
        cell.connect_edited(move |_, _, value| {
            debug!("Day cell modified; new value: {}", value);
            //let value = value.parse::<i32>();
            //if value.is_err() {
            //debug!("'{}' could NOT be parsed into a day", value);
            //return;
            //}

            // TODO: I need to access selected_month / year here, but I can't access the
            // self.model!
            //match data::Day::try_new(value, ) {
            //Err(_) => debug!("'{}' could NOT be parsed into a day", value),
            //Ok(amount) => {
            //let tree_model = spendings_tree_view_ref.get_model().unwrap();
            //let model = tree_model.downcast::<gtk::ListStore>().unwrap();
            //let iter = model.get_iter(&path).unwrap();
            //model.set_value(&iter, 3, &Value::from(&amount));
            //debug!("Parsed amount: {}", amount);
            //relm.stream().emit(MoneyzMsg::SpendingCellChanged);
            //}
            //}
        });
    }

    fn update(&mut self, event: MoneyzMsg) {
        debug!("update");
        match event {
            MoneyzMsg::BudgetAmountChanged => {
                debug!("MoneyzMsg::BudgetAmountChanged");
                self.model.monthly_budget = data_to_model::list_store_to_monthly_budget(
                    self.spendings_tree_view.get_model().unwrap(),
                    self.budget_categories_tree_view.get_model().unwrap(),
                );
                self.model
                    .file_loader
                    .save_monthly_budget(
                        self.model.selected_month,
                        self.model.selected_year,
                        &self.model.monthly_budget,
                    )
                    .unwrap();
            }
            MoneyzMsg::CategoryNameChanged => {
                debug!("MoneyzMsg::CategoryNameChanged");
                // TODO: it's a bit shit not to use a generated int actually, if we rename a
                // category...
                self.model.budget_categories = data_to_model::list_store_to_budget_categories(
                    self.budget_categories_tree_view.get_model().unwrap(),
                );
                self.model
                    .file_loader
                    .save_budget_categories(&self.model.budget_categories)
                    .unwrap();

                // because the name of a category has changed, we want to also update the displayed name in the spendings list
                // to do that, we recreate the ListModel using the new budget categories and the old monthly_budget,
                // then we create the new monthly_budget from the model. A bit awkward, needs some refactoring..?
                self.spendings_tree_view
                    .set_model(Some(&data_to_model::get_spendings_model(
                        &self.model.monthly_budget,
                        &self.model.budget_categories,
                    )));
                self.model.monthly_budget = data_to_model::list_store_to_monthly_budget(
                    self.spendings_tree_view.get_model().unwrap(),
                    self.budget_categories_tree_view.get_model().unwrap(),
                );
                self.model
                    .file_loader
                    .save_monthly_budget(
                        self.model.selected_month,
                        self.model.selected_year,
                        &self.model.monthly_budget,
                    )
                    .unwrap();
            }
            MoneyzMsg::SpendingCellChanged => {
                debug!("MoneyzMsg::SpendingCellChanged");
                self.model.monthly_budget = data_to_model::list_store_to_monthly_budget(
                    self.spendings_tree_view.get_model().unwrap(),
                    self.budget_categories_tree_view.get_model().unwrap(),
                );
                self.model
                    .file_loader
                    .save_monthly_budget(
                        self.model.selected_month,
                        self.model.selected_year,
                        &self.model.monthly_budget,
                    )
                    .unwrap();
            }
            MoneyzMsg::ChangeSelectedDate => {
                debug!("MoneyzMsg::ChangeSelectedDate");
                let selected_month_id = if let Some(id) = self.month_combo_box.get_active() {
                    id
                } else {
                    return;
                };
                self.model.selected_month =
                    num_traits::FromPrimitive::from_u32(selected_month_id).unwrap();
                debug!(
                    "month_combo_box: id is {}, which corresponds to the month {}",
                    selected_month_id,
                    self.model.selected_month.display_string()
                );

                self.model.selected_year = if let Some(id) = self.year_combo_box.get_active() {
                    data::Year(id + FIRST_YEAR)
                } else {
                    return;
                };
                debug!("year_combo_box: year is {}", self.model.selected_year.0);

                self.model.monthly_budget = self
                    .model
                    .file_loader
                    .load_monthly_budget(self.model.selected_month, self.model.selected_year)
                    .unwrap();
            }
            MoneyzMsg::Quit => gtk::main_quit(),
            MoneyzMsg::Save => {
                debug!("MoneyzMsg::Save");
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
                        changed(_) => MoneyzMsg::ChangeSelectedDate,
                    },
                    #[name="year_combo_box"]
                    gtk::ComboBox {
                        changed(_) => MoneyzMsg::ChangeSelectedDate,
                    },
                    #[name="config_button"]
                    gtk::Button {
                        label: "Configuration"
                    },
                    #[name="save_button"]
                    gtk::Button {
                        label: "Save",
                            clicked(_) => MoneyzMsg::Save,
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

        let monthly_budget = self
            .model
            .file_loader
            .load_monthly_budget(self.model.selected_month, self.model.selected_year)
            .unwrap();
        println!("{:?}", monthly_budget);
        let budget_categories = self.model.file_loader.load_budget_categories().unwrap();
        self.initialize_budget_categories_headers();
        self.initialize_spendings_tree_view_headers();

        let budget_categories_model =
            data_to_model::get_model_from_budget_categories_and_monthly_budget(
                &budget_categories,
                &monthly_budget,
            );
        self.budget_categories_tree_view
            .set_model(Some(&budget_categories_model));

        let spendings_model =
            data_to_model::get_spendings_model(&monthly_budget, &budget_categories);
        self.spendings_tree_view.set_model(Some(&spendings_model));
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
    mcb.add_attribute(&cell, "text", 0);
    mcb.set_active(Some(month as u32));

    let cell = gtk::CellRendererText::new();
    let year_model = create_and_fill_year_model();
    ycb.set_model(Some(&year_model));
    ycb.pack_start(&cell, true);
    ycb.add_attribute(&cell, "text", 0);
    ycb.set_active(Some(year.0 - FIRST_YEAR));
}

fn create_and_fill_month_model() -> gtk::ListStore {
    let model = gtk::ListStore::new(&[String::static_type()]);
    for m_idx in 0 as u32..12 {
        let m: Month = num_traits::FromPrimitive::from_u32(m_idx).unwrap();
        model.insert_with_values(None, &[0], &[&m.display_string()]);
    }
    model
}

fn create_and_fill_year_model() -> gtk::ListStore {
    let model = gtk::ListStore::new(&[String::static_type()]);
    for year in FIRST_YEAR..LAST_YEAR {
        model.insert_with_values(None, &[0], &[&year.to_string()]);
    }
    model
}

fn main() {
    env_logger::init();

    let file_loader = FileLoader::new(DATA_DIR);
    Win::run(file_loader).expect("Win::run failed");
}
