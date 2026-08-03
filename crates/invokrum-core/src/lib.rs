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
    use super::{Cardinality, DomainError, Identifier, PackRelativePath, SCHEMA_FAMILY};

    #[test]
    fn schema_family_is_stable_and_versioned() {
        assert_eq!(SCHEMA_FAMILY, "invokrum.dev/v1");
        assert!(SCHEMA_FAMILY.ends_with("/v1"));
    }

    #[test]
    fn identifiers_reject_empty_and_structural_characters() {
        assert!(matches!(
            Identifier::parse(""),
            Err(DomainError::InvalidIdentifier(_))
        ));
        assert!(matches!(
            Identifier::parse("mode/read-only"),
            Err(DomainError::InvalidIdentifier(_))
        ));
        assert_eq!(
            Identifier::parse("read-only").expect("identifier should be valid").as_str(),
            "read-only"
        );
    }

    #[test]
    fn pack_paths_reject_absolute_and_parent_segments() {
        assert!(PackRelativePath::parse("/etc/passwd").is_err());
        assert!(PackRelativePath::parse("overlays/../secret.md").is_err());
        assert_eq!(
            PackRelativePath::parse("overlays/core.md")
                .expect("path should be valid")
                .as_str(),
            "overlays/core.md"
        );
    }

    #[test]
    fn cardinality_rejects_inverted_bounds() {
        assert!(matches!(
            Cardinality::new(2, Some(1)),
            Err(DomainError::InvalidCardinality { .. })
        ));
        let exactly_one = Cardinality::new(1, Some(1)).expect("cardinality should be valid");
        assert!(exactly_one.accepts(1));
        assert!(!exactly_one.accepts(0));
        assert!(!exactly_one.accepts(2));
    }
}
