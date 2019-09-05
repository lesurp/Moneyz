mod config;
mod data;
mod data_to_model;
mod file_loader;
mod main_window;
mod translation_provider;

use relm::Widget;

pub struct MoneyzModel {
    file_loader: file_loader::FileLoader,
    relm: relm::Relm<main_window::MainWindow>,
    spending_category_combox_box: Option<gtk::CellRendererCombo>,
    spending_day_combox_box: Option<gtk::CellRendererCombo>,

    selected_month: data::Month,
    selected_year: data::Year,
    today: data::Day,

    budget_categories: data::BudgetCategories,
    monthly_budget: data::MonthlyBudget,

    translation_provider: translation_provider::TranslationProvider,
    config: config::Config,
    language_list: Vec<(String, String)>,
}

#[derive(relm_derive::Msg, Debug)]
pub enum MoneyzMsg {
    ChangeSelectedDate,
    LanguageChanged,
    BudgetCategoriesDeleteKeyPressed,
    SpendingsDeleteKeyPressed,
    SpendingCategoryCellChanged(gtk::TreePath, String),
    SpendingNameCellChanged(gtk::TreePath, String),
    SpendingAmountCellChanged(gtk::TreePath, String),
    SpendingDayCellChanged(gtk::TreePath, String),
    CategoryNameChanged(gtk::TreePath, String),
    BudgetAmountChanged(gtk::TreePath, String),
    Quit,
}

const DATA_DIR: &str = "./data";

fn main() {
    env_logger::init();

    let file_loader = file_loader::FileLoader::new(DATA_DIR);
    main_window::MainWindow::run(file_loader).expect("MainWindow::run failed");
}
