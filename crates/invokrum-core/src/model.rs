use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component, Path};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Identifier(String);

impl Identifier {
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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackRelativePath(String);

impl PackRelativePath {
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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cardinality {
    minimum: usize,
    maximum: Option<usize>,
}

impl Cardinality {
    pub fn new(minimum: usize, maximum: Option<usize>) -> Result<Self, DomainError> {
        if maximum.is_some_and(|maximum| maximum < minimum) {
            return Err(DomainError::InvalidCardinality { minimum, maximum });
        }
        Ok(Self { minimum, maximum })
    }

    pub const fn minimum(self) -> usize {
        self.minimum
    }

    pub const fn maximum(self) -> Option<usize> {
        self.maximum
    }

    pub fn accepts(self, count: usize) -> bool {
        count >= self.minimum && self.maximum.is_none_or(|maximum| count <= maximum)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sensitivity {
    Public,
    Secret,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variable {
    pub name: Identifier,
    pub sensitivity: Sensitivity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayClass {
    pub id: Identifier,
    pub order: u32,
    pub cardinality: Cardinality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Overlay {
    pub id: Identifier,
    pub class: Identifier,
    pub source: PackRelativePath,
    pub incompatible_with: BTreeSet<Identifier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    pub id: Identifier,
    pub selections: BTreeMap<Identifier, Vec<Identifier>>,
}

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

        classes.sort_by(|left, right| left.order.cmp(&right.order).then_with(|| left.id.cmp(&right.id)));
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
            for (class, selected) in &profile.selections {
                let class_definition = classes
                    .iter()
                    .find(|candidate| &candidate.id == class)
                    .ok_or_else(|| DomainError::UnknownProfileClass {
                        profile: profile.id.clone(),
                        class: class.clone(),
                    })?;
                if !class_definition.cardinality.accepts(selected.len()) {
                    return Err(DomainError::CardinalityViolation {
                        profile: profile.id.clone(),
                        class: class.clone(),
                        count: selected.len(),
                    });
                }
                let mut seen = BTreeSet::new();
                for overlay_id in selected {
                    if !seen.insert(overlay_id) {
                        return Err(DomainError::DuplicateProfileSelection(overlay_id.clone()));
                    }
                    let overlay = overlays
                        .iter()
                        .find(|candidate| &candidate.id == overlay_id)
                        .ok_or_else(|| DomainError::UnknownOverlayReference(overlay_id.clone()))?;
                    if overlay.class != *class {
                        return Err(DomainError::OverlayClassMismatch {
                            overlay: overlay.id.clone(),
                            expected: class.clone(),
                            actual: overlay.class.clone(),
                        });
                    }
                }
            }
        }

        Ok(Self { id, schema_family, classes, overlays, profiles, variables })
    }

    pub fn classes(&self) -> &[OverlayClass] {
        &self.classes
    }

    pub fn overlays(&self) -> &[Overlay] {
        &self.overlays
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    InvalidIdentifier(String),
    InvalidPackRelativePath(String),
    InvalidCardinality { minimum: usize, maximum: Option<usize> },
    EmptySchemaFamily,
    DuplicateIdentifier { kind: &'static str, identifier: Identifier },
    DuplicateClassOrder(u32),
    UnknownClass { overlay: Identifier, class: Identifier },
    UnknownProfileClass { profile: Identifier, class: Identifier },
    UnknownOverlayReference(Identifier),
    DuplicateProfileSelection(Identifier),
    OverlayClassMismatch { overlay: Identifier, expected: Identifier, actual: Identifier },
    CardinalityViolation { profile: Identifier, class: Identifier, count: usize },
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for DomainError {}
