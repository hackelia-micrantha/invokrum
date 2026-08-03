use std::collections::{BTreeMap, BTreeSet};

use invokrum_core::{
    Cardinality, Composition, CompositionLimits, Identifier, Overlay, OverlayClass, OverlayPack,
    OverlaySource, PackRelativePath, Profile, Sensitivity, SourceFailure, SourceFailureKind,
    Variable, compose,
};
use invokrum_integrity::{
    DriftKind, IntegrityError, Sha256Digester, build_lockfile, decode_lockfile, encode_lockfile,
    verify,
};

fn id(value: &str) -> Identifier {
    Identifier::parse(value).expect("test identifier should be valid")
}

fn path(value: &str) -> PackRelativePath {
    PackRelativePath::parse(value).expect("test path should be valid")
}

fn class(name: &str, order: u32) -> OverlayClass {
    OverlayClass {
        id: id(name),
        order,
        cardinality: Cardinality::new(1, Some(1)).expect("valid cardinality"),
    }
}

fn overlay(name: &str, class_name: &str, source: &str) -> Overlay {
    Overlay {
        id: id(name),
        class: id(class_name),
        source: path(source),
        incompatible_with: BTreeSet::new(),
    }
}

fn pack(reverse_declarations: bool, safe_mode: bool) -> OverlayPack {
    let mut classes = vec![class("core", 10), class("mode", 20)];
    let mut overlays = vec![
        overlay("core-default", "core", "overlays/core.md"),
        overlay("review", "mode", "overlays/review.md"),
        overlay("safe", "mode", "overlays/safe.md"),
    ];
    if reverse_declarations {
        classes.reverse();
        overlays.reverse();
    }
    let selected_mode = if safe_mode { "safe" } else { "review" };
    let profile = Profile {
        id: id("default"),
        selections: BTreeMap::from([
            (id("core"), vec![id("core-default")]),
            (id("mode"), vec![id(selected_mode)]),
        ]),
    };
    OverlayPack::new(
        id("example"),
        "invokrum.dev/v1",
        classes,
        overlays,
        vec![profile],
        vec![Variable {
            name: id("api-token"),
            sensitivity: Sensitivity::Secret,
        }],
    )
    .expect("test pack should be valid")
}

#[derive(Default)]
struct MemorySource {
    files: BTreeMap<PackRelativePath, Vec<u8>>,
}

impl MemorySource {
    fn with(mut self, source: &str, bytes: &[u8]) -> Self {
        self.files.insert(path(source), bytes.to_vec());
        self
    }
}

impl OverlaySource for MemorySource {
    fn load(
        &self,
        source: &PackRelativePath,
        maximum_bytes: usize,
    ) -> Result<Vec<u8>, SourceFailure> {
        let bytes = self
            .files
            .get(source)
            .ok_or_else(|| SourceFailure::new(source.clone(), SourceFailureKind::NotFound))?;
        if bytes.len() > maximum_bytes {
            return Err(SourceFailure::new(
                source.clone(),
                SourceFailureKind::TooLarge,
            ));
        }
        Ok(bytes.clone())
    }
}

fn sources(review: &[u8]) -> MemorySource {
    MemorySource::default()
        .with("overlays/core.md", b"core")
        .with("overlays/review.md", review)
        .with("overlays/safe.md", b"safe")
}

fn composition(pack: &OverlayPack, source: &MemorySource) -> Composition {
    compose(pack, &id("default"), source, CompositionLimits::default())
        .expect("composition should succeed")
}

#[test]
fn equivalent_declaration_order_produces_identical_lock_bytes() {
    let first_pack = pack(false, false);
    let second_pack = pack(true, false);
    let first = build_lockfile(
        &first_pack,
        &composition(&first_pack, &sources(b"review")),
        &Sha256Digester,
    )
    .expect("lock should build");
    let second = build_lockfile(
        &second_pack,
        &composition(&second_pack, &sources(b"review")),
        &Sha256Digester,
    )
    .expect("lock should build");

    assert_eq!(
        encode_lockfile(&first).expect("lock should encode"),
        encode_lockfile(&second).expect("lock should encode")
    );
}

#[test]
fn verification_reports_content_and_output_drift_separately() {
    let pack = pack(false, false);
    let expected = composition(&pack, &sources(b"review"));
    let current = composition(&pack, &sources(b"changed"));
    let lock = build_lockfile(&pack, &expected, &Sha256Digester).expect("lock should build");
    let report = verify(&lock, &pack, &current).expect("verification should run");

    assert_eq!(
        report.drifts(),
        [
            DriftKind::OverlayContent { index: 1 },
            DriftKind::RenderedOutput,
        ]
    );
}

#[test]
fn verification_reports_configuration_and_overlay_set_drift() {
    let expected_pack = pack(false, false);
    let current_pack = pack(false, true);
    let lock = build_lockfile(
        &expected_pack,
        &composition(&expected_pack, &sources(b"review")),
        &Sha256Digester,
    )
    .expect("lock should build");
    let report = verify(
        &lock,
        &current_pack,
        &composition(&current_pack, &sources(b"review")),
    )
    .expect("verification should run");

    assert_eq!(
        report.drifts(),
        [
            DriftKind::PackMetadata,
            DriftKind::ProfileSelection,
            DriftKind::OverlaySet,
            DriftKind::RenderedOutput,
        ]
    );
}

#[test]
fn unsupported_versions_and_tampered_manifests_fail_before_drift_comparison() {
    let pack = pack(false, false);
    let lock = build_lockfile(
        &pack,
        &composition(&pack, &sources(b"review")),
        &Sha256Digester,
    )
    .expect("lock should build");
    let encoded = String::from_utf8(encode_lockfile(&lock).expect("lock should encode"))
        .expect("lock should be UTF-8");

    let unsupported = encoded.replacen("invokrum.lock/v1", "invokrum.lock/v2", 1);
    assert_eq!(
        decode_lockfile(unsupported.as_bytes()),
        Err(IntegrityError::UnsupportedFormat)
    );

    let tampered = encoded.replacen("\"byte_length\":4", "\"byte_length\":5", 1);
    assert_eq!(
        decode_lockfile(tampered.as_bytes()),
        Err(IntegrityError::LockfileIntegrityMismatch)
    );
}

#[test]
fn lockfile_contains_no_sensitive_variable_names_or_values() {
    let pack = pack(false, false);
    let lock = build_lockfile(
        &pack,
        &composition(&pack, &sources(b"review")),
        &Sha256Digester,
    )
    .expect("lock should build");
    let encoded = String::from_utf8(encode_lockfile(&lock).expect("lock should encode"))
        .expect("lock should be UTF-8");

    assert!(!encoded.contains("api-token"));
    assert!(!encoded.contains("super-secret-value"));
}

#[test]
fn canonical_lockfile_round_trips_and_verifies() {
    let pack = pack(false, false);
    let composition = composition(&pack, &sources(b"review"));
    let lock = build_lockfile(&pack, &composition, &Sha256Digester).expect("lock should build");
    let encoded = encode_lockfile(&lock).expect("lock should encode");
    let decoded = decode_lockfile(&encoded).expect("lock should decode");
    let report = verify(&decoded, &pack, &composition).expect("verification should run");

    assert_eq!(decoded, lock);
    assert!(report.is_verified());
    assert!(report.drifts().is_empty());
}
