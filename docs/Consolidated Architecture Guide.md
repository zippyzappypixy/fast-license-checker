# Solo Rust Architect: FAANG-Grade, Planet-Scale Development in Rust (with Cursor)

## 0. Executive Summary

Large engineering organizations (FAANG-scale) achieve quality through **process, headcount, and redundancy**: principal engineers, design committees, QA teams, SREs, AppSec, release managers. As a solo developer, you cannot replicate their organizational structure.

You must instead achieve comparable quality by combining:

1. **Architecture and Type System** (Rust as the "Chief Architect")
2. **Automation and Tooling** (your "Robot Team" for CI, security, QA)
3. **Agentic IDE (Cursor)** as your **implementation crew**, while you operate purely as an **architect and tech lead**.

This report is your **single source of truth** for building FAANG-quality, planet-scale systems in Rust as a solo dev, using Cursor as the coding agent.

***

## 1. Core Philosophy: How a Solo Dev Matches FAANG

### 1.1 The Asymmetry

- **FAANG Way**: Process, committees, multi-layer review, separation of roles.
- **Solo Rust Way**: Compiler-enforced architecture, strict type discipline, automated quality gates, and AI agents (Cursor) executing your architectural decisions.


### 1.2 The Three Pillars

1. **Architecture \& Design**
    - Hexagonal architecture, modular monolith, strict domain boundaries.
    - Docs-as-code: RFCs, ADRs, diagrams in the repo.
2. **Compiler \& Tooling as Enforcement**
    - Rust type system, ownership, clippy (pedantic), cargo-deny, property-based testing, observability tooling.
3. **AI Agent (Cursor) as Execution Engine**
    - You define requirements, architecture, and constraints.
    - Cursor + your `.cursorrules` implement, refactor, and test under your rules.

***

## 2. The 7 Core Tenets for Planet-Scale Rust

These are your non-negotiable "North Star" principles.

### Tenet 1: Security (Defense in Depth)

**Principle:** Make invalid states unrepresentable. Security is an architectural property, not a bolt-on feature.

**Tactics:**

- **Typestate Pattern**
Use distinct types for each state instead of string flags.

```rust
struct Unpaid;
struct Paid;

struct Order<State> {
    id: OrderId,
    state: PhantomData<State>,
    // ...
}

impl Order<Unpaid> {
    fn pay(self) -> Order<Paid> { /* ... */ }
}
```

You literally cannot call `ship()` on `Order<Unpaid>`.
- **Parse, Don't Validate**
    - Parse raw strings immediately into domain types:

```rust
struct Email(String); // constructor validates format
struct TickerSymbol(String);
```

    - If an `Email` exists, it is guaranteed valid.
- **Zero-Knowledge Storage**
Use the `secrecy` crate for secrets (keys, passwords) to:
    - Zero memory on drop.
    - Redact secrets from logs.


### Tenet 2: Performance (Zero-Cost Abstractions)

**Principle:** High performance is the absence of unnecessary work.

**Tactics:**

- Prefer stack allocation (structs, arrays) over heap allocations where possible.
- Use **NewType Optimization**:

```rust
#[repr(transparent)]
struct UserId(Uuid);
#[repr(transparent)]
struct Quantity(Decimal);
```

Type safety with zero runtime overhead.
- Use **criterion** for microbenchmarking and **pprof/flamegraph** for profiling hot paths.


### Tenet 3: Scalability (Shared-Nothing Architecture)

**Principle:** Components must scale independently; horizontal scaling is the default.

**Tactics:**

- **Hexagonal Architecture (Ports \& Adapters)**
    - Core: Pure domain logic with traits (ports), no IO crates.
    - Adapters: API, DB, and other IO that implement ports.
- **Stateless Application Tier**
    - All state in DB/Redis, not in API memory.
    - Enables running many instances across platforms (shuttle.rs, Fly.io, etc.).


### Tenet 4: Readability (Bus Factor = 1)

**Principle:** You must understand your own code after 6–12 months within minutes; otherwise, it's technical debt.

**Tactics:**

- **Explicitness Over Cleverness**: Avoid overly clever generics or macros hiding logic.
- **Docs-as-Code in `.rs`**:
Every public type and function gets `///` doc comments explaining **why**, not just **what**.
- **No Primitives in Core**:
In domain code, never pass `i32`, `String`, `Uuid` directly. Always wrap in NewTypes, e.g.,

```rust
struct Money(Decimal);
struct PortfolioId(Uuid);
```


### Tenet 5: Observability (The Eyes)

**Principle:** You cannot debug what you cannot see.

**Tactics:**

- Structured logging with `tracing`:

```rust
#[tracing::instrument(skip(self))]
async fn submit_order(&self, user_id: UserId, order: OrderDraft) -> Result<OrderId> {
    // ...
}
```

- **Correlation IDs**:
    - Generate a `RequestId` per incoming HTTP request.
    - Propagate through logs, DB operations, and workers.
- Use `tracing-subscriber` and optionally OpenTelemetry exporters.


### Tenet 6: Reliability (Resilience)

**Principle:** Resilience > MTBF. How gracefully do you handle inevitable failures?

**Tactics:**

- **Ban Panics**:
No `.unwrap()`, `.expect()`, `panic!` in production. Enforced by clippy.
- **Typed Errors**:
    - Domain/library-level: `thiserror` enums.
    - Application-level: `anyhow` for aggregation, but map back to typed errors at boundaries.
- **Graceful Degradation**:
    - If external quote API fails: return cached value or partial data, not HTTP 500.


### Tenet 7: Simplicity (Maintainability)

**Principle:** Complexity kills solo projects.

**Tactics:**

- **Boring Stack**:
Prefer stable, mainstream crates: `tokio`, `axum`/`actix-web`, `sqlx`, `serde`, `thiserror`, `tracing`.
- **Modular Monolith over Microservices**:
    - Single binary, multiple crates in a workspace.
    - Strong internal boundaries, single deployment unit.
- **Automated Linting \& Formatting**:
Let tools decide style, you focus on logic.

***

## 3. Architecture \& Repo Structure: The Modular Monolith

### 3.1 Complete Workspace Layout

