//! # Creditcraft
//!
//! Deterministic integer-based credit calculation and token billing engine
//! for metered APIs, LLM workflows, and SaaS consumption models.
//!
//! Built by [Kreive Tech](https://kreive.com.br).
//!
//! ## Overview
//!
//! `creditcraft` operates entirely on signed 32-bit integers (`i32`) to guarantee
//! absolute precision, zero floating-point drift, and seamless database mapping
//! (`INTEGER` in SQLite / PostgreSQL).
//!
//! It provides:
//! - Pure integer fixed-point arithmetic using **micros** ($1.00 = 1,000,000 micros).
//! - Deterministic ceiling division to protect against fractional revenue leakage.
//! - Configurable profit margin percentages (e.g. `+300%`).
//! - Minimum credit floor and optional maximum request ceilings.
//! - Pre-flight capacity checks and atomic in-place balance settlement.
//!
//! ## Quick Start
//!
//! ```rust
//! use creditcraft::{PricingRule, Usage, calculate, settle};
//!
//! // Configure pricing rule (costs in micros, margin in %)
//! let rule = PricingRule::builder()
//!     .unit_input_cost_micros(100)      // $0.00010 per unit
//!     .unit_output_cost_micros(400)     // $0.00040 per unit
//!     .margin_percent(300)              // +300% margin (4x multiplier)
//!     .micros_per_credit(1_000)         // 1 credit = 1,000 micros ($0.001)
//!     .min_credits(1)                   // Floor at 1 credit minimum
//!     .build()
//!     .expect("Valid rule");
//!
//! let usage = Usage::new(1_500, 350);
//! let mut balance = 5_000;
//!
//! // Atomically deduct credits from user's balance
//! let outcome = settle(&rule, usage, &mut balance).expect("Sufficient balance");
//!
//! println!("Credits deducted: {}", outcome.credits_to_deduct);
//! println!("Remaining balance: {}", balance);
//! ```

pub mod calculator;
pub mod error;
pub mod result;
pub mod rule;
pub mod usage;

pub use calculator::CreditEngine;
pub use error::CreditError;
pub use result::{CostBreakdown, CreditResult};
pub use rule::{PricingRule, PricingRuleBuilder};
pub use usage::Usage;

/// Calculates credit requirements and cost breakdown using default engine settings.
///
/// # Example
/// ```rust
/// use creditcraft::{PricingRule, Usage, calculate};
///
/// let rule = PricingRule::default();
/// let usage = Usage::new(100, 50);
/// let result = calculate(&rule, usage, 100).unwrap();
/// assert!(result.has_sufficient_balance);
/// ```
pub fn calculate(
    rule: &PricingRule,
    usage: Usage,
    current_balance: i32,
) -> Result<CreditResult, CreditError> {
    CreditEngine::new().calculate(rule, usage, current_balance)
}

/// Atomically settles and debits credits from a mutable balance in place.
///
/// # Errors
/// Returns [`CreditError::InsufficientBalance`] if available balance is less than required credits.
pub fn settle(
    rule: &PricingRule,
    usage: Usage,
    current_balance: &mut i32,
) -> Result<CreditResult, CreditError> {
    CreditEngine::new().settle(rule, usage, current_balance)
}

