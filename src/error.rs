use thiserror::Error;

/// Errors produced by credit calculation and balance validation operations.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum CreditError {
    /// Attempted to execute an operation with insufficient credit balance.
    #[error("Insufficient credit balance: required {required}, available {available}")]
    InsufficientBalance {
        /// Credits required for the operation.
        required: i32,
        /// Current available credit balance.
        available: i32,
    },

    /// The calculated credits exceeded a configured ceiling or request budget limit.
    #[error("Credit limit exceeded: calculated {attempted}, maximum allowed {limit}")]
    CreditLimitExceeded {
        /// Maximum allowed credits configured in the rule.
        limit: i32,
        /// Calculated credits attempted to be charged.
        attempted: i32,
    },

    /// An arithmetic overflow occurred during pricing or unit scaling calculation.
    #[error("Arithmetic overflow occurred during credit calculation")]
    CalculationOverflow,

    /// A pricing rule configuration contains invalid parameters (e.g., non-positive divisor).
    #[error("Invalid pricing rule: {0}")]
    InvalidRule(String),
}
