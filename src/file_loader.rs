use crate::data::{BudgetCategories, Month, MonthlyBudget, Year};
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

const BUDGET_CATEGORIES_FILE: &str = "budget_categories.json";

pub struct FileLoader {
    base_dir: PathBuf,
    budget_categories_path: PathBuf,
}

impl FileLoader {
    pub fn new<P: Into<PathBuf>>(base_dir: P) -> Self {
        let base_dir = base_dir.into();
        if !base_dir.is_dir() {
            std::fs::create_dir_all(&base_dir).unwrap()
        }
        let mut budget_categories_path = base_dir.clone();
        budget_categories_path.push(BUDGET_CATEGORIES_FILE);
        FileLoader {
            base_dir,
            budget_categories_path,
        }
    }

    pub fn load_budget_categories(&self) -> Result<BudgetCategories, Box<dyn Error>> {
        FileLoader::load_or_default(&self.budget_categories_path)
    }

    pub fn load_monthly_budget(&self, m: Month, y: Year) -> Result<MonthlyBudget, Box<dyn Error>> {
        let mut monthly_budget_path = self.base_dir.clone();
        monthly_budget_path.push(FileLoader::month_year_to_filename(m, y));

        FileLoader::load_or_default(monthly_budget_path)
    }

    fn load_or_default<T: serde::de::DeserializeOwned + Default, P: Into<PathBuf>>(
        path: P,
    ) -> Result<T, Box<dyn Error>> {
        let path = path.into();
        if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            Ok(serde_json::from_reader(reader)?)
        } else {
            Ok(Default::default())
        }
    }

    pub fn save_budget_categories(
        &self,
        budget_categories: &BudgetCategories,
    ) -> Result<(), Box<dyn Error>> {
        FileLoader::save(&self.budget_categories_path, budget_categories)
    }

    pub fn save_monthly_budget(
        &self,
        m: Month,
        y: Year,
        monthly_budget: &MonthlyBudget,
    ) -> Result<(), Box<dyn Error>> {
        let mut monthly_budget_path = self.base_dir.clone();
        monthly_budget_path.push(FileLoader::month_year_to_filename(m, y));

        FileLoader::save(monthly_budget_path, monthly_budget)
    }

    fn save<T: serde::Serialize, P: AsRef<Path>>(path: P, t: T) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer(writer, &t)?)
    }

    fn month_year_to_filename(m: Month, y: Year) -> String {
        y.to_string() + "_" + &format!("{:02}", m.id()) + ".json"
    }
}
