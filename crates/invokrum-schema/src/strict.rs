use std::collections::BTreeSet;
use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

const DUPLICATE_MAPPING_KEY_MARKER: &str = "__invokrum_duplicate_mapping_key__";
const MERGE_MAPPING_KEY_MARKER: &str = "__invokrum_merge_mapping_key__";
const MAX_DECODE_MESSAGE_CHARS: usize = 512;

/// A YAML feature excluded from the Invokrum v1 compatibility subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum YamlFeature {
    Directive,
    DocumentEndMarker,
    Anchor,
    Alias,
    MergeKey,
    Tag,
    BlockScalar,
    ExplicitMappingKey,
}

impl fmt::Display for YamlFeature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Directive => "directive",
            Self::DocumentEndMarker => "document end marker",
            Self::Anchor => "anchor",
            Self::Alias => "alias",
            Self::MergeKey => "merge key",
            Self::Tag => "explicit tag",
            Self::BlockScalar => "block scalar",
            Self::ExplicitMappingKey => "explicit mapping key",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PreflightError {
    Decode {
        format: &'static str,
        message: String,
    },
    DuplicateMappingKey {
        format: &'static str,
    },
    UnsupportedYamlFeature(YamlFeature),
    MultipleYamlDocuments,
}

pub(crate) fn preflight_json(input: &str) -> Result<(), PreflightError> {
    serde_json::from_str::<StrictValue<false>>(input)
        .map(|_| ())
        .map_err(|error| classify_decode("json", &error.to_string()))
}

pub(crate) fn preflight_yaml(input: &str) -> Result<(), PreflightError> {
    validate_yaml_subset(input)?;

    let mut documents = serde_yaml_ng::Deserializer::from_str(input);
    let first = documents.next().ok_or_else(|| PreflightError::Decode {
        format: "yaml",
        message: "empty YAML document".to_owned(),
    })?;

    StrictValue::<true>::deserialize(first).map_err(|error| classify_decode("yaml", &error.to_string()))?;

    if documents.next().is_some() {
        return Err(PreflightError::MultipleYamlDocuments);
    }

    Ok(())
}

fn classify_decode(format: &'static str, message: &str) -> PreflightError {
    if message.contains(DUPLICATE_MAPPING_KEY_MARKER) {
        PreflightError::DuplicateMappingKey { format }
    } else if format == "yaml" && message.contains(MERGE_MAPPING_KEY_MARKER) {
        PreflightError::UnsupportedYamlFeature(YamlFeature::MergeKey)
    } else {
        PreflightError::Decode {
            format,
            message: bounded_message(message),
        }
    }
}

fn bounded_message(message: &str) -> String {
    let mut characters = message.chars();
    let mut bounded: String = characters.by_ref().take(MAX_DECODE_MESSAGE_CHARS).collect();
    if characters.next().is_some() {
        bounded.push('…');
    }
    bounded
}

fn validate_yaml_subset(input: &str) -> Result<(), PreflightError> {
    for line in input.lines() {
        let surface = yaml_control_surface(line);
        let trimmed = surface.trim_start();

        if trimmed.starts_with('%') {
            return Err(PreflightError::UnsupportedYamlFeature(
                YamlFeature::Directive,
            ));
        }
        if is_document_marker(trimmed, "...") {
            return Err(PreflightError::UnsupportedYamlFeature(
                YamlFeature::DocumentEndMarker,
            ));
        }
        if surface.contains("<<:") || surface.contains("<< :") {
            return Err(PreflightError::UnsupportedYamlFeature(
                YamlFeature::MergeKey,
            ));
        }
        if surface.contains('&') {
            return Err(PreflightError::UnsupportedYamlFeature(YamlFeature::Anchor));
        }
        if surface.contains('*') {
            return Err(PreflightError::UnsupportedYamlFeature(YamlFeature::Alias));
        }
        if surface.contains('!') {
            return Err(PreflightError::UnsupportedYamlFeature(YamlFeature::Tag));
        }
        if surface.contains('|') || surface.contains('>') {
            return Err(PreflightError::UnsupportedYamlFeature(
                YamlFeature::BlockScalar,
            ));
        }
        if surface.contains('?') {
            return Err(PreflightError::UnsupportedYamlFeature(
                YamlFeature::ExplicitMappingKey,
            ));
        }
    }

    Ok(())
}

fn is_document_marker(input: &str, marker: &str) -> bool {
    input == marker
        || input
            .strip_prefix(marker)
            .and_then(|remainder| remainder.chars().next())
            .is_some_and(char::is_whitespace)
}

fn yaml_control_surface(line: &str) -> String {
    #[derive(Clone, Copy)]
    enum Quote {
        None,
        Single,
        Double,
    }

    let mut quote = Quote::None;
    let mut escaped = false;
    let mut previous_whitespace = true;
    let mut surface = String::with_capacity(line.len());
    let mut characters = line.chars().peekable();

    while let Some(character) = characters.next() {
        match quote {
            Quote::None => {
                if character == '#' && previous_whitespace {
                    break;
                }
                match character {
                    '\'' => {
                        quote = Quote::Single;
                        surface.push(' ');
                        previous_whitespace = false;
                    }
                    '"' => {
                        quote = Quote::Double;
                        surface.push(' ');
                        previous_whitespace = false;
                    }
                    _ => {
                        surface.push(character);
                        previous_whitespace = character.is_whitespace();
                    }
                }
            }
            Quote::Single => {
                surface.push(' ');
                if character == '\'' {
                    if characters.peek().is_some_and(|next| *next == '\'') {
                        characters.next();
                        surface.push(' ');
                    } else {
                        quote = Quote::None;
                    }
                }
            }
            Quote::Double => {
                surface.push(' ');
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == '"' {
                    quote = Quote::None;
                }
            }
        }
    }

    surface
}

struct StrictKey(String);

impl<'de> Deserialize<'de> for StrictKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictKeyVisitor)
    }
}

struct StrictKeyVisitor;

impl Visitor<'_> for StrictKeyVisitor {
    type Value = StrictKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string mapping key")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictKey(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictKey(value))
    }
}

struct StrictValue<const REJECT_MERGE_KEY: bool>;

impl<'de, const REJECT_MERGE_KEY: bool> Deserialize<'de> for StrictValue<REJECT_MERGE_KEY> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictValueVisitor::<REJECT_MERGE_KEY>)
    }
}

struct StrictValueVisitor<const REJECT_MERGE_KEY: bool>;

impl<'de, const REJECT_MERGE_KEY: bool> Visitor<'de>
    for StrictValueVisitor<REJECT_MERGE_KEY>
{
    type Value = StrictValue<REJECT_MERGE_KEY>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a scalar, sequence, or string-keyed mapping")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue)
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue)
    }

    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence
            .next_element::<StrictValue<REJECT_MERGE_KEY>>()?
            .is_some()
        {}
        Ok(StrictValue)
    }

    fn visit_map<A>(self, mut mapping: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        while let Some(StrictKey(key)) = mapping.next_key::<StrictKey>()? {
            if REJECT_MERGE_KEY && key == "<<" {
                return Err(de::Error::custom(MERGE_MAPPING_KEY_MARKER));
            }
            if !keys.insert(key) {
                return Err(de::Error::custom(DUPLICATE_MAPPING_KEY_MARKER));
            }
            let _: StrictValue<REJECT_MERGE_KEY> = mapping.next_value()?;
        }
        Ok(StrictValue)
    }
}
