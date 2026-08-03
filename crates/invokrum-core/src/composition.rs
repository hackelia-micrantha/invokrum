use std::fmt;

use crate::{Identifier, Overlay, OverlayPack, PackRelativePath};

/// Explicit resource limits applied during composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompositionLimits {
    maximum_overlays: usize,
    maximum_overlay_bytes: usize,
    maximum_output_bytes: usize,
}

impl CompositionLimits {
    /// Creates composition limits.
    #[must_use]
    pub const fn new(
        maximum_overlays: usize,
        maximum_overlay_bytes: usize,
        maximum_output_bytes: usize,
    ) -> Self {
        Self {
            maximum_overlays,
            maximum_overlay_bytes,
            maximum_output_bytes,
        }
    }

    /// Returns the maximum number of selected overlays.
    #[must_use]
    pub const fn maximum_overlays(self) -> usize {
        self.maximum_overlays
    }

    /// Returns the maximum bytes accepted from one overlay source.
    #[must_use]
    pub const fn maximum_overlay_bytes(self) -> usize {
        self.maximum_overlay_bytes
    }

    /// Returns the maximum normalized output size.
    #[must_use]
    pub const fn maximum_output_bytes(self) -> usize {
        self.maximum_output_bytes
    }
}

impl Default for CompositionLimits {
    fn default() -> Self {
        Self::new(256, 1_048_576, 8_388_608)
    }
}

/// Port used by the composition use case to obtain stable source bytes.
///
/// Implementations must return bytes from one stable read, perform no network
/// access, and fail closed when the requested path cannot be proven acceptable.
pub trait OverlaySource {
    /// Loads one validated pack-relative source.
    ///
    /// # Errors
    ///
    /// Returns a stable [`SourceFailure`] category when the source cannot be
    /// loaded under the adapter's containment and file-type policy.
    fn load(
        &self,
        source: &PackRelativePath,
        maximum_bytes: usize,
    ) -> Result<Vec<u8>, SourceFailure>;
}

/// Stable category for an outward source-adapter rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceFailureKind {
    NotFound,
    NotRegularFile,
    SymbolicLink,
    HardLink,
    MountBoundary,
    RootEscape,
    ChangedDuringRead,
    TooLarge,
    PermissionDenied,
    UnsupportedPlatform,
    Io,
}

impl fmt::Display for SourceFailureKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotFound => "not found",
            Self::NotRegularFile => "not a regular file",
            Self::SymbolicLink => "symbolic link rejected",
            Self::HardLink => "hard link rejected",
            Self::MountBoundary => "filesystem boundary rejected",
            Self::RootEscape => "pack-root escape rejected",
            Self::ChangedDuringRead => "source changed during read",
            Self::TooLarge => "source exceeds configured size limit",
            Self::PermissionDenied => "permission denied",
            Self::UnsupportedPlatform => "unsupported platform",
            Self::Io => "I/O failure",
        })
    }
}

/// A source-adapter failure tied to a validated pack-relative path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFailure {
    pub path: PackRelativePath,
    pub kind: SourceFailureKind,
}

impl SourceFailure {
    /// Creates a stable source failure without embedding host error text.
    #[must_use]
    pub const fn new(path: PackRelativePath, kind: SourceFailureKind) -> Self {
        Self { path, kind }
    }
}

/// One normalized manifest entry in composition order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedEntry {
    pub class: Identifier,
    pub overlay: Identifier,
    pub source: PackRelativePath,
    pub byte_length: usize,
}

/// Deterministic structural description of a resolved profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedManifest {
    pub pack: Identifier,
    pub schema_family: String,
    pub profile: Identifier,
    pub entries: Vec<ResolvedEntry>,
    pub source_bytes: usize,
    pub output_bytes: usize,
}

/// Exact source bytes paired with their validated structural authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSegment {
    pub class: Identifier,
    pub overlay: Identifier,
    pub source: PackRelativePath,
    pub bytes: Vec<u8>,
}

/// A deterministic composition result.
///
/// Structural ordering is represented independently from overlay prose, so a
/// later segment cannot redefine the class authority or ordering established by
/// the validated pack and profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Composition {
    manifest: ResolvedManifest,
    segments: Vec<ResolvedSegment>,
    normalized_context: Vec<u8>,
}

impl Composition {
    /// Returns the normalized manifest.
    #[must_use]
    pub const fn manifest(&self) -> &ResolvedManifest {
        &self.manifest
    }

    /// Returns exact source segments in deterministic composition order.
    #[must_use]
    pub fn segments(&self) -> &[ResolvedSegment] {
        &self.segments
    }

    /// Returns normalized context bytes.
    ///
    /// Adjacent source byte sequences are separated by exactly two line-feed
    /// bytes. Exact original bytes remain available through [`Self::segments`].
    #[must_use]
    pub fn normalized_context(&self) -> &[u8] {
        &self.normalized_context
    }
}

/// A deterministic composition failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompositionError {
    UnknownProfile(Identifier),
    MissingOverlay(Identifier),
    TooManyOverlays {
        count: usize,
        maximum: usize,
    },
    IncompatibleOverlays {
        overlay: Identifier,
        other: Identifier,
    },
    OverlayTooLarge {
        overlay: Identifier,
        size: usize,
        maximum: usize,
    },
    OutputTooLarge {
        size: usize,
        maximum: usize,
    },
    Source(SourceFailure),
}