```text
stock-anvesha/                          # ← REPOSITORY ROOT
├── .cursorrules                        # ← CURSOR RULES (AI SYSTEM PROMPT)
├── .gitignore
├── Cargo.toml                          # Workspace root definition
├── Cargo.lock
├── clippy.toml                         # Linter strict rules
├── deny.toml                           # Supply chain security policy
├── rustfmt.toml                        # Code formatting rules
├── README.md
│
├── .github/
│   ├── workflows/
│   │   └── ci.yml                      # Quality gate: fmt, clippy, deny, test
│   └── scripts/
│       └── check_primitives.sh         # Custom: ban raw primitives in core
│
├── .vscode/
│   └── settings.json                   # Cursor + rust-analyzer config
│
├── docs/
│   ├── input/                          # Business requirements (YOU WRITE THIS)
│   │   ├── template-requirements.txt   # Template for features
│   │   └── portfolio-tracking-requirements.txt
│   ├── adr/                            # Architectural Decision Records
│   │   └── 001-use-polars.md
│   ├── rfcs/                           # Request for Comments (generated by Cursor)
│   │   ├── template.md                 # RFC template
│   │   └── 0001-portfolio-tracking.md
│   ├── architecture.md                 # High-level system map
│   └── solo-rust-architect-plan.md     # This document
│
├── crates/
│   ├── core/                           # 🧠 DOMAIN LOGIC (Pure Rust)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── design_sketch.rs        # Global system skeleton (temporary)
│   │       ├── common/
│   │       │   ├── mod.rs
│   │       │   ├── result.rs           # Custom Result type
│   │       │   └── error.rs            # Domain error types
│   │       ├── portfolio/              # Feature module
│   │       │   ├── mod.rs
│   │       │   ├── design_sketch.rs    # Feature-specific skeleton
│   │       │   ├── types.rs            # NewTypes, domain models
│   │       │   ├── logic.rs            # Pure business functions
│   │       │   └── tests.rs            # Unit + property tests
│   │       ├── users/
│   │       │   ├── mod.rs
│   │       │   ├── types.rs
│   │       │   ├── logic.rs
│   │       │   └── tests.rs
│   │       └── ports/                  # Trait definitions (Ports)
│   │           ├── mod.rs
│   │           ├── portfolio_repository.rs
│   │           ├── price_service.rs
│   │           └── event_publisher.rs
│   │
│   ├── api/                            # 🔌 HTTP ADAPTER (Web Interface)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                 # Entry point (tracing init)
│   │       ├── lib.rs
│   │       ├── handlers/
│   │       │   ├── mod.rs
│   │       │   ├── portfolio.rs        # Portfolio HTTP endpoints
│   │       │   └── health.rs
│   │       ├── routes/
│   │       │   └── mod.rs              # Route definitions
│   │       ├── dto/                    # Data Transfer Objects
│   │       │   ├── mod.rs
│   │       │   ├── portfolio_dto.rs    # Request/Response models
│   │       │   └── error_response.rs
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── request_id.rs       # Correlation ID injection
│   │       │   ├── error_handling.rs
│   │       │   └── rate_limit.rs
│   │       └── config.rs               # Server configuration
│   │
│   ├── storage/                        # 💾 DATABASE ADAPTER (Persistence)
│   │   ├── Cargo.toml
│   │   ├── migrations/
│   │   │   ├── 001_init_schema.sql
│   │   │   └── 002_add_indexes.sql
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── db_config.rs            # Connection pool setup
│   │       ├── portfolio/
│   │       │   └── mod.rs              # Implements PortfolioRepository
│   │       └── error.rs                # Database-specific errors
│   │
│   └── infra/                          # ☁️ INFRASTRUCTURE (Optional)
│       ├── Cargo.toml
│       └── src/lib.rs
│
├── tests/                              # 🧪 INTEGRATION TESTS
│   ├── common/
│   │   └── mod.rs                      # Test fixtures
│   └── integration/
│       └── portfolio_flow.rs           # End-to-end flow tests
│
├── snapshots/                          # 📸 INSTA SNAPSHOTS
│   └── api__handlers__portfolio__tests__get_summary.snap
│
└── benches/                            # (Optional) Performance benchmarks
    └── portfolio_calculations.rs
```


### 3.2 Key Structural Principles

#### Separation of Concerns via Crates

1. **`core`**: Pure domain logic
    - No imports of `sqlx`, `axum`, `tokio::net`
    - Defines traits (ports), business rules, type safety
    - Testable in isolation
2. **`api`**: HTTP/web adapter
    - Parses HTTP requests into domain commands
    - Calls `core` service layer via traits
    - Returns JSON responses
    - Handles auth, rate limiting, CORS
3. **`storage`**: Persistence adapter
    - Implements `core` traits for actual I/O
    - Database migrations, connection pooling
    - No business logic
4. **`infra`** (optional): Infrastructure concerns
    - PaaS configuration (shuttle.rs, Fly.io)
    - Deployment specifics

#### Enforced Architecture via Cargo Workspace

The `Cargo.toml` workspace enforces boundaries:

```toml
[workspace]
members = ["crates/core", "crates/api", "crates/storage", "crates/infra"]

# core crate strictly forbids these in Cargo.toml
# [dependencies]
# sqlx = { ... }  # ❌ NOT ALLOWED
# axum = { ... }  # ❌ NOT ALLOWED
```


### 3.3 Workspace Root (`Cargo.toml`)

```toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    "crates/api",
    "crates/storage",
    "crates/infra"
]

# Synchronized dependencies across all crates
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "fmt"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1.33", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
secrecy = { version = "0.8", features = ["serde"] }

# Linting and testing tools
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```


***

## 4. Docs-as-Code: Requirements \& System Design

### 4.1 Philosophy

For planet-scale systems, documentation is **the first phase of coding**, not an afterthought. As a solo dev, docs:

- Protect you from scope creep.
- Preserve context and rationale.
- Make refactors safer.

All design artefacts live in Git as Markdown.

### 4.2 Business Requirements Document

**Location:** `docs/input/[feature-name]-requirements.txt`

This is what **you write** before involving Cursor. It captures domain expertise:

```text
FEATURE: Portfolio Position Tracking

BUSINESS CONTEXT:
Users want to track their stock holdings across multiple portfolios 
(personal, retirement, trading account). They need to see:
- Current positions (ticker, quantity, cost basis)
- Unrealized gains/losses
- Portfolio performance over time

USER PERSONAS:
1. Retail Investor: Tracks 5-10 positions, checks daily
2. Active Trader: 50+ positions, needs real-time updates
3. Long-term Holder: Quarterly reviews, wants tax lot tracking

CORE USE CASES:
1. Add new position (buy shares)
2. Close position (sell all shares)
3. Partial sell (reduce quantity)
4. View portfolio summary with P&L
5. Handle corporate actions (stock splits, dividends)

BUSINESS RULES & INVARIANTS:
- Quantity must always be positive (can't go negative)
- Cost basis cannot be zero (avoid division errors)
- Users can have max 10 portfolios
- Position uniqueness: (portfolio_id, ticker_symbol) must be unique
- Closed positions retain history (soft delete, not hard delete)

CONSTRAINTS:
- Must support 10,000+ concurrent users
- P&L calculations need <100ms latency
- Must handle market data delays gracefully (15-min delayed quotes OK)

EDGE CASES TO HANDLE:
1. User tries to sell more shares than they own
2. Duplicate buy orders submitted within 1 second
3. Market data API is down (how to show P&L?)
4. Stock gets delisted (how to handle?)
5. User uploads CSV with 10,000 rows
6. Transaction ID collision (UUID collision handling)
7. Timezone issues when timestamps are recorded

NON-GOALS (v1):
- No options/derivatives support
- No multi-currency (USD only for now)
- No tax optimization features
- No social/sharing features

TECHNICAL PREFERENCES:
- Use Postgres for transactional data
- Redis for caching current prices (15-min TTL)
- Consider using polars for bulk CSV imports
- Prefer async/await over blocking operations
```


