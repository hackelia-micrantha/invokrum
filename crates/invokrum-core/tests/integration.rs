use std::collections::{BTreeMap, BTreeSet};

use invokrum_core::{
    Cardinality, DomainError, Identifier, Overlay, OverlayClass, OverlayPack, PackRelativePath,
    Profile, SCHEMA_FAMILY, SchemaError, parse_json, parse_yaml, to_normalized_json,
};

fn id(value: &str) -> Identifier {
    Identifier::parse(value).expect("test identifier should be valid")
}

fn path(value: &str) -> PackRelativePath {
    PackRelativePath::parse(value).expect("test path should be valid")
}

#[test]
fn public_schema_contract_is_available_to_consumers() {
    assert_eq!(SCHEMA_FAMILY, "invokrum.dev/v1");
}

#[test]
fn pack_construction_normalizes_declared_class_order() {
    let classes = vec![
        OverlayClass {
            id: id("quality"),
            order: 30,
            cardinality: Cardinality::new(0, None).expect("valid cardinality"),
        },
        OverlayClass {
            id: id("core"),
            order: 10,
            cardinality: Cardinality::new(1, Some(1)).expect("valid cardinality"),
        },
        OverlayClass {
            id: id("mode"),
            order: 20,
            cardinality: Cardinality::new(1, Some(1)).expect("valid cardinality"),
        },
    ];
    let overlays = vec![
        Overlay {
            id: id("core-default"),
            class: id("core"),
            source: path("overlays/core.md"),
            incompatible_with: BTreeSet::new(),
        },
        Overlay {
            id: id("read-only"),
            class: id("mode"),
            source: path("overlays/read-only.md"),
            incompatible_with: BTreeSet::new(),
        },
    ];
    let profile = Profile {
        id: id("review"),
        selections: BTreeMap::from([
            (id("core"), vec![id("core-default")]),
            (id("mode"), vec![id("read-only")]),
        ]),
    };

    let pack = OverlayPack::new(
        id("example"),
        SCHEMA_FAMILY,
        classes,
        overlays,
        vec![profile],
        Vec::new(),
    )
    .expect("pack should be valid");

    let ordered: Vec<_> = pack
        .classes()
        .iter()
        .map(|class| class.id.as_str())
        .collect();
    assert_eq!(ordered, vec!["core", "mode", "quality"]);
}

#[test]
fn pack_rejects_profile_selection_from_the_wrong_class() {
    let classes = vec![
        OverlayClass {
            id: id("core"),
            order: 10,
            cardinality: Cardinality::new(0, None).expect("valid cardinality"),
        },
        OverlayClass {
            id: id("mode"),
            order: 20,
            cardinality: Cardinality::new(0, None).expect("valid cardinality"),
        },
    ];
    let overlays = vec![Overlay {
        id: id("read-only"),
        class: id("mode"),
        source: path("overlays/read-only.md"),
        incompatible_with: BTreeSet::new(),
    }];
    let profile = Profile {
        id: id("invalid"),
        selections: BTreeMap::from([(id("core"), vec![id("read-only")])]),
    };

    let result = OverlayPack::new(
        id("example"),
        SCHEMA_FAMILY,
        classes,
        overlays,
        vec![profile],
        Vec::new(),
    );

    assert!(matches!(
        result,
        Err(DomainError::OverlayClassMismatch { .. })
    ));
}

#[test]
fn pack_rejects_profile_that_omits_a_required_class() {
    let classes = vec![OverlayClass {
        id: id("core"),
        order: 10,
        cardinality: Cardinality::new(1, Some(1)).expect("valid cardinality"),
    }];
    let profile = Profile {
        id: id("invalid"),
        selections: BTreeMap::new(),
    };

    let result = OverlayPack::new(
        id("example"),
        SCHEMA_FAMILY,
        classes,
        Vec::new(),
        vec![profile],
        Vec::new(),
    );

    assert!(matches!(
        result,
        Err(DomainError::CardinalityViolation { count: 0, .. })
    ));
}

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
fn schema_rejects_unknown_fields_and_versions() {
    let unknown_field = r#"{
        "schema":"invokrum.dev/v1",
        "id":"example",
        "classes":[],
        "unexpected":true
    }"#;
    assert!(matches!(parse_json(unknown_field), Err(SchemaError::Decode(_))));

    let unsupported = r#"{
        "schema":"invokrum.dev/v2",
        "id":"example",
        "classes":[]
    }"#;
    assert!(matches!(
        parse_json(unsupported),
        Err(SchemaError::UnsupportedSchema(_))
    ));
}
