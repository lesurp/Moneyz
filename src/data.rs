use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BudgetCategory(pub String);

// the choice of BTreeMap is ordered, therefore, we can easily generate a new UNIQUE id for each
// category by increment the max index by one. Of course we *should* handle overflow but I doubt
// someone's gonna create THOUSANDS OF CATEGORIES GODDAMMIT
#[derive(Serialize, Deserialize, Debug)]
pub struct BudgetCategories(pub BTreeMap<BudgetCategoryId, BudgetCategory>);

impl Default for BudgetCategories {
    fn default() -> Self {
        BudgetCategories(BTreeMap::new())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BudgetAmount(pub i32);

#[derive(Serialize, Deserialize, Debug)]
pub struct Spending {
    pub name: String,
    // TODO: need to use Option<T>
    // we use a flag for default with a dummy category value,
    // when reloading the program we reload the line with the defualt id which is obv wrong
    // note that setting the default id to max u32 would also work...
    // note2: Option<T> may not be that great if gtk doesn't support NULL model entries
    // Then I'd still need a default value which would conflict with the Spending value...
    pub budget_category_id: BudgetCategoryId,
    pub budget_category_name: BudgetCategory,
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

pub enum MoneyAmountType {
    Credit,
    Debit,
}

pub struct MoneyAmount {
    amount_type: MoneyAmountType,
    whole: u32,
    cents: u32,
}

impl MoneyAmount {
    pub fn from_string(amount_str: &str, decimal_separator: &str) -> Option<Self> {
        let split_val: Vec<_> = amount_str.split(decimal_separator).collect();
        let (whole, cents) = match split_val.len() {
            1 => {
                let whole = split_val[0].parse::<i32>().ok()?;
                (whole, 0)
            }
            2 => {
                let whole = split_val[0].parse::<i32>().ok()?;
                let cents = split_val[1].parse::<u32>().ok()?;
                (whole, cents)
            }
            _ => return None,
        };

        if cents > 100 {
            return None;
        }

        let amount_type = if whole >= 0 {
            MoneyAmountType::Credit
        } else {
            MoneyAmountType::Debit
        };

        Some(MoneyAmount {
            whole: whole.abs() as u32,
            cents,
            amount_type,
        })
    }

    pub fn from_i32(amount: i32) -> Self {
        let amount_type = if amount >= 0 {
            MoneyAmountType::Credit
        } else {
            MoneyAmountType::Debit
        };

        let amount = amount.abs() as u32;
        let whole = amount / 100;
        let cents = amount - 100 * whole;

        MoneyAmount {
            whole,
            cents,
            amount_type,
        }
    }

    pub fn to_i32(&self) -> i32 {
        (self.whole * 100 + self.cents) as i32
            * match self.amount_type {
                MoneyAmountType::Debit => -1,
                MoneyAmountType::Credit => 1,
            }
    }

    pub fn whole(&self) -> u32 {
        self.whole
    }

    pub fn cents_padded(&self) -> String {
        format!("{:02}", self.cents)
    }

    pub fn sign(&self) -> String {
        match self.amount_type {
            MoneyAmountType::Debit => "-".to_owned(),
            MoneyAmountType::Credit => "".to_owned(),
        }
    }
}