/// Checks whether a balance is sufficient to cover an estimated usage amount.
pub fn check_capacity(
    rule: &PricingRule,
    estimated_usage: Usage,
    current_balance: i32,
) -> Result<bool, CreditError> {
    CreditEngine::new().check_capacity(rule, estimated_usage, current_balance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_calculation_with_margin_and_ceil_rounding() {
        // Input: 1,000 units @ 100 micros = 100,000 micros
        // Output: 500 units @ 400 micros = 200,000 micros
        // Total raw: 300,000 micros
        // Margin: +300% (4x) -> commercial = 1,200,000 micros
        // Conversion: 1 credit = 1,000 micros -> 1,200 credits
        let rule = PricingRule::builder()
            .unit_input_cost_micros(100)
            .unit_output_cost_micros(400)
            .margin_percent(300)
            .micros_per_credit(1_000)
            .min_credits(1)
            .build()
            .unwrap();

        let usage = Usage::new(1_000, 500);
        let result = calculate(&rule, usage, 2_000).unwrap();

        assert_eq!(result.raw_cost_micros, 300_000);
        assert_eq!(result.commercial_cost_micros, 1_200_000);
        assert_eq!(result.credits_to_deduct, 1_200);
        assert!(result.has_sufficient_balance);
        assert_eq!(result.remaining_balance, Some(800));
        assert!(!result.breakdown.applied_min_credits);
    }

    #[test]
    fn test_ceiling_division_fractional_protection() {
        // Raw cost: 1 unit @ 10 micros = 10 micros
        // Commercial: 10 * 100% = 10 micros
        // Conversion: 1,000 micros per credit
        // 10 / 1,000 = 0.01 -> ceil division must yield 1 credit
        let rule = PricingRule::builder()
            .unit_input_cost_micros(10)
            .unit_output_cost_micros(0)
            .margin_percent(0)
            .micros_per_credit(1_000)
            .min_credits(0)
            .build()
            .unwrap();

        let usage = Usage::input_only(1);
        let result = calculate(&rule, usage, 50).unwrap();

        assert_eq!(result.commercial_cost_micros, 10);
        assert_eq!(result.credits_to_deduct, 1);
    }

    #[test]
    fn test_min_credits_floor_enforcement() {
        let rule = PricingRule::builder()
            .unit_input_cost_micros(1)
            .unit_output_cost_micros(1)
            .margin_percent(0)
            .micros_per_credit(10_000)
            .min_credits(5)
            .build()
            .unwrap();

        let usage = Usage::new(2, 2);
        let result = calculate(&rule, usage, 10).unwrap();

        assert_eq!(result.credits_to_deduct, 5);
        assert!(result.breakdown.applied_min_credits);
        assert_eq!(result.remaining_balance, Some(5));
    }

    #[test]
    fn test_atomic_settle_success_and_failure() {
        let rule = PricingRule::builder()
            .unit_input_cost_micros(500)
            .unit_output_cost_micros(500)
            .margin_percent(100) // 2x
            .micros_per_credit(1_000)
            .min_credits(1)
            .build()
            .unwrap();

        let usage = Usage::new(10, 10); // raw: 10,000, commercial: 20,000 -> 20 credits
        let mut balance = 25;

        // Successful settlement
        let outcome = settle(&rule, usage, &mut balance).unwrap();
        assert_eq!(outcome.credits_to_deduct, 20);
        assert_eq!(balance, 5);

        // Insufficient balance on second attempt
        let err = settle(&rule, usage, &mut balance).unwrap_err();
        match err {
            CreditError::InsufficientBalance { required, available } => {
                assert_eq!(required, 20);
                assert_eq!(available, 5);
            }
            _ => panic!("Expected InsufficientBalance error"),
        }
        // Balance remains intact
        assert_eq!(balance, 5);
    }

    #[test]
    fn test_preflight_capacity_check() {
        let rule = PricingRule::builder()
            .unit_input_cost_micros(100)
            .margin_percent(0)
            .micros_per_credit(100)
            .build()
            .unwrap();

        let usage = Usage::input_only(50); // 50 credits needed

        assert!(check_capacity(&rule, usage, 100).unwrap());
        assert!(check_capacity(&rule, usage, 50).unwrap());
        assert!(!check_capacity(&rule, usage, 49).unwrap());
    }

    #[test]
    fn test_max_credit_limit_exceeded() {
        let rule = PricingRule::builder()
            .unit_input_cost_micros(1_000)
            .margin_percent(0)
            .micros_per_credit(1_000)
            .max_credit_limit(Some(50))
            .build()
            .unwrap();

        let usage = Usage::input_only(100); // 100 credits attempted
        let err = calculate(&rule, usage, 1_000).unwrap_err();

        match err {
            CreditError::CreditLimitExceeded { limit, attempted } => {
                assert_eq!(limit, 50);
                assert_eq!(attempted, 100);
            }
            _ => panic!("Expected CreditLimitExceeded"),
        }
    }

    #[test]
    fn test_rule_validation_rejects_negative_or_zero_divisor() {
        assert!(PricingRule::builder().micros_per_credit(0).build().is_err());
        assert!(PricingRule::builder().micros_per_credit(-10).build().is_err());
        assert!(PricingRule::builder().unit_input_cost_micros(-5).build().is_err());
        assert!(PricingRule::builder().margin_percent(-1).build().is_err());
        assert!(PricingRule::builder().min_credits(-2).build().is_err());
    }

    #[test]
    fn test_json_serialization() {
        let rule = PricingRule::default();
        let serialized = serde_json::to_string(&rule).expect("Serialize rule");
        let deserialized: PricingRule = serde_json::from_str(&serialized).expect("Deserialize rule");
        assert_eq!(rule, deserialized);
    }
}
