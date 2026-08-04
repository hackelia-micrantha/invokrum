use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use invokrum_core::{CompositionLimits, Identifier, OverlayPack, OverlaySource, PackRelativePath};
use invokrum_fs::LocalPackSource;
use invokrum_host::{HOST_CONTRACT_VERSION, HostError, ResolvedBundle, resolve_bundle, verify_bundle};
use invokrum_integrity::{DriftKind, MAX_LOCKFILE_BYTES, decode_lockfile};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{Execution, EXIT_INPUT, EXIT_INTERNAL, EXIT_SUCCESS, EXIT_VALIDATION};

const MAX_REQUEST_BYTES: usize = 1_048_576;
const MAX_PACK_BYTES: usize = 1_048_576;
const MAX_JSON_DEPTH: usize = 32;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    protocol: String,
    request_id: String,
    #[serde(flatten)]
    operation: Operation,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
enum Operation {
    Capabilities,
    Resolve {
        pack: PathBuf,
        profile: String,
    },
    Verify {
        pack: PathBuf,
        profile: String,
        expected_lock_base64: String,
    },
}

#[derive(Clone, Copy, Debug)]
enum RpcErrorKind {
    Request,
    Input,
    Validation,
    Composition,
    Integrity,
    Internal,
}

impl RpcErrorKind {
    const fn code(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Input => "input",
            Self::Validation => "validation",
            Self::Composition => "composition",
            Self::Integrity => "integrity",
            Self::Internal => "internal",
        }
    }

    const fn exit_code(self) -> i32 {
        match self {
            Self::Request | Self::Input => EXIT_INPUT,
            Self::Validation | Self::Composition => EXIT_VALIDATION,
            Self::Integrity | Self::Internal => EXIT_INTERNAL,
        }
    }
}

#[derive(Debug)]
struct RpcError {
    kind: RpcErrorKind,
    message: String,
}

impl RpcError {
    fn new(kind: RpcErrorKind, message: impl ToString) -> Self {
        Self {
            kind,
            message: message.to_string(),
        }
    }
}

pub(crate) fn execute(stdin: &mut dyn Read) -> Execution {
    let request_bytes = match read_bounded(stdin, MAX_REQUEST_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => return error_execution(None, error),
    };
    if let Err(message) = validate_json_depth(&request_bytes, MAX_JSON_DEPTH) {
        return error_execution(None, RpcError::new(RpcErrorKind::Request, message));
    }
    let request = match serde_json::from_slice::<Request>(&request_bytes) {
        Ok(request) => request,
        Err(error) => {
            return error_execution(
                None,
                RpcError::new(RpcErrorKind::Request, bounded_parser_message(&error)),
            );
        }
    };
    let request_id = Some(request.request_id.as_str());
    if request.protocol != HOST_CONTRACT_VERSION {
        return error_execution(
            request_id,
            RpcError::new(
                RpcErrorKind::Request,
                format!(
                    "unsupported protocol `{}`; expected `{HOST_CONTRACT_VERSION}`",
                    request.protocol
                ),
            ),
        );
    }
    if request.request_id.is_empty() || request.request_id.len() > 128 {
        return error_execution(
            request_id,
            RpcError::new(
                RpcErrorKind::Request,
                "request_id must contain between 1 and 128 bytes",
            ),
        );
    }

    match execute_operation(request.operation) {
        Ok((operation, result)) => success_execution(&request.request_id, operation, result),
        Err(error) => error_execution(request_id, error),
    }
}

fn execute_operation(operation: Operation) -> Result<(&'static str, Value), RpcError> {
    match operation {
        Operation::Capabilities => Ok(("capabilities", capabilities())),
        Operation::Resolve { pack, profile } => {
            let (pack_value, source) = load_pack(&pack)?;
            let profile = parse_profile(&profile)?;
            let bundle = resolve_bundle(
                &pack_value,
                &profile,
                &source,
                CompositionLimits::default(),
            )
            .map_err(map_host_error)?;
            Ok(("resolve", bundle_json(&bundle)))
        }
        Operation::Verify {
            pack,
            profile,
            expected_lock_base64,
        } => {
            let lock_bytes = BASE64.decode(expected_lock_base64).map_err(|_| {
                RpcError::new(
                    RpcErrorKind::Request,
                    "expected_lock_base64 is not valid canonical base64",
                )
            })?;
            if lock_bytes.len() > MAX_LOCKFILE_BYTES {
                return Err(RpcError::new(
                    RpcErrorKind::Integrity,
                    "decoded expected lock exceeds the configured byte limit",
                ));
            }
            let expected = decode_lockfile(&lock_bytes)
                .map_err(|error| RpcError::new(RpcErrorKind::Integrity, error))?;
            let (pack_value, source) = load_pack(&pack)?;
            let profile = parse_profile(&profile)?;
            let verified = verify_bundle(
                &expected,
                &pack_value,
                &profile,
                &source,
                CompositionLimits::default(),
            )
            .map_err(map_host_error)?;
            Ok((
                "verify",
                json!({
                    "bundle": bundle_json(verified.current()),
                    "drifts": verified.report().drifts().iter().map(|drift| drift_json(*drift)).collect::<Vec<_>>(),
                    "verified": verified.is_verified(),
                }),
            ))
        }
    }
}