impl fmt::Display for CompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProfile(profile) => write!(formatter, "unknown profile `{profile}`"),
            Self::MissingOverlay(overlay) => {
                write!(formatter, "validated profile references missing overlay `{overlay}`")
            }
            Self::TooManyOverlays { count, maximum } => {
                write!(formatter, "selected {count} overlays; maximum is {maximum}")
            }
            Self::IncompatibleOverlays { overlay, other } => {
                write!(formatter, "overlay `{overlay}` is incompatible with `{other}`")
            }
            Self::OverlayTooLarge {
                overlay,
                size,
                maximum,
            } => write!(
                formatter,
                "overlay `{overlay}` contains {size} bytes; maximum is {maximum}"
            ),
            Self::OutputTooLarge { size, maximum } => {
                write!(formatter, "normalized output requires {size} bytes; maximum is {maximum}")
            }
            Self::Source(failure) => write!(formatter, "overlay source was rejected: {}", failure.kind),
        }
    }
}

impl std::error::Error for CompositionError {}

/// Resolves one profile and loads its overlays in deterministic class order.
///
/// Overlay order within a class is the explicit order declared by the profile.
/// Compatibility checks and source reads use that same stable order. The use
/// case performs no network, filesystem, process, environment, clock, or random
/// access; all source bytes arrive through [`OverlaySource`].
///
/// # Errors
///
/// Returns [`CompositionError`] when the profile is unknown, selected overlays
/// conflict, a configured limit is exceeded, the validated aggregate is
/// inconsistent, or the source adapter rejects an input.
pub fn compose(
    pack: &OverlayPack,
    profile_id: &Identifier,
    source: &impl OverlaySource,
    limits: CompositionLimits,
) -> Result<Composition, CompositionError> {
    let profile = pack
        .profiles()
        .iter()
        .find(|profile| &profile.id == profile_id)
        .ok_or_else(|| CompositionError::UnknownProfile(profile_id.clone()))?;

    let mut selected: Vec<&Overlay> = Vec::new();
    for class in pack.classes() {
        if let Some(overlay_ids) = profile.selections.get(&class.id) {
            for overlay_id in overlay_ids {
                let overlay = pack
                    .overlays()
                    .iter()
                    .find(|overlay| &overlay.id == overlay_id)
                    .ok_or_else(|| CompositionError::MissingOverlay(overlay_id.clone()))?;
                selected.push(overlay);
            }
        }
    }

    if selected.len() > limits.maximum_overlays {
        return Err(CompositionError::TooManyOverlays {
            count: selected.len(),
            maximum: limits.maximum_overlays,
        });
    }

    for overlay in &selected {
        for other in &selected {
            if overlay.incompatible_with.contains(&other.id) {
                return Err(CompositionError::IncompatibleOverlays {
                    overlay: overlay.id.clone(),
                    other: other.id.clone(),
                });
            }
        }
    }

    let mut entries = Vec::with_capacity(selected.len());
    let mut segments = Vec::with_capacity(selected.len());
    let mut normalized_context = Vec::new();
    let mut source_bytes = 0_usize;

    for overlay in selected {
        let bytes = source
            .load(&overlay.source, limits.maximum_overlay_bytes)
            .map_err(CompositionError::Source)?;
        if bytes.len() > limits.maximum_overlay_bytes {
            return Err(CompositionError::OverlayTooLarge {
                overlay: overlay.id.clone(),
                size: bytes.len(),
                maximum: limits.maximum_overlay_bytes,
            });
        }

        source_bytes = source_bytes
            .checked_add(bytes.len())
            .ok_or(CompositionError::OutputTooLarge {
                size: usize::MAX,
                maximum: limits.maximum_output_bytes,
            })?;
        let separator_bytes = usize::from(!entries.is_empty()) * 2;
        let required = normalized_context
            .len()
            .checked_add(separator_bytes)
            .and_then(|size| size.checked_add(bytes.len()))
            .ok_or(CompositionError::OutputTooLarge {
                size: usize::MAX,
                maximum: limits.maximum_output_bytes,
            })?;
        if required > limits.maximum_output_bytes {
            return Err(CompositionError::OutputTooLarge {
                size: required,
                maximum: limits.maximum_output_bytes,
            });
        }

        if separator_bytes != 0 {
            normalized_context.extend_from_slice(b"\n\n");
        }
        normalized_context.extend_from_slice(&bytes);

        entries.push(ResolvedEntry {
            class: overlay.class.clone(),
            overlay: overlay.id.clone(),
            source: overlay.source.clone(),
            byte_length: bytes.len(),
        });
        segments.push(ResolvedSegment {
            class: overlay.class.clone(),
            overlay: overlay.id.clone(),
            source: overlay.source.clone(),
            bytes,
        });
    }

    let manifest = ResolvedManifest {
        pack: pack.id.clone(),
        schema_family: pack.schema_family.clone(),
        profile: profile.id.clone(),
        entries,
        source_bytes,
        output_bytes: normalized_context.len(),
    };

    Ok(Composition {
        manifest,
        segments,
        normalized_context,
    })
}
