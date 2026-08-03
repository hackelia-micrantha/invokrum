use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component, Path};

/// A validated, portable identifier used by pack domain objects.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Identifier(String);

impl Identifier {
    /// Parses an identifier containing ASCII letters, digits, `.`, `_`, or `-`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidIdentifier`] when the value is empty,
    /// exceeds 128 bytes, or contains an unsupported character.
    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.is_empty() || value.len() > 128 {
            return Err(DomainError::InvalidIdentifier(value));
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(DomainError::InvalidIdentifier(value));
        }
        Ok(Self(value))
    }

    /// Returns the validated identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A non-empty path that cannot lexically escape its pack root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackRelativePath(String);

impl PackRelativePath {
    /// Parses a path relative to the pack root.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidPackRelativePath`] when the value is
    /// empty, absolute, or contains a parent, root, or platform prefix
    /// component.
    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        let path = Path::new(&value);
        if value.is_empty()
            || path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(DomainError::InvalidPackRelativePath(value));
        }
        Ok(Self(value))
    }

    /// Returns the validated pack-relative path text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Inclusive minimum and optional maximum selections for an overlay class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cardinality {
    minimum: usize,
    maximum: Option<usize>,
}

impl Cardinality {
    /// Creates a cardinality constraint.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidCardinality`] when `maximum` is lower
    /// than `minimum`.
    pub fn new(minimum: usize, maximum: Option<usize>) -> Result<Self, DomainError> {
        if maximum.is_some_and(|maximum| maximum < minimum) {
            return Err(DomainError::InvalidCardinality { minimum, maximum });
        }
        Ok(Self { minimum, maximum })
    }

    /// Returns the inclusive minimum selection count.
    #[must_use]
    pub const fn minimum(self) -> usize {
        self.minimum
    }

    /// Returns the inclusive maximum selection count, when bounded.
    #[must_use]
    pub const fn maximum(self) -> Option<usize> {
        self.maximum
    }

    /// Reports whether `count` satisfies this cardinality.
    #[must_use]
    pub fn accepts(self, count: usize) -> bool {
        count >= self.minimum && self.maximum.is_none_or(|maximum| count <= maximum)
    }
}

/// Whether a variable may be exposed in ordinary outputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sensitivity {
    Public,
    Secret,
}

/// A declared variable and its sensitivity classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variable {
    pub name: Identifier,
    pub sensitivity: Sensitivity,
}

/// An explicitly ordered overlay class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayClass {
    pub id: Identifier,
    pub order: u32,
    pub cardinality: Cardinality,
}

/// A selectable overlay belonging to one class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Overlay {
    pub id: Identifier,
    pub class: Identifier,
    pub source: PackRelativePath,
    pub incompatible_with: BTreeSet<Identifier>,
}

/// A named set of overlay selections keyed by class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    pub id: Identifier,
    pub selections: BTreeMap<Identifier, Vec<Identifier>>,
}

/// The validated aggregate root for a complete overlay pack.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayPack {
    pub id: Identifier,
    pub schema_family: String,
    classes: Vec<OverlayClass>,
    overlays: Vec<Overlay>,
    profiles: Vec<Profile>,
    variables: Vec<Variable>,
}

