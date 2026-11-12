use crate::error::CreditError;
use crate::result::{CostBreakdown, CreditResult};
use crate::rule::PricingRule;
use crate::usage::Usage;

/// Integer ceiling division for positive numbers.
///
/// Ensures any fractional remainder always rounds up to prevent
/// revenue leakage on fractional currency or credit amounts.
#[inline]
pub(crate) const fn div_ceil_i64(a: i64, b: i64) -> i64 {
    if a <= 0 || b <= 0 {
        0
    } else {
        (a + b - 1) / b
    }
}

/// Pure integer credit calculation engine.
#[derive(Debug, Clone, Copy, Default)]
pub struct CreditEngine;

impl CreditEngine {
    /// Creates a new `CreditEngine` instance.
    pub const fn new() -> Self {
        Self
    }

    /// Calculates required credits and cost breakdown based on pricing rules and usage.
    ///
    /// # Arguments
    /// * `rule` - Pricing parameters including costs, margin, and credit conversion.
    /// * `usage` - Number of input and output units consumed.
    /// * `current_balance` - User's available credits at the start of the operation.
    ///
    /// # Returns
    /// * `Ok(CreditResult)` with all calculated fields and balance status.
    /// * `Err(CreditError)` if rules are invalid or credit limit was exceeded.
    pub fn calculate(
        &self,
        rule: &PricingRule,
        usage: Usage,
        current_balance: i32,
    ) -> Result<CreditResult, CreditError> {
        rule.validate()?;

        let input_units = usage.input_units.max(0) as i64;
        let output_units = usage.output_units.max(0) as i64;

        let input_unit_cost = rule.unit_input_cost_micros as i64;
        let output_unit_cost = rule.unit_output_cost_micros as i64;

        // Raw provider costs in micros
        let raw_input_micros = input_units
            .checked_mul(input_unit_cost)
            .ok_or(CreditError::CalculationOverflow)?;

        let raw_output_micros = output_units
            .checked_mul(output_unit_cost)
            .ok_or(CreditError::CalculationOverflow)?;

        let total_raw_micros = raw_input_micros
            .checked_add(raw_output_micros)
            .ok_or(CreditError::CalculationOverflow)?;

        // Apply profit margin percentage: commercial = raw * (100 + margin%) / 100 (ceil)
        let margin_multiplier = 100_i64
            .checked_add(rule.margin_percent as i64)
            .ok_or(CreditError::CalculationOverflow)?;

        let margined_product = total_raw_micros
            .checked_mul(margin_multiplier)
            .ok_or(CreditError::CalculationOverflow)?;

        let commercial_cost_micros = div_ceil_i64(margined_product, 100);

        let margin_added_micros = commercial_cost_micros
            .checked_sub(total_raw_micros)
            .unwrap_or(0);

        // Convert commercial micros to credits with integer ceiling rounding
        let micros_per_credit = rule.micros_per_credit as i64;
        let unfloored_credits = div_ceil_i64(commercial_cost_micros, micros_per_credit);

        // Apply minimum credit floor rule
        let min_credits = rule.min_credits as i64;
        let (final_credits, applied_min) = if unfloored_credits < min_credits {
            (min_credits, true)
        } else {
            (unfloored_credits, false)
        };

        // Cast safely back to i32
        let raw_cost_i32 = i32::try_from(total_raw_micros)
            .map_err(|_| CreditError::CalculationOverflow)?;
        let commercial_cost_i32 = i32::try_from(commercial_cost_micros)
            .map_err(|_| CreditError::CalculationOverflow)?;
        let credits_to_deduct = i32::try_from(final_credits)
            .map_err(|_| CreditError::CalculationOverflow)?;

        // Check if a maximum credit ceiling rule was violated
        if let Some(limit) = rule.max_credit_limit {
            if credits_to_deduct > limit {
                return Err(CreditError::CreditLimitExceeded {
                    limit,
                    attempted: credits_to_deduct,
                });
            }
        }

        let has_sufficient_balance = current_balance >= credits_to_deduct;
        let remaining_balance = if has_sufficient_balance {
            Some(current_balance - credits_to_deduct)
        } else {
            None
        };

        let breakdown = CostBreakdown {
            input_raw_micros: i32::try_from(raw_input_micros).unwrap_or(i32::MAX),
            output_raw_micros: i32::try_from(raw_output_micros).unwrap_or(i32::MAX),
            total_raw_micros: raw_cost_i32,
            margin_added_micros: i32::try_from(margin_added_micros).unwrap_or(i32::MAX),
            commercial_cost_micros: commercial_cost_i32,
            unfloored_credits: i32::try_from(unfloored_credits).unwrap_or(i32::MAX),
            applied_min_credits: applied_min,
        };

        Ok(CreditResult {
            raw_cost_micros: raw_cost_i32,
            commercial_cost_micros: commercial_cost_i32,
            credits_to_deduct,
            has_sufficient_balance,
            remaining_balance,
            breakdown,
        })
    }

    /// Atomically validates and debits credits from a mutable balance in place.
    ///
    /// # Errors
    /// * `CreditError::InsufficientBalance` if `*current_balance < credits_to_deduct`.
    pub fn settle(
        &self,
        rule: &PricingRule,
        usage: Usage,
        current_balance: &mut i32,
    ) -> Result<CreditResult, CreditError> {
        let result = self.calculate(rule, usage, *current_balance)?;

        if !result.has_sufficient_balance {
            return Err(CreditError::InsufficientBalance {
                required: result.credits_to_deduct,
                available: *current_balance,
            });
        }

        *current_balance = result
            .remaining_balance
            .expect("Remaining balance exists when sufficient");

        Ok(result)
    }

    /// Fast pre-flight credit capacity check without altering balance.
    ///
    /// Useful for validating whether a user can initiate an operation before
    /// dispatching network requests to external providers.
    pub fn check_capacity(
        &self,
        rule: &PricingRule,
        estimated_usage: Usage,
        current_balance: i32,
    ) -> Result<bool, CreditError> {
        let result = self.calculate(rule, estimated_usage, current_balance)?;
        Ok(result.has_sufficient_balance)
    }
}