### 4.3 RFC Template

**Location:** `docs/rfcs/template.md`

```markdown
# RFC NNNN: Feature Name

## Executive Summary

**Problem Statement:**  
One sentence describing the pain point.

**Proposed Solution:**  
High-level description of the proposed feature.

**Goals:**  
- Measurable goal 1 (e.g., "P95 latency < 50ms")
- Measurable goal 2

**Non-Goals:**  
- Explicit exclusion 1
- Explicit exclusion 2

## User Stories

### Happy Path
As a [persona], I want to [action], so that [benefit].

Steps:
1. User does X
2. System validates
3. System returns Y

### Sad Path 1: [Edge Case Name]
As a [persona], when [condition], I want [behavior], so that [outcome].

Example: As a trader, when I try to sell more shares than I own, 
I want to see a validation error, so that I don't accidentally create an invalid position.

[Repeat for 3-5 edge cases]

## System Design

### Sequence Diagram

[Mermaid diagram here]

### Component Architecture

[Mermaid diagram here]

## Architectural Decision Records

| Decision | Context | Trade-offs |
|----------|---------|-----------|
| Use PostgreSQL for positions | ACID compliance needed for correctness | Can't use NoSQL |
| Use polars for CSV parsing | Column-based ops for performance | 15s extra compile time |
| Redis for price cache | Reduce external API calls | Cache invalidation complexity |

## Data Models (Skeleton)

```rust
#[repr(transparent)]
pub struct PortfolioId(Uuid);

#[repr(transparent)]
pub struct PositionId(Uuid);

pub struct Quantity(Decimal);
impl Quantity {
    pub fn new(value: Decimal) -> Result<Self, ValidationError> { ... }
}

pub struct Position {
    pub id: PositionId,
    pub portfolio_id: PortfolioId,
    pub ticker: TickerSymbol,
    pub quantity: Quantity,
    pub cost_basis: Money,
}
```


## Implementation Phases

1. **Phase 1**: Core domain types and validation
2. **Phase 2**: HTTP API endpoints
3. **Phase 3**: Database layer
4. **Phase 4**: Bulk CSV import
```

### 4.4 Skeleton Sketch

Before production code, create a **design sketch** that compiles to validate your architecture.

#### Option 1: Global System Sketch (RFC Phase)

**Location:** `crates/core/src/design_sketch.rs`

Use during the **RFC/requirements phase** when sketching cross-cutting concerns:

```rust
// crates/core/src/design_sketch.rs

//! Architectural skeleton for the Stock Anvesha system.
//! This file exists to validate that our RFC design actually compiles.
//! Once implementation begins, types migrate to their proper modules.
//! Run: cargo check
//! Once types are implemented, delete this file.

use uuid::Uuid;
use std::marker::PhantomData;

/// Job status shared across API and Worker components
pub enum JobStatus {
    Queued,
    Processing { progress_percent: u8 },
    Completed,
    Failed(String),
}

/// Core ingestion contract
#[async_trait::async_trait]
pub trait IngestionService {
    /// Accepts a file stream and returns a tracking ID.
    /// Async processing to keep HTTP connection short-lived.
    async fn submit_job(
        &self,
        user_id: Uuid,
        file: ByteStream
    ) -> Result<Uuid, IngestionError>;
}

pub struct IngestionError;
pub struct ByteStream;
```

Reference in `crates/core/src/lib.rs`:

```rust
pub mod design_sketch;  // Remove once migrated
pub mod common;
pub mod portfolio;
pub mod ports;
```


#### Option 2: Feature-Specific Skeleton

**Location:** `crates/core/src/portfolio/design_sketch.rs`

Use when exploring a specific feature domain:

```rust
// crates/core/src/portfolio/design_sketch.rs

//! Design sketch for portfolio tracking feature (RFC-0001)
//! Validates types and traits compile before full implementation.

use uuid::Uuid;
use rust_decimal::Decimal;

/// Portfolio identifier (NewType with zero-cost abstraction)
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortfolioId(Uuid);

/// Position identifier
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PositionId(Uuid);

/// Non-negative quantity with validation
#[derive(Debug, Clone, Copy)]
pub struct Quantity(Decimal);

#[derive(Debug, Clone)]
pub enum QuantityError {
    Negative,
    TooLarge,
}

impl Quantity {
    pub fn new(value: Decimal) -> Result<Self, QuantityError> {
        if value < Decimal::ZERO {
            return Err(QuantityError::Negative);
        }
        if value > Decimal::from(1_000_000_000) {
            return Err(QuantityError::TooLarge);
        }
        Ok(Self(value))
    }
}

/// Portfolio and its positions
pub struct Portfolio {
    pub id: PortfolioId,
    pub owner_id: Uuid,
    pub positions: Vec<Position>,
}

pub struct Position {
    pub id: PositionId,
    pub portfolio_id: PortfolioId,
    pub ticker: String,
    pub quantity: Quantity,
    pub cost_basis: Decimal,
}

/// The core service port
#[async_trait::async_trait]
pub trait PortfolioRepository {
    async fn save_position(&self, pos: Position) -> Result<(), RepositoryError>;
    async fn get_position(&self, id: PositionId) -> Result<Option<Position>, RepositoryError>;
    async fn list_positions(&self, portfolio_id: PortfolioId) -> Result<Vec<Position>, RepositoryError>;
}

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    DatabaseError(String),
    ConstraintViolation,
}
```

Run `cargo check` from `crates/core` to validate the design compiles.

***

## 5. The Robot Team: Toolchain \& Automation

### 5.1 Static Analysis: clippy (The Strict Senior Engineer)

**Config:** `clippy.toml` at repository root

Enforces reliability, correctness, and code quality without human debate.

```toml
# clippy.toml - Place in repository root

[lints.clippy]

# Tenet 6: Reliability - Ban crashes
unwrap_used = "deny"           # No .unwrap() - forces proper error handling
expect_used = "deny"           # No .expect() - requires explicit error context
panic = "deny"                 # No panic!() macros in business logic
indexing_slicing = "deny"      # No arr[^0] - prevents panics on empty collections

# Tenet 2: Correctness - Prevent silent bugs
arithmetic_side_effects = "deny"  # Forces checked_add/checked_mul for overflow safety
float_arithmetic = "warn"         # Warns on floating-point math (precision issues)

# Tenet 7: Simplicity - Clean code
wildcard_imports = "deny"      # No `use crate::*;` - keep dependencies explicit
shadow_unrelated = "deny"      # Prevent accidental variable shadowing
enum_glob_use = "deny"         # Don't glob import enum variants
```

