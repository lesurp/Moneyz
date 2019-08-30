use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

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
    pub fn id(self) -> i32 {
        (self as i32) + 1
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

#[derive(Serialize, Deserialize, Debug, Hash, Copy, Clone, PartialEq)]
pub struct BudgetCategoryId(pub u32);

impl Ord for BudgetCategoryId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for BudgetCategoryId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for BudgetCategoryId {}

#[derive(Serialize, Deserialize, Debug)]
pub struct BudgetCategory(pub BudgetCategoryId, pub String);

impl Ord for BudgetCategory {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialEq for BudgetCategory {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for BudgetCategory {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for BudgetCategory {}

impl Hash for BudgetCategory {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl BudgetCategory {
    pub fn id(&self) -> BudgetCategoryId {
        self.0
    }

    pub fn name(&self) -> &str {
        &self.1
    }
}

// the choice of BTreeMap is ordered, therefore, we can easily generate a new UNIQUE id for each
// category by increment the max index by one. Of course we *should* handle overflow but I doubt
// someone's gonna create THOUSANDS OF CATEGORIES GODDAMMIT
#[derive(Serialize, Deserialize, Debug)]
pub struct BudgetCategories(pub BTreeSet<BudgetCategory>);

impl Default for BudgetCategories {
    fn default() -> Self {
        BudgetCategories(BTreeSet::new())
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
    pub budgets: HashMap<BudgetCategoryId, BudgetAmount>,
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
