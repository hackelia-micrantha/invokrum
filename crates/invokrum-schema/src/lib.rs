//! Strict serialization adapters for Invokrum overlay packs.
//!
//! This crate is an infrastructure boundary. It translates YAML and JSON
//! documents into the parsing-neutral domain model owned by `invokrum-core`.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use invokrum_core::{
    Cardinality, DomainError, Identifier, Overlay, OverlayClass, OverlayPack, PackRelativePath,
    Profile, Sensitivity, Variable,
};
use serde::{Deserialize, Serialize};

/// Schema family implemented by this adapter.
pub const SCHEMA_FAMILY: &str = "invokrum.dev/v1";

#[derive(Clone, Debug, Deserialize)]
struct SchemaEnvelope {
    schema: String,
}

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
    minimum: u32,
    #[serde(default)]
    maximum: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OverlayDocument {
    id: String,
    class: String,
    source: String,
    #[serde(default)]
    incompatible_with: Vec<String>,
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
/// The schema envelope is checked before the full strict document so a future
/// schema family receives an unsupported-version error even when it introduces
/// fields unknown to v1.
///
/// # Errors
///
/// Returns [`SchemaError`] when JSON decoding fails, unknown fields are present,
/// the schema family is unsupported, duplicate set values are present, or domain
/// invariants are violated.
pub fn parse_json(input: &str) -> Result<OverlayPack, SchemaError> {
    let envelope: SchemaEnvelope = serde_json::from_str(input)
        .map_err(|error| SchemaError::decode("json", error.to_string()))?;
    ensure_supported_schema(&envelope.schema)?;

    let document: PackDocument = serde_json::from_str(input)
        .map_err(|error| SchemaError::decode("json", error.to_string()))?;
    document.try_into()
}

/// Parse a strict v1 YAML pack document into the validated domain model.
///
/// The schema envelope is checked before the full strict document so a future
/// schema family receives an unsupported-version error even when it introduces
/// fields unknown to v1.
///
/// # Errors
///
/// Returns [`SchemaError`] when YAML decoding fails, unknown fields are present,
/// the schema family is unsupported, duplicate set values are present, or domain
/// invariants are violated.
pub fn parse_yaml(input: &str) -> Result<OverlayPack, SchemaError> {
    let envelope: SchemaEnvelope = serde_yaml::from_str(input)
        .map_err(|error| SchemaError::decode("yaml", error.to_string()))?;
    ensure_supported_schema(&envelope.schema)?;

    let document: PackDocument = serde_yaml::from_str(input)
        .map_err(|error| SchemaError::decode("yaml", error.to_string()))?;
    document.try_into()
}

/// Serialize a validated pack into stable, pretty-printed JSON.
///
/// The domain aggregate normalizes classes by explicit order and other named
/// collections by identifier. Ordered map and set types stabilize nested data.
///
/// # Errors
///
/// Returns [`SchemaError`] if serialization unexpectedly fails.
pub fn to_normalized_json(pack: &OverlayPack) -> Result<String, SchemaError> {
    serde_json::to_string_pretty(&PackDocument::from(pack))
        .map_err(|error| SchemaError::Encode(error.to_string()))
}

fn ensure_supported_schema(schema: &str) -> Result<(), SchemaError> {
    if schema == SCHEMA_FAMILY {
        Ok(())
    } else {
        Err(SchemaError::UnsupportedSchema(schema.to_owned()))
    }
}

fn parse_identifier_set(
    values: Vec<String>,
    field: &'static str,
) -> Result<BTreeSet<Identifier>, SchemaError> {
    let mut identifiers = BTreeSet::new();
    for value in values {
        let identifier = Identifier::parse(value)?;
        let duplicate = identifier.clone();
        if !identifiers.insert(identifier) {
            return Err(SchemaError::DuplicateListValue {
                field,
                value: duplicate,
            });
        }
    }
    Ok(identifiers)
}

impl TryFrom<PackDocument> for OverlayPack {
    type Error = SchemaError;

    fn try_from(document: PackDocument) -> Result<Self, Self::Error> {
        ensure_supported_schema(&document.schema)?;

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
                    incompatible_with: parse_identifier_set(
                        overlay.incompatible_with,
                        "overlays[].incompatible_with",
                    )?,
                })
            })
            .collect::<Result<Vec<_>, SchemaError>>()?;

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

/// A schema-boundary failure while decoding, validating, or encoding a pack.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaError {
    Decode {
        format: &'static str,
        message: String,
    },
    Encode(String),
    UnsupportedSchema(String),
    DuplicateListValue {
        field: &'static str,
        value: Identifier,
    },
    Domain(DomainError),
}

impl SchemaError {
    fn decode(format: &'static str, message: String) -> Self {
        Self::Decode { format, message }
    }
}

impl From<DomainError> for SchemaError {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode { format, message } => write!(formatter, "invalid {format}: {message}"),
            Self::Encode(message) => {
                write!(formatter, "failed to encode normalized JSON: {message}")
            }
            Self::UnsupportedSchema(schema) => {
                write!(formatter, "unsupported schema family: {schema}")
            }
            Self::DuplicateListValue { field, value } => {
                write!(formatter, "duplicate value `{value}` in {field}")
            }
            Self::Domain(error) => write!(formatter, "invalid pack: {error}"),
        }
    }
}

impl Error for SchemaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Domain(error) => Some(error),
            _ => None,
        }
    }
}