**Run:**

```bash
cargo clippy --workspace -- -D warnings
```


### 5.2 Supply Chain Defense: cargo-deny (Security Team)

**Config:** `deny.toml` at repository root

Automatically scans dependencies for vulnerabilities and legal risks.

```toml
# deny.toml - Place in repository root

[advisories]
# Fail immediately if any dependency has a known CVE
vulnerability = "deny"
notice = "warn"
ignore = []  # Explicit exceptions only in emergencies

[licenses]
# Strictly limit licenses to those safe for your business
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "MPL-2.0",
]
# Strictly ban viral licenses that require source release
copyleft = "deny"
allow-copyleft = []

[bans]
# Warn if multiple versions of same crate linked (binary bloat)
multiple-versions = "warn"
# List any crates to ban outright
deny = []
skip = []
skip-tree = []
```

**Run:**

```bash
cargo deny check
```


### 5.3 Testing Stack: The QA Department

Build a comprehensive testing pyramid:

#### Unit Tests (Core logic)

```rust
// crates/core/src/portfolio/tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantity_rejects_negative() {
        assert!(Quantity::new(Decimal::from(-1)).is_err());
    }

    #[test]
    fn quantity_accepts_positive() {
        assert!(Quantity::new(Decimal::from(100)).is_ok());
    }
}
```


#### Property-Based Tests (proptest)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_quantity_never_panics(value in -1000.0f64..1_000_000.0f64) {
        let _ = Quantity::new(Decimal::from_f64(value).unwrap_or_default());
    }
}
```


#### Snapshot Testing (insta)

```rust
#[test]
fn portfolio_json_snapshot() {
    let portfolio = create_test_portfolio();
    insta::assert_json_snapshot!(portfolio);
}
```


#### Integration Tests

```bash
# tests/integration/portfolio_flow.rs

#[tokio::test]
async fn test_full_portfolio_workflow() {
    // API -> Core -> DB flow
}
```


#### Concurrency Testing (loom)

```rust
#[test]
fn concurrent_position_updates_safe() {
    loom::model(|| {
        // Verify no data races under all possible thread schedules
    });
}
```


#### Additional Tools

- **`cargo-nextest`**: Faster test runner (2-3x improvement)

```bash
cargo nextest run --workspace
```

- **`cargo-mutants`**: Mutation testing to verify test quality

```bash
cargo mutants -j 4
```

- **`cargo-llvm-cov`**: Code coverage reporting

```bash
cargo llvm-cov --workspace --html
```


### 5.4 Observability: `tracing` (NOC Dashboard)

Replace `println!` with structured logging.

#### Initialization (`crates/api/src/main.rs`)

```rust
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() {
    // Initialize tracing first—before any other code runs
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Now the app runs with full instrumentation
    api::run().await;
}
```


#### Auto-Instrumentation

```rust
// crates/core/src/services/portfolio_service.rs

#[tracing::instrument(skip(self))]
pub async fn add_position(
    &self,
    user_id: UserId,
    ticker: TickerSymbol,
    quantity: Quantity,
) -> Result<PositionId> {
    // Entry/exit automatically logged with arguments, execution time
    let position_id = PositionId::new();
    
    // Nested spans create full context
    span!("validate_position").in_scope(|| {
        self.validate_ticker(&ticker)?;
        Ok::<_, Error>(())
    })?;

    Ok(position_id)
}
```


#### Correlation IDs (Request Tracking)

```rust
// crates/api/src/middleware/request_id.rs

#[tracing::instrument(skip(next))]
pub async fn request_id_middleware(
    req: HttpRequest,
    next: Next,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    
    // Every log from this request carries the correlation ID
    tracing::Span::current().record("request_id", request_id.to_string());
    
    next.call(req).await
}
```


### 5.5 Performance Profiling

#### Benchmarking (criterion)

```rust
// benches/portfolio_calculations.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_calculate_pnl(c: &mut Criterion) {
    c.bench_function("calculate_pnl_1000_positions", |b| {
        b.iter(|| {
            calculate_unrealized_pnl(
                black_box(Quantity::new(Decimal::from(100)).unwrap()),
                black_box(Money::new(Decimal::from(150)).unwrap()),
            )
        });
    });
}

criterion_group!(benches, bench_calculate_pnl);
criterion_main!(benches);
```

Run: `cargo bench`

#### Profiling (pprof / flamegraph)

```bash
cargo install flamegraph
cargo flamegraph --bin api -- --profile=cpu
```


***

## 6. CI/CD \& Quality Gates

### 6.1 GitHub Actions Workflow

**File:** `.github/workflows/ci.yml`

This is your **automated quality gate**—the build fails before you merge bad code.

```yaml
name: "Planet Scale CI"

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  quality-gate:
    name: Quality Gate
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Install Rust Toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-deny
        uses: EmbarkStudios/cargo-deny-action@v1

      # Step 1: Security Audit
      - name: Security Audit (cargo-deny)
        run: cargo deny check

      # Step 2: Code Formatting
      - name: Check Code Formatting
        run: cargo fmt --all -- --check

      # Step 3: Strict Linting
      - name: Clippy (Strict Checks)
        run: cargo clippy --workspace --all-targets -- -D warnings

      # Step 4: Custom Architecture Check
      - name: Verify Core Domain Cleanliness
        run: |
          chmod +x .github/scripts/check_primitives.sh
          ./.github/scripts/check_primitives.sh

      # Step 5: Run All Tests
      - name: Test Suite
        run: cargo test --workspace

      # Step 6: Code Coverage (Optional)
      - name: Generate Coverage Report
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov --workspace --lcov --output-path lcov.info

      # Step 7: Upload Coverage (Optional)
      - name: Upload Coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./lcov.info
```


### 6.2 Pre-Commit Hook (Local)

Enforce checks before you commit:

**File:** `.git/hooks/pre-commit` (create via script)

```bash
#!/bin/bash

echo "Running pre-commit quality checks..."

# Check formatting
cargo fmt -- --check
if [ $? -ne 0 ]; then
  echo "❌ Code not formatted. Run: cargo fmt"
  exit 1
fi

# Run clippy
cargo clippy --workspace -- -D warnings
if [ $? -ne 0 ]; then
  echo "❌ Clippy errors found."
  exit 1
fi

# Run tests
cargo test --workspace
if [ $? -ne 0 ]; then
  echo "❌ Tests failed."
  exit 1
fi

echo "✅ Pre-commit checks passed."
exit 0
```

Install with:

```bash
chmod +x .git/hooks/pre-commit
```

Or use the `pre-commit` framework (Python-based):

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt
        language: system
        types: [rust]
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --workspace -- -D warnings
        language: system
        types: [rust]
```


### 6.3 Custom Script: No-Primitives Check

**File:** `.github/scripts/check_primitives.sh`

Enforces NewType discipline in the core domain.

