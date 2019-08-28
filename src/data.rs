use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(FromPrimitive, Copy, Clone)]
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

impl Month {
    pub fn display_string(self) -> String {
        match self {
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

    pub fn id_string(self) -> String {
        (self as i32).to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Year(pub u32);

impl ToString for Year {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Day(pub i32);

impl Day {
    pub fn try_new(d: i32, m: Month, y: Year) -> Result<Day, ()> {
        if d <= 0 {
            return Err(());
        }

        match m {
            Month::Jan
            | Month::Mar
            | Month::May
            | Month::Jul
            | Month::Aug
            | Month::Oct
            | Month::Dec => {
                if d <= 31 {
                    Ok(Day(d))
                } else {
                    Err(())
                }
            }
            Month::Apr | Month::Jun | Month::Sep | Month::Nov => {
                if d <= 30 {
                    Ok(Day(d))
                } else {
                    Err(())
                }
            }

            Month::Feb => {
                if d <= 28 {
                    Ok(Day(d))
                } else if d == 29 {
                    if (y.0 % 4 == 0 && y.0 % 100 != 0) || y.0 % 400 == 0 {
                        Ok(Day(d))
                    } else {
                        Err(())
                    }
                } else {
                    Err(())
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct BudgetCategory(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct BudgetCategories(pub HashSet<BudgetCategory>);

impl Default for BudgetCategories {
    fn default() -> Self {
        BudgetCategories(HashSet::new())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BudgetAmount(pub i32);

#[derive(Serialize, Deserialize, Debug)]
pub struct Spending {
    pub name: String,
    pub budget_category: BudgetCategory,
    pub amount: i32,
    // only a day, for the month and year should be known for each MonthlyBudget anyway
    pub day: Day,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spendings(pub Vec<Spending>);

#[derive(Serialize, Deserialize, Debug)]
pub struct MonthlyBudget {
    pub budgets: HashMap<BudgetCategory, BudgetAmount>,
    pub spendings: Spendings,
}

impl Default for MonthlyBudget {
    fn default() -> Self {
        MonthlyBudget {
            budgets: HashMap::new(),
            spendings: Spendings(Vec::new()),
        }
    }
}
