use crate::data::{BudgetCategory, Day, Month, Year};
use crate::data_to_model::{
    add_default_budget_category, add_default_spending,
    get_model_from_budget_categories_and_monthly_budget, get_spendings_model,
    list_model_from_month_year, list_store_to_budget_categories, list_store_to_monthly_budget,
    order_spendings_by_day, BudgetCategoriesListStoreIds, SpendingsGtkModelIds,
    BACKGROUND_COLOR_NORMAL,
};
use crate::file_loader::FileLoader;
use crate::translation_provider;
use crate::{MoneyzModel, MoneyzMsg};
use chrono::Datelike;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::*;
use log::debug;
use relm::{connect, connect_stream, Widget};
use relm_derive::widget;

const MARGIN_LEFT: i32 = 15;
const MARGIN_BETWEEN: i32 = 3;

const FIRST_YEAR: u32 = 2000;
const LAST_YEAR: u32 = 2100;

#[widget]
impl Widget for MainWindow {
    fn model(relm: &relm::Relm<Self>, file_loader: FileLoader) -> MoneyzModel {
        let local: chrono::DateTime<chrono::Local> = chrono::Local::now();
        let today = Day(local.date().day() as i32);
        let current_month = local.date().month() - 1; // chrono starts counting at 1
        let current_year = local.date().year();
        let selected_month: Month = num_traits::FromPrimitive::from_u32(current_month).unwrap();
        let selected_year = Year(current_year as u32);

        // no need to initialize the model there, because when we set the initial date (to
        // today's), the callback already takes care of loading the model
        let budget_categories = Default::default();
        let monthly_budget = Default::default();

        MoneyzModel {
            file_loader,
            relm: relm.clone(),
            spending_category_combox_box: None,
            spending_day_combox_box: None,
            selected_month,
            selected_year,
            today,
            budget_categories,
            monthly_budget,
            translation_provider: translation_provider::get("fr"),
        }
    }

