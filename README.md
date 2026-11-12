# Creditcraft

Deterministic integer-based credit calculation and token billing engine for metered APIs, AI models, and consumption-based SaaS workflows.

Developed by [Kreive Tech](https://kreive.com.br).

---

## Overview

Billing for LLM tokens or metered API units with floating-point numbers (`f32`/`f64`) introduces cumulative precision drift, rounding inconsistencies, and database friction.

`creditcraft` solves this by executing all calculations strictly on 32-bit signed integers (`i32`) through fixed-point arithmetic using **micros** ($1.00 = 1,000,000 micros). It applies integer ceiling division to ensure that fractional remainder costs always resolve to the next whole credit, protecting commercial margins.

## Key Features

- **Integer Fixed-Point Math (`i32`)**: Zero floating-point drift and direct compatibility with SQL `INTEGER` columns.
- **Provider-Agnostic**: Operates purely on units and rates. Works seamlessly with LLM tokens, compute seconds, characters, or custom meters.
- **Ceiling Division**: Automatically rounds up fractional amounts (`div_ceil`) to prevent revenue leakage.
- **Commercial Margins**: Native integer profit margin scaling (e.g., `+300%`).
- **Safety Floors & Ceilings**: Enforces minimum billable credits (`min_credits`) and optional request ceilings (`max_credit_limit`).
- **Atomic Balance Settlement**: Includes in-place balance deduction (`settle`) and pre-flight capacity verification (`check_capacity`).
- **Zero Heavy Dependencies**: Lightweight, fast, and fully serializable via Serde.

---

## Installation

Add `creditcraft` to your `Cargo.toml`:

```toml
[dependencies]
creditcraft = "0.1.0"
```

---

## Quick Start

### 1. Basic Credit Calculation

```rust
use creditcraft::{PricingRule, Usage, calculate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define pricing rule (costs in micros, margin in %)
    let rule = PricingRule::builder()
        .unit_input_cost_micros(100)       // $0.00010 per input unit
        .unit_output_cost_micros(400)      // $0.00040 per output unit
        .margin_percent(300)               // +300% profit margin (4x total)
        .micros_per_credit(1_000)          // 1 credit = 1,000 micros ($0.001)
        .min_credits(1)                    // Minimum 1 credit per operation
        .build()?;

    // 2. Specify units consumed (e.g., tokens)
    let usage = Usage::new(1_000, 500);
    let current_balance = 2_000;

    // 3. Compute cost and required credits
    let result = calculate(&rule, usage, current_balance)?;

    println!("Raw provider cost: {} micros", result.raw_cost_micros);
    println!("Commercial cost:   {} micros", result.commercial_cost_micros);
    println!("Credits to deduct: {}", result.credits_to_deduct);
    println!("Remaining balance: {:?}", result.remaining_balance);

    Ok(())
}
```

---

### 2. Atomic In-Place Balance Settlement

```rust
use creditcraft::{PricingRule, Usage, settle};

let rule = PricingRule::builder()
    .unit_input_cost_micros(100)
    .unit_output_cost_micros(400)
    .margin_percent(300)
    .micros_per_credit(1_000)
    .build()?;

let mut user_balance = 5_000;
let usage = Usage::new(1_500, 350);

// Deducts directly from user_balance if sufficient; returns CreditError::InsufficientBalance otherwise.
let outcome = settle(&rule, usage, &mut user_balance)?;

println!("Credits deducted: {}", outcome.credits_to_deduct);
println!("New user balance: {}", user_balance);
```

---

### 3. Pre-Flight Capacity Check

Before dispatching expensive requests to upstream AI providers, verify if the user's balance covers estimated usage:

```rust
use creditcraft::{PricingRule, Usage, check_capacity};

let rule = PricingRule::default();
let estimated_usage = Usage::new(2_000, 1_000);
let balance = 15;

if !check_capacity(&rule, estimated_usage, balance)? {
    println!("HTTP 402: Insufficient credits to initiate request.");
}
```

---

## Arithmetic Model

| Dimension | Representation | Example |
| :--- | :--- | :--- |
| **Currency Scaling** | Micros (`1 Currency Unit = 1,000,000 Micros`) | `$0.00010` = `100 micros` |
| **Tokens / Units** | Signed 32-bit Integer (`i32`) | `1,500` prompt tokens |
| **Profit Margin** | Integer Percentage (`i32`) | `300` (+300% / 4x commercial) |
| **Credit Conversion** | Micros per Credit (`i32`) | `1_000 micros` = 1 credit |
| **Rounding Strategy** | Integer Ceiling Division | `(a + b - 1) / b` |

---

## License

This project is licensed under the [MIT License](LICENSE).

Copyright (c) 2026 [Kreive Tech](https://kreive.com.br) - <contact@kreive.com.br>
