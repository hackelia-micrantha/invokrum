use std::cell::Cell;
use std::collections::BTreeMap;

use invokrum_core::{
    Cardinality, CompositionLimits, Identifier, Overlay, OverlayClass, OverlayPack, OverlaySource,
    PackRelativePath, Profile, SourceFailure, SourceFailureKind,
};
use invokrum_host::{HOST_CONTRACT_VERSION, resolve_bundle, verify_bundle};
use invokrum_integrity::{DriftKind, decode_lockfile};

fn id(value: &str) -> Identifier {
    Identifier::parse(value).expect("test identifier should be valid")
}

fn path(value: &str) -> PackRelativePath {
    PackRelativePath::parse(value).expect("test path should be valid")
}

fn pack() -> OverlayPack {
    OverlayPack::new(
        id("example"),
        "invokrum.dev/v1",
        vec![
            OverlayClass {
                id: id("core"),
                order: 10,
                cardinality: Cardinality::new(1, Some(1)).expect("valid cardinality"),
            },
            OverlayClass {
                id: id("mode"),
                order: 20,
                cardinality: Cardinality::new(1, Some(2)).expect("valid cardinality"),
            },
        ],
        vec![
            Overlay {
                id: id("core-default"),
                class: id("core"),
                source: path("overlays/core.md"),
                incompatible_with: Default::default(),
            },
            Overlay {
                id: id("review"),
                class: id("mode"),
                source: path("overlays/review.md"),
                incompatible_with: Default::default(),
            },
            Overlay {
                id: id("security"),
                class: id("mode"),
                source: path("overlays/security.md"),
                incompatible_with: Default::default(),
            },
        ],
        vec![Profile {
            id: id("governed-review"),
            selections: BTreeMap::from([
                (id("core"), vec![id("core-default")]),
                (id("mode"), vec![id("review"), id("security")]),
            ]),
        }],
        Vec::new(),
    )
    .expect("pack should be valid")
}

struct CountingSource {
    files: BTreeMap<PackRelativePath, Vec<u8>>,
    reads: Cell<usize>,
}

impl CountingSource {
    fn standard() -> Self {
        Self {
            files: BTreeMap::from([
                (path("overlays/core.md"), b"core".to_vec()),
                (path("overlays/review.md"), b"review".to_vec()),
                (path("overlays/security.md"), b"security".to_vec()),
            ]),
            reads: Cell::new(0),
        }
    }

    fn changed() -> Self {
        let mut source = Self::standard();
        source
            .files
            .insert(path("overlays/security.md"), b"changed".to_vec());
        source
    }

    fn reads(&self) -> usize {
        self.reads.get()
    }
}

impl OverlaySource for CountingSource {
    fn load(
        &self,
        source: &PackRelativePath,
        maximum_bytes: usize,
    ) -> Result<Vec<u8>, SourceFailure> {
        self.reads.set(self.reads.get() + 1);
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

#[test]
fn resolve_returns_context_manifest_and_canonical_lock_from_one_read_set() {
    let pack = pack();
    let source = CountingSource::standard();
    let resolved = resolve_bundle(
        &pack,
        &id("governed-review"),
        &source,
        CompositionLimits::default(),
    )
    .expect("host resolution should succeed");

    assert_eq!(HOST_CONTRACT_VERSION, "invokrum.host/v1");
    assert_eq!(source.reads(), 3);
    assert_eq!(resolved.context(), b"core\n\nreview\n\nsecurity");
    assert_eq!(resolved.manifest().entries.len(), 3);
    assert_eq!(resolved.lockfile().manifest.output.byte_length, 24);
    assert_eq!(
        decode_lockfile(resolved.lock_bytes()).expect("lock bytes should be canonical"),
        resolved.lockfile().clone()
    );
}

#[test]
fn verification_does_not_reread_sources_after_current_composition() {
    let pack = pack();
    let baseline_source = CountingSource::standard();
    let baseline = resolve_bundle(
        &pack,
        &id("governed-review"),
        &baseline_source,
        CompositionLimits::default(),
    )
    .expect("baseline should resolve");

    let current_source = CountingSource::changed();
    let verified = verify_bundle(
        baseline.lockfile(),
        &pack,
        &id("governed-review"),
        &current_source,
        CompositionLimits::default(),
    )
    .expect("verification should produce an ordered report");

    assert_eq!(current_source.reads(), 3);
    assert!(!verified.is_verified());
    assert_eq!(
        verified.report().drifts(),
        &[
            DriftKind::OverlayContent { index: 2 },
            DriftKind::RenderedOutput,
        ]
    );
    assert_eq!(verified.current().context(), b"core\n\nreview\n\nchanged");
}
