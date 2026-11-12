use serde::{Deserialize, Serialize};

use crate::error::CreditError;

/// Pricing parameters and business rules used to calculate and convert
/// metered unit consumption into billable credits.
///
/// All monetary values are represented as scaled integers in **micros**
/// (where 1 standard currency unit = 1,000,000 micros).
///
/// # Example
/// ```rust
/// use creditcraft::PricingRule;
///
/// let rule = PricingRule::builder()
///     .unit_input_cost_micros(100)       // $0.00010 per unit
///     .unit_output_cost_micros(400)      // $0.00040 per unit
///     .margin_percent(300)               // +300% profit margin (4x total)
///     .micros_per_credit(1_000)          // 1 credit = 1,000 micros ($0.001)
///     .min_credits(1)                    // Minimum 1 credit per operation
///     .build()
///     .expect("Valid pricing rule");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricingRule {
    /// Cost per input unit in micros (e.g. 100 micros for $0.00010).
    pub unit_input_cost_micros: i32,
    /// Cost per output unit in micros (e.g. 400 micros for $0.00040).
    pub unit_output_cost_micros: i32,
    /// Profit margin added on top of raw cost, expressed as an integer percentage
    /// (e.g. `300` means +300% margin, resulting in a 400% commercial multiplier).
    pub margin_percent: i32,
    /// Number of micros that equate to 1 full credit (e.g. `1_000` micros = 1 credit).
    pub micros_per_credit: i32,
    /// Floor of minimum credits charged for any billable execution.
    pub min_credits: i32,
    /// Optional maximum credit ceiling permitted per request.
    pub max_credit_limit: Option<i32>,
}

impl Default for PricingRule {
    fn default() -> Self {
        Self {
            unit_input_cost_micros: 0,
            unit_output_cost_micros: 0,
            margin_percent: 0,
            micros_per_credit: 1_000,
            min_credits: 1,
            max_credit_limit: None,
        }
    }
}

impl PricingRule {
    /// Returns a new builder to configure a `PricingRule`.
    pub fn builder() -> PricingRuleBuilder {
        PricingRuleBuilder::default()
    }

    /// Validates that rule parameters are within sound mathematical bounds.
    pub fn validate(&self) -> Result<(), CreditError> {
        if self.micros_per_credit <= 0 {
            return Err(CreditError::InvalidRule(
                "micros_per_credit must be strictly greater than zero".to_string(),
            ));
        }
        if self.unit_input_cost_micros < 0 {
            return Err(CreditError::InvalidRule(
                "unit_input_cost_micros cannot be negative".to_string(),
            ));
        }
        if self.unit_output_cost_micros < 0 {
            return Err(CreditError::InvalidRule(
                "unit_output_cost_micros cannot be negative".to_string(),
            ));
        }
        if self.margin_percent < 0 {
            return Err(CreditError::InvalidRule(
                "margin_percent cannot be negative".to_string(),
            ));
        }
        if self.min_credits < 0 {
            return Err(CreditError::InvalidRule(
                "min_credits cannot be negative".to_string(),
            ));
        }
        if let Some(limit) = self.max_credit_limit {
            if limit < self.min_credits {
                return Err(CreditError::InvalidRule(
                    "max_credit_limit cannot be less than min_credits".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Fluent builder for constructing and validating a [`PricingRule`].
#[derive(Debug, Clone, Default)]
pub struct PricingRuleBuilder {
    rule: PricingRule,
}

impl PricingRuleBuilder {
    /// Sets the unit input cost in micros (e.g. 100 for $0.00010).
    pub const fn unit_input_cost_micros(mut self, micros: i32) -> Self {
        self.rule.unit_input_cost_micros = micros;
        self
    }

    /// Sets the unit output cost in micros (e.g. 400 for $0.00040).
    pub const fn unit_output_cost_micros(mut self, micros: i32) -> Self {
        self.rule.unit_output_cost_micros = micros;
        self
    }

    /// Sets the profit margin percentage (e.g. 300 for +300% margin).
    pub const fn margin_percent(mut self, percent: i32) -> Self {
        self.rule.margin_percent = percent;
        self
    }

    /// Sets the conversion rate of micros per 1 credit (e.g. 1,000 micros = 1 credit).
    pub const fn micros_per_credit(mut self, micros: i32) -> Self {
        self.rule.micros_per_credit = micros;
        self
    }

    /// Sets the minimum credits charged per billable operation.
    pub const fn min_credits(mut self, min: i32) -> Self {
        self.rule.min_credits = min;
        self
    }

    /// Sets an optional ceiling limit for credits per single request.
    pub const fn max_credit_limit(mut self, limit: Option<i32>) -> Self {
        self.rule.max_credit_limit = limit;
        self
    }

    /// Validates and returns the configured [`PricingRule`].
    pub fn build(self) -> Result<PricingRule, CreditError> {
        self.rule.validate()?;
        Ok(self.rule)
    }
}
