use invokrum_schema::{SCHEMA_FAMILY, SchemaError, parse_json, parse_yaml, to_normalized_json};
use serde_json::Value;

#[test]
fn yaml_and_json_produce_the_same_normalized_model() {
    let yaml = include_str!("../../../tests/fixtures/schema/minimal-pack.yaml");
    let json = include_str!("../../../tests/fixtures/schema/minimal-pack.json");

    let yaml_pack = parse_yaml(yaml).expect("YAML fixture should be valid");
    let json_pack = parse_json(json).expect("JSON fixture should be valid");

    assert_eq!(
        to_normalized_json(&yaml_pack).expect("YAML pack should serialize"),
        to_normalized_json(&json_pack).expect("JSON pack should serialize")
    );
}

#[test]
fn schema_rejects_unknown_v1_fields() {
    let unknown_field = r#"{
        "schema":"invokrum.dev/v1",
        "id":"example",
        "classes":[],
        "unexpected":true
    }"#;

    assert!(matches!(
        parse_json(unknown_field),
        Err(SchemaError::Decode { format: "json", .. })
    ));
}

#[test]
fn unsupported_schema_is_reported_before_future_fields_are_decoded() {
    let unsupported = r#"{
        "schema":"invokrum.dev/v2",
        "id":"example",
        "classes":[],
        "future_only":true
    }"#;

    assert_eq!(
        parse_json(unsupported),
        Err(SchemaError::UnsupportedSchema("invokrum.dev/v2".to_owned()))
    );
}

#[test]
fn duplicate_incompatibility_entries_are_rejected_instead_of_deduplicated() {
    let duplicate = r#"{
        "schema":"invokrum.dev/v1",
        "id":"example",
        "classes":[
            {"id":"mode","order":10,"minimum":0}
        ],
        "overlays":[
            {
                "id":"read-only",
                "class":"mode",
                "source":"overlays/read-only.md",
                "incompatible_with":["read-only","read-only"]
            }
        ]
    }"#;

    assert!(matches!(
        parse_json(duplicate),
        Err(SchemaError::DuplicateListValue {
            field: "overlays[].incompatible_with",
            ..
        })
    ));
}

#[test]
fn optional_maximum_defaults_to_unbounded() {
    let document = r#"{
        "schema":"invokrum.dev/v1",
        "id":"example",
        "classes":[
            {"id":"quality","order":10,"minimum":0}
        ]
    }"#;

    let pack = parse_json(document).expect("maximum should be optional");
    assert_eq!(pack.classes()[0].cardinality.maximum(), None);
}

#[test]
fn machine_schema_is_valid_json_and_bound_to_the_runtime_family() {
    let schema_text = include_str!("../../../schemas/invokrum-pack-v1.schema.json");
    let schema: Value = serde_json::from_str(schema_text).expect("schema should be valid JSON");

    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["properties"]["schema"]["const"], SCHEMA_FAMILY);
    assert_eq!(schema["additionalProperties"], false);

    let required = schema["$defs"]["class"]["required"]
        .as_array()
        .expect("class required fields should be an array");
    assert!(!required.iter().any(|field| field == "maximum"));
}