impl OverlayPack {
    /// Constructs and validates a complete overlay pack aggregate.
    ///
    /// Class definitions are normalized by their explicit numeric order.
    ///
    /// # Errors
    ///
    /// Returns a [`DomainError`] for duplicate declarations, duplicate class
    /// ordering, dangling references, class mismatches, invalid profile
    /// cardinality, or an empty schema family.
    pub fn new(
        id: Identifier,
        schema_family: impl Into<String>,
        mut classes: Vec<OverlayClass>,
        overlays: Vec<Overlay>,
        profiles: Vec<Profile>,
        variables: Vec<Variable>,
    ) -> Result<Self, DomainError> {
        let schema_family = schema_family.into();
        if schema_family.is_empty() {
            return Err(DomainError::EmptySchemaFamily);
        }

        ensure_unique(classes.iter().map(|class| &class.id), "class")?;
        ensure_unique(overlays.iter().map(|overlay| &overlay.id), "overlay")?;
        ensure_unique(profiles.iter().map(|profile| &profile.id), "profile")?;
        ensure_unique(variables.iter().map(|variable| &variable.name), "variable")?;

        classes.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| left.id.cmp(&right.id))
        });
        for pair in classes.windows(2) {
            if pair[0].order == pair[1].order {
                return Err(DomainError::DuplicateClassOrder(pair[0].order));
            }
        }

        let class_ids: BTreeSet<_> = classes.iter().map(|class| &class.id).collect();
        let overlay_ids: BTreeSet<_> = overlays.iter().map(|overlay| &overlay.id).collect();

        for overlay in &overlays {
            if !class_ids.contains(&overlay.class) {
                return Err(DomainError::UnknownClass {
                    overlay: overlay.id.clone(),
                    class: overlay.class.clone(),
                });
            }
            for incompatible in &overlay.incompatible_with {
                if !overlay_ids.contains(incompatible) {
                    return Err(DomainError::UnknownOverlayReference(incompatible.clone()));
                }
            }
        }

        for profile in &profiles {
            for class in profile.selections.keys() {
                if !class_ids.contains(class) {
                    return Err(DomainError::UnknownProfileClass {
                        profile: profile.id.clone(),
                        class: class.clone(),
                    });
                }
            }

            for class_definition in &classes {
                let selected = profile.selections.get(&class_definition.id);
                let count = selected.map_or(0, Vec::len);
                if !class_definition.cardinality.accepts(count) {
                    return Err(DomainError::CardinalityViolation {
                        profile: profile.id.clone(),
                        class: class_definition.id.clone(),
                        count,
                    });
                }

                let mut seen = BTreeSet::new();
                for overlay_id in selected.into_iter().flatten() {
                    if !seen.insert(overlay_id) {
                        return Err(DomainError::DuplicateProfileSelection(overlay_id.clone()));
                    }
                    let overlay = overlays
                        .iter()
                        .find(|candidate| &candidate.id == overlay_id)
                        .ok_or_else(|| DomainError::UnknownOverlayReference(overlay_id.clone()))?;
                    if overlay.class != class_definition.id {
                        return Err(DomainError::OverlayClassMismatch {
                            overlay: overlay.id.clone(),
                            expected: class_definition.id.clone(),
                            actual: overlay.class.clone(),
                        });
                    }
                }
            }
        }

        Ok(Self {
            id,
            schema_family,
            classes,
            overlays,
            profiles,
            variables,
        })
    }

    /// Returns classes in deterministic declared order.
    #[must_use]
    pub fn classes(&self) -> &[OverlayClass] {
        &self.classes
    }

    /// Returns all declared overlays.
    #[must_use]
    pub fn overlays(&self) -> &[Overlay] {
        &self.overlays
    }

    /// Returns all declared profiles.
    #[must_use]
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    /// Returns all declared variables.
    #[must_use]
    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }
}

fn ensure_unique<'a>(
    identifiers: impl IntoIterator<Item = &'a Identifier>,
    kind: &'static str,
) -> Result<(), DomainError> {
    let mut seen = BTreeSet::new();
    for identifier in identifiers {
        if !seen.insert(identifier) {
            return Err(DomainError::DuplicateIdentifier {
                kind,
                identifier: identifier.clone(),
            });
        }
    }
    Ok(())
}

/// A domain invariant violation encountered while constructing a pack.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    InvalidIdentifier(String),
    InvalidPackRelativePath(String),
    InvalidCardinality {
        minimum: usize,
        maximum: Option<usize>,
    },
    EmptySchemaFamily,
    DuplicateIdentifier {
        kind: &'static str,
        identifier: Identifier,
    },
    DuplicateClassOrder(u32),
    UnknownClass {
        overlay: Identifier,
        class: Identifier,
    },
    UnknownProfileClass {
        profile: Identifier,
        class: Identifier,
    },
    UnknownOverlayReference(Identifier),
    DuplicateProfileSelection(Identifier),
    OverlayClassMismatch {
        overlay: Identifier,
        expected: Identifier,
        actual: Identifier,
    },
    CardinalityViolation {
        profile: Identifier,
        class: Identifier,
        count: usize,
    },
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for DomainError {}