    fn initialize_budget_categories_headers(&self) {
        use BudgetCategoriesListStoreIds::*;
        let col = gtk::TreeViewColumn::new();
        col.set_title("Budget category name");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Name.into());
        col.add_attribute(&cell, "background", BackgroundColor.into());
        self.budget_categories_tree_view.append_column(&col);
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::CategoryNameChanged(path, value.to_owned()));
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title("Budget amount");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Amount.into());
        col.add_attribute(&cell, "background", BackgroundColor.into());
        self.budget_categories_tree_view.append_column(&col);
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::BudgetAmountChanged(path, value.to_owned()));
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title("Budget surplus");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(false);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Surplus.into());
        col.add_attribute(&cell, "background", BackgroundColor.into());
        self.budget_categories_tree_view.append_column(&col);
    }

    fn initialize_spendings_tree_view_headers(&mut self) {
        use SpendingsGtkModelIds::*;
        let col = gtk::TreeViewColumn::new();
        col.set_title("Name");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Name.into());
        col.add_attribute(&cell, "background", BackgroundColor.into());
        self.spendings_tree_view.append_column(&col);
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::SpendingNameCellChanged(path, value.to_owned()));
        });

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
        col.add_attribute(&cell, "text", CategoryName.into());
        col.add_attribute(&cell, "background", BackgroundColor.into());
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream().emit(MoneyzMsg::SpendingCategoryCellChanged(
                path,
                value.to_owned(),
            ));
        });
        self.spendings_tree_view.append_column(&col);
        self.model.spending_category_combox_box = Some(cell);

        let col = gtk::TreeViewColumn::new();
        col.set_title("Amount");
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Amount.into());
        col.add_attribute(&cell, "background", BackgroundColor.into());
        self.spendings_tree_view.append_column(&col);
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::SpendingAmountCellChanged(path, value.to_owned()));
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title("Day");
        let cell = gtk::CellRendererCombo::new();
        let day_model =
            list_model_from_month_year(self.model.selected_month, self.model.selected_year);
        let tree_model = day_model.upcast::<gtk::TreeModel>();
        cell.set_property_model(Some(&tree_model));
        cell.set_property_editable(true);
        cell.set_property_has_entry(false);
        cell.set_property_text_column(0);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Day.into());
        col.add_attribute(&cell, "background", BackgroundColor.into());
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::SpendingDayCellChanged(path, value.to_owned()));
        });
        self.spendings_tree_view.append_column(&col);
        self.model.spending_day_combox_box = Some(cell);
    }

    fn initialize_month_year_combo_boxes(&self) {
        let cell = gtk::CellRendererText::new();
        let month_model = self.create_and_fill_month_model();
        self.month_combo_box.set_model(Some(&month_model));
        self.month_combo_box.pack_start(&cell, true);
        self.month_combo_box.add_attribute(&cell, "text", 0);
        self.month_combo_box
            .set_active(Some(self.model.selected_month as u32));

        let cell = gtk::CellRendererText::new();
        let year_model = self.create_and_fill_year_model();
        self.year_combo_box.set_model(Some(&year_model));
        self.year_combo_box.pack_start(&cell, true);
        self.year_combo_box.add_attribute(&cell, "text", 0);
        self.year_combo_box
            .set_active(Some(self.model.selected_year.0 - FIRST_YEAR));
    }

    fn on_budget_amount_changed(&mut self, path: gtk::TreePath, value: String) {
        debug!("Budget amount modified; new value: {}", value);
        let amount = if let Ok(amount) = value.parse::<i32>() {
            amount
        } else {
            debug!("'{}' could NOT be parsed into an amount", value);
            return;
        };

        // TODO since all the work is done here we may as well modify directly self.model
        // and then update the gtk model rather doing gtk -> self -> gtk...
        let tree_model = self.budget_categories_tree_view.get_model().unwrap();
        let model = tree_model.downcast::<gtk::ListStore>().unwrap();
        let iter = model.get_iter(&path).unwrap();
        model.set_value(&iter, 2, &Value::from(&amount));
        debug!("Parsed amount: {}", amount);
        self.update_monthly_budget_moneyz_model_from_gtk_model();
        self.update_budget_categories_gtk_model_from_moneyz_model();
    }

    fn on_category_name_changed(&mut self, path: gtk::TreePath, value: String) {
        use BudgetCategoriesListStoreIds::*;
        debug!("Category name has been changed: {}", value);
        let tree_model = self.budget_categories_tree_view.get_model().unwrap();
        let model = tree_model.downcast::<gtk::ListStore>().unwrap();
        let mut does_name_already_exist = false;
        model.foreach(|m, _, i| {
            let val = m.get_value(i, Name.into());
            does_name_already_exist = val.get() == Some(&*value);
            does_name_already_exist
        });
        if does_name_already_exist {
            debug!("Selected category name already exists!");
            return;
        }

        let iter = model.get_iter(&path).unwrap();
        model.set_value(&iter, Name as u32, &Value::from(&value));
        // if the row was the default one, change it to non-default then add another default one
        if model.get_value(&iter, IsDefault.into()).get().unwrap() {
            model.set_value(&iter, IsDefault.into(), &Value::from(&false));
            model.set_value(
                &iter,
                BackgroundColor.into(),
                &Value::from(&BACKGROUND_COLOR_NORMAL),
            );
            let max_id = model.get_value(&iter, Id.into()).get::<u32>().unwrap();
            add_default_budget_category(&model, max_id);
        }
        self.update_budget_categories_moneyz_model_from_gtk_model();

        // because the name of a category has changed, we want to also update the displayed name in the spendings list
        // to do that, we recreate the ListModel using the new budget categories and the old monthly_budget,
        // then we create the new monthly_budget from the model. A bit awkward, needs some refactoring..?
        self.spendings_tree_view
            .set_model(Some(&get_spendings_model(
                &self.model.monthly_budget,
                &self.model.budget_categories,
                self.model.today,
            )));
        self.update_monthly_budget_moneyz_model_from_gtk_model();

        let category_model: gtk::ListStore = (&self.model.budget_categories).into();
        let tree_model = category_model.upcast::<gtk::TreeModel>();
        self.model
            .spending_category_combox_box
            .as_ref()
            .unwrap()
            .set_property_model(Some(&tree_model));
    }

    fn on_spending_amount_cell_changed(&mut self, path: gtk::TreePath, value: String) {
        use SpendingsGtkModelIds::*;
        debug!("Amount cell modified; new value: {}", value);
        let amount = if let Ok(amount) = value.parse::<i32>() {
            amount
        } else {
            debug!("'{}' could NOT be parsed into an amount", value);
            return;
        };

        let tree_model = self.spendings_tree_view.get_model().unwrap();
        let model = tree_model.downcast::<gtk::ListStore>().unwrap();
        let iter = model.get_iter(&path).unwrap();
        model.set_value(&iter, Amount.into(), &Value::from(&amount));
        model.set_value(
            &iter,
            BackgroundColor.into(),
            &Value::from(&BACKGROUND_COLOR_NORMAL),
        );
        if model.get_value(&iter, IsDefault.into()).get().unwrap() {
            model.set_value(&iter, IsDefault.into(), &Value::from(&false));
            add_default_spending(&model, self.model.today);
        }
        debug!("Parsed amount: {}", amount);
        self.update_monthly_budget_moneyz_model_from_gtk_model();
        self.update_budget_categories_gtk_model_from_moneyz_model();
    }

    fn on_spending_name_cell_changed(&mut self, path: gtk::TreePath, value: String) {
        use SpendingsGtkModelIds::*;
        debug!("Spending name has been updated; new value: {}", value);
        let tree_model = self.spendings_tree_view.get_model().unwrap();
        let model = tree_model.downcast::<gtk::ListStore>().unwrap();
        let iter = model.get_iter(&path).unwrap();
        model.set_value(&iter, Name.into(), &Value::from(&value));
        model.set_value(
            &iter,
            BackgroundColor.into(),
            &Value::from(&BACKGROUND_COLOR_NORMAL),
        );
        if model.get_value(&iter, IsDefault.into()).get().unwrap() {
            model.set_value(&iter, IsDefault.into(), &Value::from(&false));
            add_default_spending(&model, self.model.today);
        }
        self.update_monthly_budget_moneyz_model_from_gtk_model();
    }

    fn on_spending_day_cell_changed(&mut self, path: gtk::TreePath, value: String) {
        use SpendingsGtkModelIds::*;
        debug!("Day cell modified; new value: {}", value);
        let day = value.parse::<i32>().unwrap();

        debug!("Parsed amount: {}", day);
        let tree_model = self.spendings_tree_view.get_model().unwrap();
        let model = tree_model.downcast::<gtk::ListStore>().unwrap();
        let iter = model.get_iter(&path).unwrap();
        model.set_value(&iter, Day.into(), &Value::from(&day));
        model.set_value(
            &iter,
            BackgroundColor.into(),
            &Value::from(&BACKGROUND_COLOR_NORMAL),
        );
        if model.get_value(&iter, IsDefault.into()).get().unwrap() {
            model.set_value(&iter, IsDefault.into(), &Value::from(&false));
            order_spendings_by_day(&model);
            add_default_spending(&model, self.model.today);
        }

        self.update_monthly_budget_moneyz_model_from_gtk_model();
    }

    fn on_spending_category_cell_changed(&mut self, path: gtk::TreePath, value: String) {
        use SpendingsGtkModelIds::*;
        let id = (|| {
            for BudgetCategory(id, name) in &self.model.budget_categories.0 {
                if name == &value {
                    return id;
                }
            }
            panic!("How come the ID wasn't in the budget_categories?");
        })();

        let tree_model = self.spendings_tree_view.get_model().unwrap();
        let model = tree_model.downcast::<gtk::ListStore>().unwrap();
        let iter = model.get_iter(&path).unwrap();
        model.set_value(&iter, CategoryId.into(), &Value::from(&id.0));
        model.set_value(&iter, CategoryName.into(), &Value::from(&value));
        model.set_value(
            &iter,
            BackgroundColor.into(),
            &Value::from(&BACKGROUND_COLOR_NORMAL),
        );
        if model.get_value(&iter, IsDefault.into()).get().unwrap() {
            model.set_value(&iter, IsDefault.into(), &Value::from(&false));
            add_default_spending(&model, self.model.today);
        }
        self.update_monthly_budget_moneyz_model_from_gtk_model();
        self.update_budget_categories_gtk_model_from_moneyz_model();
    }

    fn on_change_selected_date(&mut self) {
        let selected_month_id = if let Some(id) = self.month_combo_box.get_active() {
            id
        } else {
            return;
        };
        self.model.selected_month = num_traits::FromPrimitive::from_u32(selected_month_id).unwrap();
        debug!(
            "month_combo_box: id is {}, which corresponds to the month {}",
            selected_month_id,
            self.month_to_name(self.model.selected_month),
        );

        self.model.selected_year = if let Some(id) = self.year_combo_box.get_active() {
            Year(id + FIRST_YEAR)
        } else {
            return;
        };
        debug!("year_combo_box: year is {}", self.model.selected_year.0);

        self.model.monthly_budget = self
            .model
            .file_loader
            .load_monthly_budget(self.model.selected_month, self.model.selected_year)
            .unwrap();
        self.update_budget_categories_gtk_model_from_moneyz_model();
        self.update_monthly_budget_gtk_model_from_moneyz_model();

        let day_model =
            list_model_from_month_year(self.model.selected_month, self.model.selected_year);
        let tree_model = day_model.upcast::<gtk::TreeModel>();
        self.model
            .spending_day_combox_box
            .as_ref()
            .unwrap()
            .set_property_model(Some(&tree_model));
    }

    fn update(&mut self, event: MoneyzMsg) {
        use MoneyzMsg::*;
        debug!("Update with message: {:?}", event);
        match event {
            BudgetAmountChanged(path, value) => self.on_budget_amount_changed(path, value),
            CategoryNameChanged(path, value) => self.on_category_name_changed(path, value),
            SpendingAmountCellChanged(path, value) => {
                self.on_spending_amount_cell_changed(path, value)
            }
            SpendingNameCellChanged(path, value) => self.on_spending_name_cell_changed(path, value),
            SpendingDayCellChanged(path, value) => self.on_spending_day_cell_changed(path, value),
            SpendingCategoryCellChanged(path, value) => {
                self.on_spending_category_cell_changed(path, value)
            }
            ChangeSelectedDate => self.on_change_selected_date(),
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
                        changed(_) => MoneyzMsg::ChangeSelectedDate,
                        margin_start: MARGIN_LEFT,
                        margin_end: MARGIN_BETWEEN,
                    },
                    #[name="year_combo_box"]
                    gtk::ComboBox {
                        changed(_) => MoneyzMsg::ChangeSelectedDate,
                        margin_end: MARGIN_BETWEEN,
                    },
                    #[name="config_button"]
                    gtk::Button {
                        label: "Configuration"
                    },
                },
                gtk::Box {
                    orientation: Horizontal,
                    #[name="spendings_tree_view"]
                    gtk::TreeView {
                        margin_start: MARGIN_LEFT,
                        margin_end: MARGIN_LEFT,
                    },
                    gtk::Separator { orientation: Vertical },
                    #[name="budget_categories_tree_view"]
                    gtk::TreeView {
                        margin_start: MARGIN_LEFT,
                    },
                },
            },
            delete_event(_, _) => (MoneyzMsg::Quit, Inhibit(false)),
        }
    }

    fn init_view(&mut self) {
        self.initialize_budget_categories_headers();
        self.initialize_spendings_tree_view_headers();
        self.initialize_month_year_combo_boxes();

        self.model.monthly_budget = self
            .model
            .file_loader
            .load_monthly_budget(self.model.selected_month, self.model.selected_year)
            .unwrap();
        self.model.budget_categories = self.model.file_loader.load_budget_categories().unwrap();

        self.update_budget_categories_gtk_model_from_moneyz_model();
        self.update_monthly_budget_gtk_model_from_moneyz_model();
    }

    fn update_monthly_budget_gtk_model_from_moneyz_model(&mut self) {
        let spendings_model = get_spendings_model(
            &self.model.monthly_budget,
            &self.model.budget_categories,
            self.model.today,
        );
        self.spendings_tree_view.set_model(Some(&spendings_model));
    }

    fn update_budget_categories_gtk_model_from_moneyz_model(&mut self) {
        let budget_categories_model = get_model_from_budget_categories_and_monthly_budget(
            &self.model.budget_categories,
            &self.model.monthly_budget,
        );
        self.budget_categories_tree_view
            .set_model(Some(&budget_categories_model));
    }

    fn update_monthly_budget_moneyz_model_from_gtk_model(&mut self) {
        self.model.monthly_budget = list_store_to_monthly_budget(
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

    fn update_budget_categories_moneyz_model_from_gtk_model(&mut self) {
        self.model.budget_categories =
            list_store_to_budget_categories(self.budget_categories_tree_view.get_model().unwrap());

        self.model
            .file_loader
            .save_budget_categories(&self.model.budget_categories)
            .unwrap();
    }

    fn create_and_fill_month_model(&self) -> gtk::ListStore {
        let model = gtk::ListStore::new(&[String::static_type()]);
        for m_idx in 0 as u32..12 {
            let m: Month = num_traits::FromPrimitive::from_u32(m_idx).unwrap();
            model.insert_with_values(None, &[0], &[&self.month_to_name(m)]);
        }
        model
    }

    fn create_and_fill_year_model(&self) -> gtk::ListStore {
        let model = gtk::ListStore::new(&[String::static_type()]);
        for year in FIRST_YEAR..LAST_YEAR {
            model.insert_with_values(None, &[0], &[&year.to_string()]);
        }
        model
    }

    fn month_to_name(&self, m: Month) -> String {
        match m {
            Month::Jan => &self.model.translation_provider.trans().jan,
            Month::Feb => "february",
            Month::Mar => "march",
            Month::Apr => "april",
            Month::May => "may",
            Month::Jun => "june",
            Month::Jul => "july",
            Month::Aug => "august",
            Month::Sep => "september",
            Month::Oct => "october",
            Month::Nov => "november",
            Month::Dec => "december",
        }
        .to_owned()
    }
}
