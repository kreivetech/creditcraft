use serde::{Deserialize, Serialize};

/// Detailed breakdown of cost components calculated for an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Raw un-margined cost of input units in micros.
    pub input_raw_micros: i32,
    /// Raw un-margined cost of output units in micros.
    pub output_raw_micros: i32,
    /// Total raw un-margined cost (input + output) in micros.
    pub total_raw_micros: i32,
    /// Amount of profit margin added on top of raw cost in micros.
    pub margin_added_micros: i32,
    /// Commercial cost before conversion to credits in micros.
    pub commercial_cost_micros: i32,
    /// Number of credits calculated before applying the minimum floor rule.
    pub unfloored_credits: i32,
    /// Indicates whether the minimum credit floor was enforced.
    pub applied_min_credits: bool,
}

/// Comprehensive outcome of a credit calculation and balance verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CreditResult {
    /// Raw underlying provider cost in micros.
    pub raw_cost_micros: i32,
    /// Total commercial cost after applying profit margin in micros.
    pub commercial_cost_micros: i32,
    /// Final credits to deduct (integer with ceil rounding applied).
    pub credits_to_deduct: i32,
    /// Whether the user's available balance covers the required credits.
    pub has_sufficient_balance: bool,
    /// Remaining balance after deduction (None if balance was insufficient).
    pub remaining_balance: Option<i32>,
    /// Granular breakdown of individual cost and margin elements.
    pub breakdown: CostBreakdown,
}

impl CreditResult {
    /// Returns true if the operation can proceed within the given balance.
    pub const fn is_approved(&self) -> bool {
        self.has_sufficient_balance
    }
}