```bash
#!/bin/bash

# Fail if fn signatures in crates/core use raw primitives
# This enforces Tenet 4: No raw primitives in domain

echo "Checking crates/core for raw primitive usage in function signatures..."

PRIMITIVES=("i32" "i64" "u32" "u64" "f32" "f64" "String" "usize")

for PRIM in "${PRIMITIVES[@]}"; do
  if grep -r "pub fn.*: *$PRIM\|fn.*: *$PRIM" crates/core/src --include="*.rs" | grep -v "test\|doc"; then
    echo "❌ Error: Raw $PRIM detected in Core Domain function signature!"
    echo "   Wrap it in a NewType struct. Example:"
    echo "   #[repr(transparent)]"
    echo "   struct UserId(Uuid);"
    exit 1
  fi
done

echo "✅ Core Domain cleanliness check passed."
exit 0
```

Run as part of CI:

```yaml
- name: Check Primitives in Core
  run: chmod +x .github/scripts/check_primitives.sh && ./.github/scripts/check_primitives.sh
```


***

## 7. Cursor as Your FAANG-Grade Implementation Team

You act as **architect and tech lead**; Cursor acts as a **multi-agent implementation team**.

### 7.1 `.cursorrules`: Encoding Your Tenets

Create `.cursorrules` at repository root. This is your **system prompt** that tells Cursor how to behave.

**File:** `.cursorrules`

```markdown
# Project: Stock Anvesha - FAANG-Grade Planet-Scale Rust
# Role: You are a senior Rust engineer on a FAANG infrastructure team

## Core Principles (The 7 Tenets)

1. **Security (Defense in Depth)**
   - Make invalid states unrepresentable
   - Use Typestate pattern for state machines
   - Parse, don't validate: wrap all primitives immediately
   - Use secrecy::Secret<T> for sensitive data

2. **Performance (Zero-Cost Abstractions)**
   - All NewTypes use #[repr(transparent)] - zero runtime overhead
   - Prefer stack allocation over heap
   - Profile before optimizing; use criterion for benchmarks

3. **Scalability (Shared-Nothing)**
   - Core crate must never import sqlx, axum, tokio::net
   - All state lives in DB/Redis, not in-process
   - Design for horizontal scaling

4. **Readability (Bus Factor = 1)**
   - Every public fn/struct must have /// doc comments
   - Explain WHY, not just WHAT
   - No magic; explicit > clever

5. **Observability (The Eyes)**
   - Use #[tracing::instrument] on all service functions
   - All errors must include context
   - Structured logging via tracing subscriber (JSON)

6. **Reliability (Resilience)**
   - ZERO .unwrap() or .expect() in production code
   - All errors must be typed with thiserror
   - Never panic—return Result instead
   - This is enforced by clippy and will cause CI to fail

7. **Simplicity (Maintainability)**
   - Prefer stable, boring crates: tokio, axum, sqlx, serde, tracing
   - Modular monolith (single binary, multiple crates)
   - Let clippy and rustfmt decide style disputes

## Architecture Constraints

### Core Crate (`crates/core/src/`)
- **Allowed imports**: serde, thiserror, uuid, decimal, chrono, async-trait
- **FORBIDDEN imports**: sqlx, axum, tokio::net, actix-web, actix, tonic
- Purpose: Pure business logic, type safety, trait definitions
- No I/O operations

### API Crate (`crates/api/src/`)
- **Allowed imports**: axum, tokio, serde, tracing, thiserror, core (local)
- Purpose: HTTP handlers, routing, DTOs, validation
- Converts HTTP requests → domain commands
- Calls core service layer via traits

### Storage Crate (`crates/storage/src/`)
- **Allowed imports**: sqlx, redis, serde, core (local)
- Purpose: Implements core traits with actual DB/external service calls
- Database migrations and schema
- No business logic

## Code Style & Patterns

### NewTypes (Zero-Cost Wrappers)
Always wrap primitives with NewTypes:
```rust
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct UserId(Uuid);

#[repr(transparent)]
struct PortfolioId(Uuid);

struct Quantity(Decimal);  // Non-Copy: needs validation in constructor

impl Quantity {
    pub fn new(value: Decimal) -> Result<Self, QuantityError> {
        if value <= Decimal::ZERO {
            return Err(QuantityError::MustBePositive);
        }
        Ok(Self(value))
    }
}
```


### Error Handling

Use `thiserror` for typed errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum PortfolioError {
    #[error("Position not found: {0}")]
    NotFound(PositionId),
    
    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

Result type should generally NOT use String errors in core logic.

### Instrumentation

Add tracing to service functions:

```rust
#[tracing::instrument(skip(self, repo))]
pub async fn create_position(
    &self,
    user_id: UserId,
    ticker: TickerSymbol,
    quantity: Quantity,
    repo: &dyn PortfolioRepository,
) -> Result<PositionId, PortfolioError> {
    tracing::info!(user_id = %user_id, ticker = %ticker, "Creating new position");
    // ...
}
```


### Pattern Matching (Exhaustive)

Never use catch-all `_` in domain logic:

```rust
// ❌ BAD
match result {
    Ok(v) => process(v),
    _ => log_error(),  // Hides bugs!
}

