use serde::{Deserialize, Serialize};

/// Quantities of metered units consumed during an operation.
///
/// Units are typically prompt/completion tokens, API call counts,
/// compute seconds, or characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Number of input units consumed (e.g., prompt tokens).
    pub input_units: i32,
    /// Number of output units consumed (e.g., completion tokens).
    pub output_units: i32,
}

impl Usage {
    /// Creates a new usage record with input and output units.
    pub const fn new(input_units: i32, output_units: i32) -> Self {
        Self {
            input_units,
            output_units,
        }
    }

    /// Creates a usage record where only input units are consumed.
    pub const fn input_only(input_units: i32) -> Self {
        Self {
            input_units,
            output_units: 0,
        }
    }

    /// Creates a usage record where only output units are consumed.
    pub const fn output_only(output_units: i32) -> Self {
        Self {
            input_units: 0,
            output_units,
        }
    }

    /// Total units consumed (input + output).
    pub const fn total_units(&self) -> i32 {
        self.input_units.saturating_add(self.output_units)
    }
}
