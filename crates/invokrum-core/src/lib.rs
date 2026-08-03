//! Generic, deterministic prompt-overlay composition primitives.
//!
//! This crate owns mechanism: parsing-neutral domain types, validation,
//! deterministic resolution, rendering inputs, and attestable manifests.
//! It must not encode Anthesis-specific policy, approval, or runtime behavior.

#![forbid(unsafe_code)]

pub mod model;

pub use model::{
    Cardinality, DomainError, Identifier, Overlay, OverlayClass, OverlayPack, PackRelativePath,
    Profile, Sensitivity, Variable,
};

/// Current public schema family understood by the initial architecture.
pub const SCHEMA_FAMILY: &str = "invokrum.dev/v1";

#[cfg(test)]
mod tests {
    use super::SCHEMA_FAMILY;

    #[test]
    fn schema_family_is_stable_and_versioned() {
        assert_eq!(SCHEMA_FAMILY, "invokrum.dev/v1");
        assert!(SCHEMA_FAMILY.ends_with("/v1"));
    }
}
