use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(FromPrimitive)]
pub enum Month {
    Jan = 0,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl ToString for Month {
    fn to_string(&self) -> String {
        match &self {
            Month::Jan => "january",
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

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct Year(i32);

impl ToString for Year {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct BudgetCategory(String);

#[derive(Serialize, Deserialize, Debug)]
pub struct BudgetCategories(HashSet<BudgetCategory>);

impl Default for BudgetCategories {
    fn default() -> Self {
        BudgetCategories(HashSet::new())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BudgetAmount(i32);

#[derive(Serialize, Deserialize, Debug)]
pub struct Spending {
    name: String,
    budget_category: BudgetCategory,
    amount: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MonthlyBudget {
    budgets: HashMap<BudgetCategory, BudgetAmount>,
    spendings: Vec<Spending>,
}

impl Default for MonthlyBudget {
    fn default() -> Self {
        MonthlyBudget {
            budgets: HashMap::new(),
            spendings: Vec::new(),
        }
    }
}