// ✅ GOOD
match result {
    Ok(v) => process(v),
    Err(PortfolioError::NotFound(id)) => { /* specific handling */ },
    Err(PortfolioError::InvalidQuantity(msg)) => { /* specific handling */ },
    Err(PortfolioError::Database(err)) => { /* specific handling */ },
}
```


## Testing Requirements

### Unit Tests (Core Logic)

- Every public function must have at least one unit test
- Test both happy path and error cases
- Use property-based testing (proptest) for invariants
- Aim for >= 80% code coverage


### Integration Tests

- End-to-end flows in `/tests` directory
- Test API → Core → DB path
- Use test fixtures for database setup
- Verify error responses with insta snapshots


### Test Execution

```bash
cargo test --workspace
cargo llvm-cov --workspace  # Coverage
cargo mutants -j 4          # Mutation testing
```


## Forbidden Patterns (Will Cause CI Failure)

❌ `.unwrap()` in production code
❌ `.expect()` in production code
❌ `panic!()` macros
❌ Array indexing `arr[i]` (use `.get()`)
❌ Floating-point arithmetic without explicit checks
❌ `use crate::*;` (explicit imports)
❌ Variable shadowing
❌ Raw primitives in function signatures (core crate)
❌ `println!()` - use `tracing` macros

These are enforced by clippy with `-D warnings` and will break your build.

## When Generating Code

### Before Implementation:

1. Ask for type and trait definitions first (design sketch)
2. Run `cargo check` to validate design compiles
3. Ask for tests before business logic
4. Review the skeleton for architectural correctness

### During Implementation:

1. Start with error types (thiserror enums)
2. Define NewTypes for all domain concepts
3. Implement validation in constructors
4. Write unit tests with proptest
5. Implement service logic (pure functions where possible)
6. Add tracing instrumentation
7. Document all public APIs

### Code Review Checklist:

- [ ] No unwrap/expect/panic (except tests)
- [ ] All errors typed with thiserror
- [ ] All public functions documented
- [ ] All public types are NewTypes (no raw primitives)
- [ ] Service functions have \#[tracing::instrument]
- [ ] Tests exist for all public functions
- [ ] clippy passes with -D warnings


## File Organization

- Core business logic: `/crates/core/src/[feature]/*.rs`
- HTTP handlers: `/crates/api/src/handlers/[feature].rs`
- Database implementations: `/crates/storage/src/[feature]/*.rs`
- Traits (ports): `/crates/core/src/ports/[feature].rs`
- Tests: `/tests/integration/[feature].rs` or inline in `.rs` files
- Snapshots: `/snapshots/[test_path].snap` (auto-generated by insta)


## When Refactoring

1. Ensure refactoring doesn't introduce unwrap/expect/panic
2. Run full test suite: `cargo test --workspace`
3. Run clippy: `cargo clippy --workspace -- -D warnings`
4. Show diffs before applying changes
5. Do not remove or rename public APIs without consensus

## Reference Docs

- RFC Location: `docs/rfcs/[feature].md`
- ADR Location: `docs/adr/[decision].md`
- Architecture: `docs/architecture.md`
- Requirements: `docs/input/[feature]-requirements.txt`


## Key Principles

- Rust compiler is your partner, not your enemy
- Let clippy guide you—it knows better than your ego
- Explicit > Implicit > Clever
- Tests are not optional—they are part of the design
- Every line of code must have a reason; if not, delete it

```

### 7.2 Cursor Workflow: Architect-First Development

Your role is **architect**. Cursor is your implementation crew. This workflow shows the exact steps.

#### Step 1: Business Requirements (You Do This)

Create the plain-text requirements file capturing domain knowledge:

**File:** `docs/input/portfolio-tracking-requirements.txt`

```text
FEATURE: Portfolio Position Tracking

BUSINESS CONTEXT:
Retail and professional investors need to track stock positions across multiple 
portfolios. They want real-time P&L calculation and support for corporate actions.

USER PERSONAS:
1. Retail Investor: 5-10 positions, daily checks
2. Active Trader: 50+ positions, needs sub-100ms latency
3. Long-term Holder: Quarterly reviews, tax lot tracking

CORE USE CASES:
1. Add new position (buy)
2. Close position (sell)
3. Partial sell
4. View portfolio summary with P&L
5. Handle stock splits

BUSINESS RULES:
- Quantity must be positive
- Cost basis must be > 0
- Max 10 portfolios per user
- Unique constraint: (portfolio_id, ticker)

CONSTRAINTS:
- 10,000+ concurrent users
- P&L latency < 100ms
- Handle 15-min delayed quotes

EDGE CASES:
1. Sell more than owned
2. Duplicate orders (1-second window)
3. Market data API down
4. Stock delisted
5. Bulk CSV import (10k rows)

NON-GOALS (v1):
- No options
- USD only
- No tax optimization
- No social features

TECH PREFERENCES:
- Postgres (transactional)
- Redis (price cache, 15-min TTL)
- Polars for CSV parsing
```


#### Step 2: RFC Generation (Cursor Agent)

Now use Cursor Agent (Cmd/Ctrl+I or Ctrl+K depending on Cursor version):

```
@Files/docs/input/portfolio-tracking-requirements.txt
@Files/docs/rfcs/template.md

Create RFC at docs/rfcs/0001-portfolio-tracking.md

Transform the business requirements into a structured RFC:

1. Executive Summary
   - Extract problem and solution
   - Convert business rules into measurable goals (latency, scale, correctness)

2. User Stories
   - "As a [persona], I want to [action], so that [benefit]"
   - Happy path: Add Position workflow
   - Sad paths: 5 edge cases from requirements

3. Sequence Diagram (Mermaid)
   - Show: API -> Core -> DB -> Response
   - Include validation failure path
   - Show async error handling

4. Component Architecture Diagram (Mermaid)
   - Core (domain logic)
   - API (HTTP adapter)
   - Storage (DB adapter)
   - Ports (traits connecting them)

5. ADR Section
   - Why PostgreSQL over NoSQL (ACID for correctness)
   - Why Polars over csv crate (column-based perf)
   - Why Redis for price cache (API rate limits)

Use business terminology. Flag any ambiguities for architect review.
```

**You review the generated RFC and iterate:**

```
@Files/docs/rfcs/0001-portfolio-tracking.md

Changes needed:
1. Sequence diagram: add 5-second timeout on price API call
2. ADR: explain Decimal type choice over f64 (financial precision)
3. Add edge case: handling stock split -> all positions need adjustment factor
4. Component diagram: add EventBus for notifying users of batch import completion
```

Cursor updates RFC iteratively until you approve.

#### Step 3: Domain Types \& Traits (Cursor Composer)

Once RFC is locked, use Cursor Composer (Cmd/Ctrl+Shift+I) to generate skeleton:

```
@Files/docs/rfcs/0001-portfolio-tracking.md
@Folders/crates/core

Create design sketch in crates/core/src/portfolio/design_sketch.rs

Based on RFC-0001 business rules:

NewTypes (all #[repr(transparent)]):
- PortfolioId(Uuid)
- PositionId(Uuid)
- TickerSymbol(String) - validation: 1-5 chars, uppercase, alphanumeric
- Quantity(Decimal) - validation: must be > 0
- Money(Decimal) - validation: must be > 0 for financial correctness

Position struct with fields from RFC and custom constructor returning Result<Position, ValidationError>

Port traits:
- PortfolioRepository: async CRUD for positions
- PriceService: fetch current prices with timeout handling
- EventPublisher: publish events for bulk imports

Add comprehensive doc comments referencing RFC sections.
Run: cargo check to validate design compiles.
```

**Your review:**

- Does the type system match business rules?
- Are constraints (Decimal, NewTypes) enforced?
- Can you understand the architecture from the skeleton?

```
@Files/crates/core/src/portfolio/design_sketch.rs

Looks good. Before we proceed to full implementation:
1. Add conversion from string to TickerSymbol in constructor (validation)
2. Show an example of how Quantity::new() would be called
3. Add #[derive(Debug)] to all types for easier debugging
```


#### Step 4: Test-First Implementation Loop

Now implement with **tests leading logic**:

```
@Files/crates/core/src/portfolio/design_sketch.rs
@Files/crates/core/src/common/error.rs

Create unit tests in crates/core/src/portfolio/tests.rs

Write tests for each NewType validation:
1. TickerSymbol rejects lowercase, rejects > 5 chars
2. Quantity rejects 0 and negative
3. Money rejects <= 0

Use proptest for invariants:
4. For any Quantity Q and Money M, total_value = Q * M never panics
5. For any valid Portfolio, unrealized_pnl is always finite

Run: cargo test portfolio::tests --lib

All tests should FAIL initially (TDD).
```

Cursor generates comprehensive tests covering happy + sad paths.

```
Implement business logic in crates/core/src/portfolio/logic.rs

Write pure functions:
- calculate_unrealized_pnl(quantity: Quantity, cost: Money, current_price: Money) -> Result<PnL>
- validate_position(pos: Position) -> Result<(), ValidationError>
- apply_stock_split(pos: Position, split_ratio: Decimal) -> Result<Position>

Requirements:
- Use checked arithmetic (no panics)
- Return typed errors (no String errors)
- Add #[tracing::instrument] to functions
- Tests should now pass

Run: cargo test portfolio::logic --lib
```

Cursor implements logic to pass the tests you've defined.

#### Step 5: Integration Layer (API \& Storage)

Split work across API and Storage crates:

```
@Files/crates/core/src/portfolio/types.rs
@Files/crates/core/src/ports/portfolio_repository.rs

Create HTTP handlers in crates/api/src/handlers/portfolio.rs

POST /portfolios/{portfolio_id}/positions - Add position
- Parse JSON request into DTO
- Map DTO -> domain Position (via core service)
- Call PortfolioRepository trait
- Return JSON response

GET /portfolios/{portfolio_id}/summary - Get P&L
- Fetch positions from repository
- Calculate P&L via core logic
- Return JSON with unrealized gains

Requirements:
- Use axum extractors for path/body
- Add error handling middleware
- Add RequestId to response header
- Serialize errors as JSON

Use @Files/crates/api/src/dto/portfolio_dto.rs for request/response shapes.
```

```
@Files/crates/core/src/ports/portfolio_repository.rs

Implement PortfolioRepository trait in crates/storage/src/portfolio/mod.rs

struct SqlxPortfolioRepository { pool: PgPool }

Methods:
- save_position: INSERT into positions table
- get_position: SELECT by ID
- list_positions: SELECT by portfolio_id
- update_position: UPDATE by ID

Requirements:
- Use sqlx for parameterized queries (SQL injection prevention)
- Map Err(sqlx::Error) -> Err(RepositoryError)
- Handle unique constraint violations gracefully
- Use transactions for multi-operation consistency

Add database migration in crates/storage/migrations/001_init.sql
```


#### Step 6: Automated Quality Checks

Ask Cursor to run your "Robot Team":

```
@Folders/crates

Run full quality pipeline:

1. cargo fmt --check
   If formatting needed: cargo fmt

2. cargo clippy --workspace -- -D warnings
   For each clippy error:
   - unwrap/expect: replace with ? or match
   - indexing_slicing: replace arr[i] with arr.get(i)?
   - show before/after diff

3. cargo deny check
   If vulnerabilities found: list them and explain risk

4. cargo test --workspace
   If tests fail: show failures and suggest fixes

5. cargo llvm-cov --workspace
   If coverage < 80%: identify untested branches and suggest tests

Apply fixes iteratively. After each fix, re-run to confirm.
```

Your review each diff before applying.

#### Step 7: Multi-Agent Parallel Work (Cursor 2.0+)

Advanced: Use Cursor 2.0's multi-agent capability to work on multiple features in parallel:

```
@Folders/crates/core
Agent 1: Implement user authentication domain in crates/core/src/users

@Folders/crates/api
Agent 2: Build additional API endpoints for portfolio management

@Folders/crates/storage
Agent 3: Implement database layer for users and portfolios

Each agent works in isolated git worktree. You review each agent's work:
- Agent 1 output: /path/to/users-branch
- Agent 2 output: /path/to/api-branch
- Agent 3 output: /path/to/storage-branch

Merge best implementations into main branch.
```


### 7.3 Context Engineering: Controlling Cursor's Focus

Use precise `@`-mentions to control what Cursor sees:

**Good (Focused Context)**:

```
@Files/crates/core/src/portfolio/types.rs
@Files/crates/core/src/ports/portfolio_repository.rs

Refactor Position struct to include acquisition_date field
```

**Bad (Too Much Context)**:

```
@Folders/crates

Add acquisition date somewhere
```

**Why**: Focused context → cleaner diffs → fewer mistakes.

### 7.4 Staged Autonomy Pattern

For complex refactors, use 3-step approval:

**Step 1: Plan**

```
Describe the refactoring plan for adding transaction history tracking.
List all files that need modification and the changes to each.
```

Cursor shows plan. You approve modifications.

**Step 2: Preview**

```
Generate diffs for each file without applying changes.
Show before/after for: types.rs, repository.rs, migrations.sql
```

You review diffs in side-by-side view.

**Step 3: Apply Selectively**

```
Apply changes to files 1-3.
Hold off on file 4 (database migration) pending schema review.
```

You apply only approved changes.

### 7.5 Error-Driven Development

Let clippy and tests catch issues first:

```
Run: cargo clippy --workspace

Clippy errors appear (unwrap_used, indexing_slicing, etc.)

Fix all clippy errors using these rules:
- unwrap_used: Replace with ? operator or match
- indexing_slicing: Replace arr[i] with arr.get(i)?
- arithmetic_side_effects: Use checked_add, checked_mul
- Show each fix with before/after comparison
- Explain why each fix is safer

Run: cargo clippy again to confirm all pass
```


***

## 8. End-to-End Action Plan

### Phase A: Bootstrap (Week 1)

**Day 1: Repository Structure**

- [ ] Initialize workspace: `cargo new stock-anvesha`
- [ ] Create directory structure: `crates/core`, `crates/api`, `crates/storage`, `crates/infra`
- [ ] Create `docs/input`, `docs/rfcs`, `docs/adr` directories

**Day 2: Configuration Files**

- [ ] Add `clippy.toml` with strict rules (Section 5.1)
- [ ] Add `deny.toml` with security policy (Section 5.2)
- [ ] Add `rustfmt.toml` for formatting consistency
- [ ] Add `.cursorrules` encoding your 7 tenets (Section 7.1)
- [ ] Add `.vscode/settings.json` for Cursor + rust-analyzer integration

**Day 3: CI/CD Infrastructure**

- [ ] Create `.github/workflows/ci.yml` (Section 6.1)
- [ ] Create `.github/scripts/check_primitives.sh` (Section 6.3)
- [ ] Add pre-commit hook for local checks

**Day 4: Tooling Setup**

- [ ] Install and test: `cargo clippy`, `cargo fmt`, `cargo-deny`, `cargo-nextest`
- [ ] Configure coverage: `cargo-llvm-cov`
- [ ] Test full CI pipeline locally

**Day 5: Documentation**

- [ ] Create `docs/rfcs/template.md` (Section 4.3)
- [ ] Create `docs/input/template-requirements.txt` (Section 4.2)
- [ ] Copy this document to `docs/solo-rust-architect-plan.md`


### Phase B: First Feature - RFC \& Design (Week 2)

**Day 1–2: Business Analysis (Architect)**

- [ ] Write business requirements in `docs/input/[feature]-requirements.txt`
- [ ] Define user personas, use cases, business rules, edge cases

**Day 3–4: RFC Generation \& Iteration (with Cursor)**

- [ ] Use Cursor Agent to generate RFC in `docs/rfcs/0001-[feature].md`
- [ ] Review and iterate on RFC until locked
- [ ] Mermaid diagrams for sequences and components

**Day 5: Design Sketch \& Type System**

- [ ] Use Cursor Composer to create `crates/core/src/[feature]/design_sketch.rs`
- [ ] Validate with `cargo check`
- [ ] Review architecture with RFC


### Phase C: Implementation - Core Layer (Week 3)

**Day 1: Test-First Development**

- [ ] Ask Cursor to generate comprehensive tests for NewTypes
- [ ] Create `crates/core/src/[feature]/tests.rs` with unit + proptest
- [ ] Tests should fail initially (TDD discipline)

**Day 2–3: Domain Logic Implementation**

- [ ] Implement types in `crates/core/src/[feature]/types.rs`
- [ ] Implement trait definitions in `crates/core/src/ports/`
- [ ] Implement service logic in `crates/core/src/[feature]/logic.rs`
- [ ] All tests should now pass

**Day 4: Quality Assurance (Core Layer)**

- [ ] Run: `cargo clippy --workspace -- -D warnings` (fix all issues)
- [ ] Run: `cargo fmt --check` (auto-format)
- [ ] Run: `cargo test --lib` (ensure tests pass)
- [ ] Run: `cargo llvm-cov` (verify 80%+ coverage)

**Day 5: Documentation \& Instrumentation**

- [ ] Add doc comments to all public types and functions
- [ ] Add `#[tracing::instrument]` to service functions
- [ ] Review and approve core implementation


### Phase D: Integration Layers (Week 3–4)

**Day 1–2: API Layer**

- [ ] Create HTTP handlers in `crates/api/src/handlers/[feature].rs`
- [ ] Map HTTP requests to domain commands
- [ ] Generate DTOs in `crates/api/src/dto/`
- [ ] Add error handling and response serialization

**Day 2–3: Storage Layer**

- [ ] Implement repository trait in `crates/storage/src/[feature]/`
- [ ] Create database migrations in `crates/storage/migrations/`
- [ ] Add connection pooling and transaction handling

**Day 4: Integration Testing**

- [ ] Write end-to-end tests in `tests/integration/[feature].rs`
- [ ] Test full flow: API → Core → DB
- [ ] Use database fixtures for reproducible tests

**Day 5: Full Quality Gate**

- [ ] Run complete CI pipeline: `cargo fmt`, `clippy`, `deny`, `test`
- [ ] Verify code coverage ≥80%
- [ ] Fix any remaining warnings or failures


### Phase E: Observability \& Hardening (Week 4)

**Day 1: Tracing \& Logging**

- [ ] Initialize tracing subscriber in `crates/api/src/main.rs` (Section 5.4)
- [ ] Add RequestId middleware for correlation
- [ ] Configure structured JSON logging
- [ ] Test log output

**Day 2: Security Hardening**

- [ ] Audit for secret management (use `secrecy::Secret<T>`)
- [ ] Verify all user input validated
- [ ] Check for SQL injection risks (ensure using parameterized queries)
- [ ] Test error messages don't leak sensitive info

**Day 3: Performance Baseline**

- [ ] Add criterion benchmarks in `benches/`
- [ ] Create flamegraph profile for hot paths
- [ ] Document baseline performance metrics

**Day 4–5: Documentation \& Deployment Prep**

- [ ] Write architecture documentation in `docs/architecture.md`
- [ ] Create deployment checklist
- [ ] Set up staging environment
- [ ] Deploy and monitor


### Phase F: Ongoing Maintenance \& Expansion

**Weekly Cadence:**

- [ ] Monday: Review failing CI checks, prioritize fixes
- [ ] Tuesday–Thursday: Implement new features (Phases B–D)
- [ ] Friday: Full quality assurance pass, deploy to staging
- [ ] Weekend: Monitor production, collect feedback

**New Features:**

- Follow the same workflow: Requirements → RFC → Design → Impl → QA

***

## 9. Key Differentiators: Your FAANG-Quality Edge

### 1. Compiler-Enforced Architecture

The Rust compiler + clippy enforces your tenets. Invalid code **doesn't compile**. No human discipline required.

### 2. Automated Quality Gates

Every commit passes:

- Code formatting
- Static analysis (clippy)
- Security audit (cargo-deny)
- All tests (unit, integration, property-based)
- Code coverage

Bad code **cannot** be merged.

### 3. Context-Aware AI (Cursor)

`.cursorrules` encodes 10+ years of FAANG engineering wisdom. Cursor generates code that already follows your principles.

### 4. Parallel Multi-Agent Work (Cursor 2.0+)

What takes a team days, you orchestrate in hours via parallel agents.

### 5. Type-First Design

You design the type system (NewTypes, traits, enums). Cursor implements under your constraints. The compiler validates correctness.

### 6. Zero-Cost Abstractions

All your architecture has zero runtime overhead. `#[repr(transparent)]` NewTypes compile to the same machine code as raw types.

***

## 10. Conclusion

You now have a **complete, end-to-end blueprint** for building FAANG-quality, planet-scale Rust systems as a solo developer using Cursor:

1. **7 Core Tenets**: Architecture principles encoded into type system and automation
2. **Modular Monolith**: Clean separation via crates + workspace boundaries
3. **Docs-as-Code**: RFC, ADR, requirements—all in Git, versioned with code
4. **Robot Team**: Automated enforcement via clippy, deny, tests, CI
5. **Cursor as Crew**: You architect, Cursor implements under `.cursorrules`
6. **End-to-End Workflow**: Business analysis → RFC → Design → Impl → QA → Deploy

This document serves as your canonical reference. Use it for every project. Refine it based on lessons learned.

You now have the systems and discipline to compete with FAANG-scale engineering quality, operating solo.

**Build with conviction. The compiler is your partner.**
<span style="display:none">[^1][^2][^3][^4][^5]</span>

<div align="center">⁂</div>

[^1]: https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/attachments/84231856/42fd019d-adc9-4f2c-9070-919095f68a60/Enterprise-Dev-FAANG-Standards.docx

[^2]: https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/attachments/84231856/8db7c84a-afde-4cb9-8fc7-0768f52ddf14/Core-tenets.docx

[^3]: https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/attachments/84231856/3cc85922-a2d2-4ede-81db-56307472385a/Branch-structure-and-config-files.docx

[^4]: https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/attachments/84231856/9a9f48d6-ab10-4c62-a23f-0ed125ceaff6/Requirements-System-Design.docx

[^5]: https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/attachments/84231856/9bf99032-61d0-4cc2-997c-f06ea7e7d57d/Implementation-Tool-Guide-and-Setup.docx

