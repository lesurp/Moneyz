use crate::data::{
    BudgetAmount, BudgetCategory, BudgetCategoryId, Day, MoneyAmount, Month, Spending, Year,
};
use crate::data_to_model::{
    add_default_budget_category, add_default_spending,
    get_model_from_budget_categories_and_monthly_budget, get_spendings_model,
    list_model_from_month_year, BudgetCategoriesListStoreIds, BudgetCategoryComboBoxIds,
    SpendingsGtkModelIds,
};
use crate::file_loader::FileLoader;
use crate::translation_provider::TranslationProvider;
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
        let config = file_loader
            .load_config()
            .expect("Could not load the configuration!");
        let translation_provider = TranslationProvider::get_provider(&config.language)
            .expect("Language ID does not exist!");
        let language_list = TranslationProvider::get_language_list();

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
            translation_provider,
            config,
            language_list,
        }
    }

    fn initialize_budget_categories_headers(&self) {
        use BudgetCategoriesListStoreIds::*;
        let col = gtk::TreeViewColumn::new();
        col.set_title(&self.model.translation_provider.budget_category_header());
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Name.into());
        col.add_attribute(&cell, "background", NameBackgroundColor.into());
        self.budget_categories_tree_view.append_column(&col);
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::CategoryNameChanged(path, value.to_owned()));
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title(&self.model.translation_provider.budget_amount_header());
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        // TODO: changing the default line's amoutn does not work with this..?
        col.add_attribute(&cell, "text", Amount.into());
        col.add_attribute(&cell, "background", AmountBackgroundColor.into());
        self.budget_categories_tree_view.append_column(&col);
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::BudgetAmountChanged(path, value.to_owned()));
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title(&self.model.translation_provider.budget_balance_header());
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(false);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Balance.into());
        col.add_attribute(&cell, "background", BalanceBackgroundColor.into());
        self.budget_categories_tree_view.append_column(&col);
    }

    fn initialize_spendings_tree_view_headers(&mut self) {
        use SpendingsGtkModelIds::*;
        let col = gtk::TreeViewColumn::new();
        col.set_title(&self.model.translation_provider.spending_name_header());
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Name.into());
        col.add_attribute(&cell, "background", NameBackgroundColor.into());
        self.spendings_tree_view.append_column(&col);
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::SpendingNameCellChanged(path, value.to_owned()));
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title(
            &self
                .model
                .translation_provider
                .spending_budget_category_header(),
        );
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
        cell.set_property_text_column(BudgetCategoryComboBoxIds::Name.into());
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", CategoryName.into());
        col.add_attribute(&cell, "background", CategoryNameBackgroundColor.into());
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
        col.set_title(&self.model.translation_provider.spending_amount_header());
        let cell = gtk::CellRendererText::new();
        cell.set_property_editable(true);
        col.pack_start(&cell, true);
        col.add_attribute(&cell, "text", Amount.into());
        col.add_attribute(&cell, "background", AmountBackgroundColor.into());
        self.spendings_tree_view.append_column(&col);
        let relm = self.model.relm.clone();
        cell.connect_edited(move |_, path, value| {
            relm.stream()
                .emit(MoneyzMsg::SpendingAmountCellChanged(path, value.to_owned()));
        });

        let col = gtk::TreeViewColumn::new();
        col.set_title(&self.model.translation_provider.spending_day_header());
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
        col.add_attribute(&cell, "background", DayBackgroundColor.into());
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

    fn initialize_language_combo_box(&self) {
        let cell = gtk::CellRendererText::new();
        let language_model = self.create_and_fill_language_model();
        self.language_combo_box.set_model(Some(&language_model));
        self.language_combo_box.pack_start(&cell, true);
        self.language_combo_box.add_attribute(&cell, "text", 1);
        self.language_combo_box.set_id_column(0);
        self.language_combo_box
            .set_active_id(Some(&self.model.config.language));
    }

    fn on_budget_amount_changed(&mut self, path: gtk::TreePath, value: String) {
        debug!("Budget amount modified; new value: {}", value);
        let amount = if let Some(amount) =
            MoneyAmount::from_string(&value, &self.model.translation_provider.decimal_separator())
        {
            amount
        } else {
            debug!("'{}' could NOT be parsed into an amount", value);
            return;
        };
        debug!("Parsed amount: {}", amount.to_i32());

        let budget_category_row = path.get_indices()[0] as usize;
        match self
            .model
            .budget_categories
            .0
            .iter_mut()
            .nth(budget_category_row)
        {
            // we changed the amount for a known budget_category
            Some((id, _)) => {
                self.model
                    .monthly_budget
                    .budgets
                    .insert(*id, BudgetAmount(amount.to_i32()));
            }
            // we changed the amount for the default budget entry
            // therefore we have nothing to update after creating it
            None => {
                let new_id = self
                    .model
                    .budget_categories
                    .0
                    .iter_mut()
                    .last()
                    .map_or(0, |(id, _)| id.0)
                    + 1;
                self.model.budget_categories.0.insert(
                    BudgetCategoryId(new_id),
                    BudgetCategory("default new category_name".to_owned()),
                );
            }
        }

        self.model
            .file_loader
            .save_monthly_budget(
                self.model.selected_month,
                self.model.selected_year,
                &self.model.monthly_budget,
            )
            .unwrap();

        // needed to update the UI - probably much slower than setting the value directly!
        self.update_budget_categories_gtk_model_from_moneyz_model();
    }

    fn on_category_name_changed(&mut self, path: gtk::TreePath, value: String) {
        debug!("Category name has been changed: {}", value);
        let budget_category_row = path.get_indices()[0] as usize;
        for budget_category in self.model.budget_categories.0.values() {
            if budget_category.0 == value {
                debug!("Selected category name already exists!");
                return;
            }
        }

        match &mut self
            .model
            .budget_categories
            .0
            .iter_mut()
            .nth(budget_category_row)
        {
            // budget_category exists - update it
            Some((id, name)) => {
                name.0 = value.clone();
                for spending in &mut self.model.monthly_budget.spendings.0 {
                    if spending.budget_category_id.0 == id.0 {
                        spending.budget_category_name = BudgetCategory(value.to_string());
                    }
                }
            }
            // budget_category does NOT exist - we modified the default entry and have to create
            // and new budget_category
            None => {
                let new_id = self
                    .model
                    .budget_categories
                    .0
                    .iter_mut()
                    .last()
                    .map_or(0, |(id, _)| id.0 + 1);

                self.model
                    .budget_categories
                    .0
                    .insert(BudgetCategoryId(new_id), BudgetCategory(value.clone()));
            }
        }

        // save the model
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

        // update the gtk model
        self.update_budget_categories_gtk_model_from_moneyz_model();
        self.update_monthly_budget_gtk_model_from_moneyz_model();
        let category_model: gtk::ListStore = (&self.model.budget_categories).into();
        let tree_model = category_model.upcast::<gtk::TreeModel>();
        self.model
            .spending_category_combox_box
            .as_ref()
            .unwrap()
            .set_property_model(Some(&tree_model));
    }

    fn on_spending_amount_cell_changed(&mut self, path: gtk::TreePath, value: String) {
        debug!("Amount cell modified; new value: {}", value);
        let amount = if let Some(amount) =
            MoneyAmount::from_string(&value, &self.model.translation_provider.decimal_separator())
        {
            amount
        } else {
            debug!("'{}' could NOT be parsed into an amount", value);
            return;
        };
        debug!("Parsed amount: {}", amount.to_i32());

        let spending_category_row = path.get_indices()[0] as usize;
        match &mut self
            .model
            .monthly_budget
            .spendings
            .0
            .get_mut(spending_category_row)
        {
            // spending - update it
            Some(spending) => {
                spending.amount = amount;
            }
            // spending does NOT exist - we modified the default entry and have to create
            // and new one
            None => {
                // TODO: use translation_provider
                let name = "Default spending name".to_owned();
                let budget_category_id = BudgetCategoryId(u32::max_value());
                let budget_category_name =
                    BudgetCategory("default budget_category_name".to_owned());
                let day = self.model.today;

                self.model.monthly_budget.spendings.0.push(Spending {
                    name,
                    budget_category_id,
                    budget_category_name,
                    amount,
                    day,
                });
            }
        }

        self.model
            .file_loader
            .save_monthly_budget(
                self.model.selected_month,
                self.model.selected_year,
                &self.model.monthly_budget,
            )
            .unwrap();

        self.update_budget_categories_gtk_model_from_moneyz_model();
        self.update_monthly_total_label_from_moneyz_model();
        self.update_monthly_budget_gtk_model_from_moneyz_model();
        self.model
            .file_loader
            .save_monthly_budget(
                self.model.selected_month,
                self.model.selected_year,
                &self.model.monthly_budget,
            )
            .unwrap();
    }

    fn on_spending_name_cell_changed(&mut self, path: gtk::TreePath, value: String) {
        debug!("Spending name has been updated; new value: {}", value);

        let spending_category_row = path.get_indices()[0] as usize;
        match &mut self
            .model
            .monthly_budget
            .spendings
            .0
            .get_mut(spending_category_row)
        {
            // spending exists - update it
            Some(spending) => {
                spending.name = value;
            }
            // spending does NOT exist - we modified the default entry and have to create
            // and new one
            None => {
                // TODO: use translation_provider
                let budget_category_id = BudgetCategoryId(u32::max_value());
                let budget_category_name =
                    BudgetCategory("default budget_category_name".to_owned());
                let day = self.model.today;
                let amount = Default::default();

                self.model.monthly_budget.spendings.0.push(Spending {
                    name: value,
                    budget_category_id,
                    budget_category_name,
                    amount,
                    day,
                });
            }
        }
        self.update_monthly_budget_gtk_model_from_moneyz_model();
        self.model
            .file_loader
            .save_monthly_budget(
                self.model.selected_month,
                self.model.selected_year,
                &self.model.monthly_budget,
            )
            .unwrap();
    }

    fn on_spending_day_cell_changed(&mut self, path: gtk::TreePath, value: String) {
        debug!("Day cell modified; new value: {}", value);
        let day = Day(value.parse::<i32>().unwrap());

        debug!("Parsed day: {}", day.0);
        let spending_category_row = path.get_indices()[0] as usize;
        match &mut self
            .model
            .monthly_budget
            .spendings
            .0
            .get_mut(spending_category_row)
        {
            // spending exists - update it
            Some(spending) => {
                spending.day = day;
            }
            // spending does NOT exist - we modified the default entry and have to create
            // and new one
            None => {
                // TODO: use translation_provider
                let budget_category_id = BudgetCategoryId(u32::max_value());
                let budget_category_name =
                    BudgetCategory("default budget_category_name".to_owned());
                let name = "default_spending_name".to_owned();
                let amount = Default::default();

                self.model.monthly_budget.spendings.0.push(Spending {
                    name,
                    budget_category_id,
                    budget_category_name,
                    amount,
                    day,
                });
            }
        }
        self.update_monthly_budget_gtk_model_from_moneyz_model();
        self.model
            .file_loader
            .save_monthly_budget(
                self.model.selected_month,
                self.model.selected_year,
                &self.model.monthly_budget,
            )
            .unwrap();
    }

    fn on_spending_category_cell_changed(&mut self, path: gtk::TreePath, value: String) {
        let id = self
            .model
            .budget_categories
            .0
            .iter()
            .find(|(_, name)| name.0 == value)
            .map(|(id, _)| id)
            .expect("How come the ID wasn't in the budget_categories?");

        let spending_category_row = path.get_indices()[0] as usize;
        match &mut self
            .model
            .monthly_budget
            .spendings
            .0
            .get_mut(spending_category_row)
        {
            // spending exists - update it
            Some(spending) => {
                spending.budget_category_id = *id;
            }
            // spending does NOT exist - we modified the default entry and have to create
            // and new one
            None => {
                // TODO: use translation_provider
                let budget_category_name =
                    BudgetCategory("default budget_category_name".to_owned());
                let name = "default_spending_name".to_owned();
                let amount = Default::default();
                let day = self.model.today;

                self.model.monthly_budget.spendings.0.push(Spending {
                    name,
                    budget_category_id: *id,
                    budget_category_name,
                    amount,
                    day,
                });
            }
        }
        self.update_monthly_budget_gtk_model_from_moneyz_model();
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
        self.update_monthly_total_label_from_moneyz_model();
        let day_model =
            list_model_from_month_year(self.model.selected_month, self.model.selected_year);
        let tree_model = day_model.upcast::<gtk::TreeModel>();
        self.model
            .spending_day_combox_box
            .as_ref()
            .unwrap()
            .set_property_model(Some(&tree_model));

        // to add new entries, we add a "default" line to the gtk model
        // it does NOT exist in the actual model
    }

    fn on_language_changed(&mut self) {
        let new_language = if let Some(new_language) = self.language_combo_box.get_active_id() {
            new_language.to_string()
        } else {
            return;
        };
        // since the callback is called on startup (when we initialize the selected language),
        // we need this check
        if new_language == self.model.config.language {
            return;
        }
        self.model.config.language = new_language;
        self.model
            .file_loader
            .save_config(&self.model.config)
            .expect("Could not save configuration file!");

        let new_language_provider = TranslationProvider::get_provider(&self.model.config.language)
            .expect("Language ID does not exist!");
        let message_dialog_text = new_language_provider.restart_required_info()
            + "\n\n"
            + &self.model.translation_provider.restart_required_info();
        let mut flags = gtk::DialogFlags::MODAL;
        flags.insert(gtk::DialogFlags::DESTROY_WITH_PARENT);
        let message_dialog = gtk::MessageDialog::new(
            Some(&self.root()),
            flags,
            gtk::MessageType::Info,
            gtk::ButtonsType::Close,
            &message_dialog_text,
        );
        message_dialog.run();
        message_dialog.emit_close();
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
            LanguageChanged => self.on_language_changed(),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Horizontal,
                #[name="config_box"]
                gtk::Box {
                    orientation: Vertical,
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
                        #[name="language_combo_box"]
                        gtk::ComboBox {
                            changed(_) => MoneyzMsg::LanguageChanged,
                            margin_end: MARGIN_BETWEEN,
                        },
                    },
                    #[name="spendings_tree_view"]
                    gtk::TreeView {
                        margin_start: MARGIN_LEFT,
                        margin_end: MARGIN_LEFT,
                    },
                },
                gtk::Box {
                    margin_start: MARGIN_LEFT,
                    orientation: Vertical,
                    #[name="monthly_budget_total_label"]
                    gtk::Label {
                        text: ""
                    },
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
        self.initialize_language_combo_box();

        // everything else is gonna be loaded bby the "on_change_selected_date" event
        self.model.budget_categories = self.model.file_loader.load_budget_categories().unwrap();
    }

    fn update_monthly_budget_gtk_model_from_moneyz_model(&mut self) {
        let spendings_model = get_spendings_model(
            &self.model.monthly_budget,
            &self.model.budget_categories,
            &self.model.translation_provider,
        );
        self.spendings_tree_view.set_model(Some(&spendings_model));
        let tree_model = self.spendings_tree_view.get_model().unwrap();
        let model = tree_model.downcast::<gtk::ListStore>().unwrap();
        add_default_spending(&model, self.model.today);
    }

    fn update_budget_categories_gtk_model_from_moneyz_model(&mut self) {
        let budget_categories_model = get_model_from_budget_categories_and_monthly_budget(
            &self.model.budget_categories,
            &self.model.monthly_budget,
            &self.model.translation_provider,
        );
        self.budget_categories_tree_view
            .set_model(Some(&budget_categories_model));
        let tree_model = self.budget_categories_tree_view.get_model().unwrap();
        let model = tree_model.downcast::<gtk::ListStore>().unwrap();
        add_default_budget_category(&model);
    }

    fn update_monthly_total_label_from_moneyz_model(&mut self) {
        let total = self
            .model
            .monthly_budget
            .spendings
            .0
            .iter()
            .fold(0, |total, spending| total + spending.amount.to_i32());
        // TODO: gotta store the amount in cents!
        let money_amount = MoneyAmount::from_i32(total);
        self.monthly_budget_total_label.set_text(
            &self
                .model
                .translation_provider
                .whole_balance(
                    money_amount.sign(),
                    money_amount.whole(),
                    money_amount.cents_padded(),
                )
                .unwrap(),
        );
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

    fn create_and_fill_language_model(&self) -> gtk::ListStore {
        let model = gtk::ListStore::new(&[String::static_type(), String::static_type()]);
        for (id, display) in &self.model.language_list {
            model.insert_with_values(None, &[0, 1], &[&id, &display]);
        }
        model
    }

    fn month_to_name(&self, m: Month) -> String {
        match m {
            Month::Jan => self.model.translation_provider.january(),
            Month::Feb => self.model.translation_provider.february(),
            Month::Mar => self.model.translation_provider.march(),
            Month::Apr => self.model.translation_provider.april(),
            Month::May => self.model.translation_provider.may(),
            Month::Jun => self.model.translation_provider.june(),
            Month::Jul => self.model.translation_provider.july(),
            Month::Aug => self.model.translation_provider.august(),
            Month::Sep => self.model.translation_provider.september(),
            Month::Oct => self.model.translation_provider.october(),
            Month::Nov => self.model.translation_provider.november(),
            Month::Dec => self.model.translation_provider.december(),
        }
    }
}
