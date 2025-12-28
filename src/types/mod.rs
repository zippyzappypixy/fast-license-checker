//! # Domain Types
//!
//! This module contains all the domain-specific types that enforce business rules
//! and provide type safety throughout the fast-license-checker library.
//!
//! ## Type Safety Philosophy
//!
//! Following the "Parse, Don't Validate" principle, all domain concepts are wrapped
//! in NewTypes that guarantee invariants at construction time. This prevents
//! invalid states from existing in the type system.
//!
//! ## Module Organization
//!
//! - [`file_types`] - Types related to file system operations
//! - [`header_types`] - Types related to license headers and comments
//! - [`results`] - Types for scan results and operation outcomes
//!
//! ## Performance Considerations
//!
//! All single-field NewTypes use `#[repr(transparent)]` for zero runtime cost.
//! Types are designed for efficient copying and comparison operations.

pub mod file_types;
pub mod header_types;
pub mod results;

// Re-export all public types for convenience
pub use file_types::*;
pub use header_types::*;
pub use results::*;
