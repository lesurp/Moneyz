use crate::data::{BudgetCategories, Month, MonthlyBudget, Year};
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

const BUDGET_CATEGORIES_FILE: &'static str = "budget_categories.json";

struct FileLoader {
    base_dir: PathBuf,
    budget_categories_path: PathBuf,
}

impl FileLoader {
    pub fn new(base_dir: std::path::PathBuf) -> Self {
        let mut budget_categories_path = PathBuf::from(base_dir.clone());
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
        let monthly_budget_file = y.to_string() + "_" + &m.to_string();
        let mut monthly_budget_path = self.budget_categories_path.clone();
        monthly_budget_path.push(monthly_budget_file);

        FileLoader::load_or_default(monthly_budget_path)
    }

    fn load_or_default<T: serde::de::DeserializeOwned + Default, P: Into<PathBuf>>(
        path: P,
    ) -> Result<T, Box<dyn Error>> {
        let path = path.into();
        if path.exists() {
            let file = std::fs::File::open(path)?;
            let reader = std::io::BufReader::new(file);
            Ok(serde_json::from_reader(reader)?)
        } else {
            Ok(Default::default())
        }
    }

    pub fn save_budget_categories(
        &self,
        budget_categories: BudgetCategories,
    ) -> Result<(), Box<dyn Error>> {
        FileLoader::save(&self.budget_categories_path, budget_categories)
    }

    pub fn save_monthly_budget(
        &self,
        m: Month,
        y: Year,
        monthly_budget: MonthlyBudget,
    ) -> Result<(), Box<dyn Error>> {
        let monthly_budget_file = y.to_string() + "_" + &m.to_string();
        let mut monthly_budget_path = self.budget_categories_path.clone();
        monthly_budget_path.push(monthly_budget_file);

        FileLoader::save(monthly_budget_path, monthly_budget)
    }

    fn save<T: serde::Serialize, P: AsRef<Path>>(path: P, t: T) -> Result<(), Box<dyn Error>> {
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        Ok(serde_json::to_writer(writer, &t)?)
    }
}