fn capabilities() -> Value {
    json!({
        "capabilities": ["capabilities", "resolve", "verify"],
        "default_limits": {
            "maximum_output_bytes": CompositionLimits::default().maximum_output_bytes(),
            "maximum_overlay_bytes": CompositionLimits::default().maximum_overlay_bytes(),
            "maximum_overlays": CompositionLimits::default().maximum_overlays(),
            "maximum_pack_bytes": MAX_PACK_BYTES,
            "maximum_request_bytes": MAX_REQUEST_BYTES,
        },
        "network_access": false,
        "persistent_writes": false,
        "runtime_invocation": false,
    })
}

fn bundle_json(bundle: &ResolvedBundle) -> Value {
    let manifest = bundle.manifest();
    json!({
        "context_base64": BASE64.encode(bundle.context()),
        "lock_base64": BASE64.encode(bundle.lock_bytes()),
        "manifest": {
            "entries": manifest.entries.iter().map(|entry| json!({
                "byte_length": entry.byte_length,
                "class": entry.class.to_string(),
                "overlay": entry.overlay.to_string(),
                "source": entry.source.as_str(),
            })).collect::<Vec<_>>(),
            "output_bytes": manifest.output_bytes,
            "pack": manifest.pack.to_string(),
            "profile": manifest.profile.to_string(),
            "schema": manifest.schema_family,
            "source_bytes": manifest.source_bytes,
        },
        "output_digest": bundle.lockfile().manifest.output.digest,
    })
}

fn load_pack(path: &Path) -> Result<(OverlayPack, LocalPackSource), RpcError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RpcError::new(RpcErrorKind::Input, "pack path must name a UTF-8 file"))?;
    let relative = PackRelativePath::parse(file_name.to_owned()).map_err(|_| {
        RpcError::new(
            RpcErrorKind::Input,
            "pack file name violates the portable path policy",
        )
    })?;
    let source = LocalPackSource::open(parent)
        .map_err(|error| RpcError::new(RpcErrorKind::Input, error))?;
    let bytes = source.load(&relative, MAX_PACK_BYTES).map_err(|error| {
        RpcError::new(
            RpcErrorKind::Input,
            format!(
                "pack input `{}` was rejected: {}",
                error.path.as_str(),
                error.kind
            ),
        )
    })?;
    let text = str::from_utf8(&bytes)
        .map_err(|_| RpcError::new(RpcErrorKind::Input, "pack document must be valid UTF-8"))?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let pack = match extension.as_str() {
        "json" => invokrum_schema::parse_json(text),
        "yaml" | "yml" => invokrum_schema::parse_yaml(text),
        _ if text.trim_start().starts_with('{') => invokrum_schema::parse_json(text),
        _ => invokrum_schema::parse_yaml(text),
    }
    .map_err(|error| RpcError::new(RpcErrorKind::Validation, error))?;
    Ok((pack, source))
}

fn parse_profile(value: &str) -> Result<Identifier, RpcError> {
    Identifier::parse(value.to_owned())
        .map_err(|_| RpcError::new(RpcErrorKind::Validation, "profile identifier is invalid"))
}

fn map_host_error(error: HostError) -> RpcError {
    match error {
        HostError::Composition(error) => RpcError::new(RpcErrorKind::Composition, error),
        HostError::Integrity(error) => RpcError::new(RpcErrorKind::Integrity, error),
    }
}

fn success_execution(request_id: &str, operation: &str, result: Value) -> Execution {
    Execution {
        code: EXIT_SUCCESS,
        stdout: encode_response(json!({
            "format": HOST_CONTRACT_VERSION,
            "ok": true,
            "operation": operation,
            "request_id": request_id,
            "result": result,
        })),
    }
}

fn error_execution(request_id: Option<&str>, error: RpcError) -> Execution {
    Execution {
        code: error.kind.exit_code(),
        stdout: encode_response(json!({
            "error": {
                "code": error.kind.code(),
                "message": error.message,
            },
            "format": HOST_CONTRACT_VERSION,
            "ok": false,
            "request_id": request_id,
        })),
    }
}

fn encode_response(value: Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(&value).unwrap_or_else(|_| {
        br#"{"error":{"code":"internal","message":"failed to encode response"},"format":"invokrum.host/v1","ok":false,"request_id":null}"#.to_vec()
    });
    bytes.push(b'\n');
    bytes
}

fn read_bounded(reader: &mut dyn Read, maximum_bytes: usize) -> Result<Vec<u8>, RpcError> {
    let mut bytes = Vec::new();
    reader
        .take((maximum_bytes as u64) + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| RpcError::new(RpcErrorKind::Input, "failed to read request from stdin"))?;
    if bytes.len() > maximum_bytes {
        return Err(RpcError::new(
            RpcErrorKind::Request,
            "request exceeds the configured byte limit",
        ));
    }
    if bytes.is_empty() {
        return Err(RpcError::new(RpcErrorKind::Request, "request is empty"));
    }
    Ok(bytes)
}

fn validate_json_depth(bytes: &[u8], maximum_depth: usize) -> Result<(), &'static str> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match *byte {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth = depth.saturating_add(1);
                if depth > maximum_depth {
                    return Err("request exceeds the configured JSON depth limit");
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

fn bounded_parser_message(error: &serde_json::Error) -> String {
    let message = error.to_string();
    message.chars().take(256).collect()
}

fn drift_json(drift: DriftKind) -> Value {
    match drift {
        DriftKind::PackMetadata => json!({ "kind": "pack_metadata" }),
        DriftKind::ProfileSelection => json!({ "kind": "profile_selection" }),
        DriftKind::OverlaySet => json!({ "kind": "overlay_set" }),
        DriftKind::OverlayContent { index } => {
            json!({ "index": index, "kind": "overlay_content" })
        }
        DriftKind::RenderedOutput => json!({ "kind": "rendered_output" }),
    }
}
