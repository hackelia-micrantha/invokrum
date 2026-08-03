//! Generic, deterministic prompt-overlay composition primitives.
//!
//! This crate owns mechanism: parsing-neutral domain types, validation,
//! deterministic resolution, rendering inputs, and attestable manifests.
//! It must not encode Anthesis-specific policy, approval, or runtime behavior.

#![forbid(unsafe_code)]

/// Current public schema family understood by the initial architecture.
pub const SCHEMA_FAMILY: &str = "invokrum.dev/v1";
