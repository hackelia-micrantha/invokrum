//! Read-only host integration facade for Invokrum.
//!
//! This crate composes one validated profile and derives canonical evidence from
//! the same in-memory bytes. It owns no filesystem, network, process, runtime,
//! serialization, or plugin-loading behavior. Hosts inject an [`OverlaySource`]
//! and may choose a separate transport adapter.

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use invokrum_core::{
    Composition, CompositionError, CompositionLimits, Identifier, OverlayPack, OverlaySource,
    ResolvedManifest, compose,
};
use invokrum_integrity::{
    Digester, IntegrityError, Lockfile, Sha256Digester, VerificationReport, build_lockfile,
    encode_lockfile, verify_with,
};

/// Compatibility identifier for the initial host contract.
pub const HOST_CONTRACT_VERSION: &str = "invokrum.host/v1";

/// One validated composition and the canonical evidence derived from its exact bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedBundle {
    composition: Composition,
    lockfile: Lockfile,
    lock_bytes: Vec<u8>,
}

impl ResolvedBundle {
    /// Returns the exact normalized context bytes a host may bind to invocation.
    #[must_use]
    pub fn context(&self) -> &[u8] {
        self.composition.normalized_context()
    }

    /// Returns the deterministic resolved manifest.
    #[must_use]
    pub fn manifest(&self) -> &ResolvedManifest {
        self.composition.manifest()
    }

    /// Returns the complete validated composition, including ordered source segments.
    #[must_use]
    pub const fn composition(&self) -> &Composition {
        &self.composition
    }

    /// Returns the canonical lock value derived from the composition.
    #[must_use]
    pub const fn lockfile(&self) -> &Lockfile {
        &self.lockfile
    }

    /// Returns exact canonical `invokrum.lock/v1` bytes.
    #[must_use]
    pub fn lock_bytes(&self) -> &[u8] {
        &self.lock_bytes
    }
}

/// Current resolved bytes plus the ordered drift result against expected evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedBundle {
    current: ResolvedBundle,
    report: VerificationReport,
}

impl VerifiedBundle {
    /// Returns whether expected evidence matches current inputs and output exactly.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        self.report.is_verified()
    }

    /// Returns the ordered drift report.
    #[must_use]
    pub const fn report(&self) -> &VerificationReport {
        &self.report
    }

    /// Returns current resolved bytes and evidence regardless of drift.
    ///
    /// A host must not invoke these bytes under the expected lock identity when
    /// [`Self::is_verified`] is false.
    #[must_use]
    pub const fn current(&self) -> &ResolvedBundle {
        &self.current
    }
}

/// Host-facade failure before a resolved bundle or verification report is available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    Composition(CompositionError),
    Integrity(IntegrityError),
}

impl fmt::Display for HostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Composition(error) => write!(formatter, "composition failed: {error}"),
            Self::Integrity(error) => write!(formatter, "integrity operation failed: {error}"),
        }
    }
}

impl Error for HostError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Composition(error) => Some(error),
            Self::Integrity(error) => Some(error),
        }
    }
}

impl From<CompositionError> for HostError {
    fn from(error: CompositionError) -> Self {
        Self::Composition(error)
    }
}

impl From<IntegrityError> for HostError {
    fn from(error: IntegrityError) -> Self {
        Self::Integrity(error)
    }
}

/// Resolves a profile with the default SHA-256 integrity implementation.
///
/// # Errors
///
/// Returns [`HostError`] when composition, lock generation, or canonical lock
/// encoding fails.
pub fn resolve_bundle(
    pack: &OverlayPack,
    profile: &Identifier,
    source: &impl OverlaySource,
    limits: CompositionLimits,
) -> Result<ResolvedBundle, HostError> {
    resolve_bundle_with(pack, profile, source, limits, &Sha256Digester)
}

/// Resolves a profile with an explicitly injected digest capability.
///
/// Source bytes are read exactly once by composition. Lock generation and
/// encoding consume only the returned in-memory composition.
///
/// # Errors
///
/// Returns [`HostError`] when composition, lock generation, or canonical lock
/// encoding fails.
pub fn resolve_bundle_with(
    pack: &OverlayPack,
    profile: &Identifier,
    source: &impl OverlaySource,
    limits: CompositionLimits,
    digester: &impl Digester,
) -> Result<ResolvedBundle, HostError> {
    let composition = compose(pack, profile, source, limits)?;
    let lockfile = build_lockfile(pack, &composition, digester)?;
    let lock_bytes = encode_lockfile(&lockfile)?;
    Ok(ResolvedBundle {
        composition,
        lockfile,
        lock_bytes,
    })
}

/// Resolves current bytes and verifies them against expected evidence using SHA-256.
///
/// # Errors
///
/// Returns [`HostError`] when current composition or integrity validation fails.
pub fn verify_bundle(
    expected: &Lockfile,
    pack: &OverlayPack,
    profile: &Identifier,
    source: &impl OverlaySource,
    limits: CompositionLimits,
) -> Result<VerifiedBundle, HostError> {
    verify_bundle_with(
        expected,
        pack,
        profile,
        source,
        limits,
        &Sha256Digester,
    )
}

/// Resolves current bytes and verifies them with an injected digest capability.
///
/// Verification may rebuild digest values from the in-memory composition but
/// never reopens or rereads an overlay source.
///
/// # Errors
///
/// Returns [`HostError`] when current composition or integrity validation fails.
pub fn verify_bundle_with(
    expected: &Lockfile,
    pack: &OverlayPack,
    profile: &Identifier,
    source: &impl OverlaySource,
    limits: CompositionLimits,
    digester: &impl Digester,
) -> Result<VerifiedBundle, HostError> {
    let current = resolve_bundle_with(pack, profile, source, limits, digester)?;
    let report = verify_with(expected, pack, current.composition(), digester)?;
    Ok(VerifiedBundle { current, report })
}
