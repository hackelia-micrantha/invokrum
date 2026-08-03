use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    Cardinality, DomainError, Identifier, Overlay, OverlayClass, OverlayPack, PackRelativePath,
    Profile, Sensitivity, Variable, SCHEMA_FAMILY,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PackDocument {
    schema: String,
    id: String,
    classes: Vec<ClassDocument>,
    #[serde(default)]
    overlays: Vec<OverlayDocument>,
    #[serde(default)]
    profiles: Vec<ProfileDocument>,
    #[serde(default)]
    variables: Vec<VariableDocument>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ClassDocument {
    id: String,
    order: u32,
    minimum: usize,
    maximum: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OverlayDocument {
    id: String,
    class: String,
    source: String,
    #[serde(default)]
    incompatible_with: BTreeSet<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfileDocument {
    id: String,
    selections: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VariableDocument {
    name: String,
    sensitivity: SensitivityDocument,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum SensitivityDocument {
    Public,
    Secret,
}

/// Parse a strict v1 JSON pack document into the validated domain model.
///
/// # Errors
///
/// Returns [`SchemaError`] when JSON decoding fails, unknown fields are present,
/// the schema family is unsupported, or domain invariants are violated.
pub fn parse_json(input: &str) -> Result<OverlayPack, SchemaError> {
    let document = serde_json::from_str(input).map_err(|error| SchemaError::Decode(error.to_string()))?;
    document.try_into()
}

/// Parse a strict v1 YAML pack document into the validated domain model.
///
/// # Errors
///
/// Returns [`SchemaError`] when YAML decoding fails, unknown fields are present,
/// the schema family is unsupported, or domain invariants are violated.
pub fn parse_yaml(input: &str) -> Result<OverlayPack, SchemaError> {
    let document = serde_yaml::from_str(input).map_err(|error| SchemaError::Decode(error.to_string()))?;
    document.try_into()
}

/// Serialize a validated pack into stable, pretty-printed JSON.
///
/// Collections are normalized by the domain model and ordered map/set types.
///
/// # Errors
///
/// Returns [`SchemaError`] if serialization unexpectedly fails.
pub fn to_normalized_json(pack: &OverlayPack) -> Result<String, SchemaError> {
    serde_json::to_string_pretty(&PackDocument::from(pack))
        .map_err(|error| SchemaError::Encode(error.to_string()))
}

impl TryFrom<PackDocument> for OverlayPack {
    type Error = SchemaError;

    fn try_from(document: PackDocument) -> Result<Self, Self::Error> {
        if document.schema != SCHEMA_FAMILY {
            return Err(SchemaError::UnsupportedSchema(document.schema));
        }

        let classes = document
            .classes
            .into_iter()
            .map(|class| {
                Ok(OverlayClass {
                    id: Identifier::parse(class.id)?,
                    order: class.order,
                    cardinality: Cardinality::new(class.minimum, class.maximum)?,
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;

        let overlays = document
            .overlays
            .into_iter()
            .map(|overlay| {
                Ok(Overlay {
                    id: Identifier::parse(overlay.id)?,
                    class: Identifier::parse(overlay.class)?,
                    source: PackRelativePath::parse(overlay.source)?,
                    incompatible_with: overlay
                        .incompatible_with
                        .into_iter()
                        .map(Identifier::parse)
                        .collect::<Result<_, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;

        let profiles = document
            .profiles
            .into_iter()
            .map(|profile| {
                let selections = profile
                    .selections
                    .into_iter()
                    .map(|(class, overlays)| {
                        Ok((
                            Identifier::parse(class)?,
                            overlays
                                .into_iter()
                                .map(Identifier::parse)
                                .collect::<Result<Vec<_>, _>>()?,
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, DomainError>>()?;
                Ok(Profile {
                    id: Identifier::parse(profile.id)?,
                    selections,
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;

        let variables = document
            .variables
            .into_iter()
            .map(|variable| {
                Ok(Variable {
                    name: Identifier::parse(variable.name)?,
                    sensitivity: match variable.sensitivity {
                        SensitivityDocument::Public => Sensitivity::Public,
                        SensitivityDocument::Secret => Sensitivity::Secret,
                    },
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;

        OverlayPack::new(
            Identifier::parse(document.id)?,
            document.schema,
            classes,
            overlays,
            profiles,
            variables,
        )
        .map_err(Into::into)
    }
}

impl From<&OverlayPack> for PackDocument {
    fn from(pack: &OverlayPack) -> Self {
        Self {
            schema: pack.schema_family.clone(),
            id: pack.id.to_string(),
            classes: pack
                .classes()
                .iter()
                .map(|class| ClassDocument {
                    id: class.id.to_string(),
                    order: class.order,
                    minimum: class.cardinality.minimum(),
                    maximum: class.cardinality.maximum(),
                })
                .collect(),
            overlays: pack
                .overlays()
                .iter()
                .map(|overlay| OverlayDocument {
                    id: overlay.id.to_string(),
                    class: overlay.class.to_string(),
                    source: overlay.source.as_str().to_owned(),
                    incompatible_with: overlay
                        .incompatible_with
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                })
                .collect(),
            profiles: pack
                .profiles()
                .iter()
                .map(|profile| ProfileDocument {
                    id: profile.id.to_string(),
                    selections: profile
                        .selections
                        .iter()
                        .map(|(class, overlays)| {
                            (
                                class.to_string(),
                                overlays.iter().map(ToString::to_string).collect(),
                            )
                        })
                        .collect(),
                })
                .collect(),
            variables: pack
                .variables()
                .iter()
                .map(|variable| VariableDocument {
                    name: variable.name.to_string(),
                    sensitivity: match variable.sensitivity {
                        Sensitivity::Public => SensitivityDocument::Public,
                        Sensitivity::Secret => SensitivityDocument::Secret,
                    },
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaError {
    Decode(String),
    Encode(String),
    UnsupportedSchema(String),
    Domain(DomainError),
}

impl From<DomainError> for SchemaError {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for SchemaError {}
