# Deterministic composition and local filesystem contract

## Composition ownership

`invokrum-core` owns the composition use case and the narrow `OverlaySource` port. It resolves a validated profile without filesystem, network, process, environment, clock, randomness, or serialization access.

Resolution order is:

1. classes by validated numeric `order`;
2. selected overlays within each class in explicit profile order.

The use case rejects an unknown profile, an inconsistent aggregate reference, selected incompatibilities, excessive overlay counts, oversized source bytes, and excessive normalized output. Compatibility is directional: if either selected overlay declares the other incompatible, composition fails when that declaration is encountered in deterministic composition order. Self-incompatibility also fails.

Overlay prose never determines structural authority. The result retains ordered `ResolvedSegment` values containing exact source bytes and separate class, overlay, and source identities. `normalized_context` joins adjacent segments with exactly two LF bytes; even empty segments retain a separator boundary. Hashes and persistent manifest formats remain part of issue #5.

Default limits are:

- 256 selected overlays;
- 1 MiB per overlay source;
- 8 MiB normalized output.

Hosts may choose stricter explicit limits.

## Source port failure contract

Infrastructure adapters return stable `SourceFailureKind` categories and do not pass host parser or operating-system error strings into application diagnostics. The validated source path remains available as structured data, but human `CompositionError` text deliberately does not echo it. Delivery layers remain responsible for any later path presentation and must escape control characters.

## Linux local filesystem adapter

`invokrum-fs::LocalPackSource` is the v0.1 concrete adapter. It supports Linux only and fails closed elsewhere.

At root establishment it requires:

- the selected root itself is not a symbolic link;
- the selected root is a directory;
- the root can be canonicalized;
- a root filesystem device identity can be recorded.

For each overlay it:

1. walks every declared path component with `symlink_metadata`;
2. rejects symbolic links at every component;
3. rejects intermediate non-directories;
4. rejects a device change from the established root, including ordinary mount-point crossings;
5. canonicalizes the candidate and verifies component-aware containment below the root;
6. requires a regular file with exactly one hard link;
7. opens the file and verifies that the opened device/inode matches the pre-open candidate;
8. resolves `/proc/self/fd/<fd>` and verifies the opened target remains below the root;
9. reads at most the configured file limit plus one byte;
10. compares device, inode, size, modification time, and change time before and after reading;
11. verifies the current path still names the same non-symlink inode.

The adapter returns the bytes from that one opened file. Composition, later hashing, and manifests must use those returned bytes rather than reopening the path.

## Supported-platform boundary

Windows junction and reparse-point behavior is not approximated. Non-Linux platforms receive `UnsupportedPlatform` until an adapter can enforce an equivalent contract with platform-native no-follow and identity primitives.

## Host preconditions and residual risk

The adapter cannot prove a trustworthy filesystem namespace against a privileged actor that can remap mounts during the operation. The host must provide a stable mount namespace and protect the selected pack root from concurrent privileged namespace manipulation.

A hostile kernel, `/proc` implementation, filesystem, or privileged mount-namespace controller remains outside the v0.1 guarantee. Filesystems whose metadata does not provide stable Linux device/inode/time semantics may fail closed or require a future specialized adapter.

No network acquisition occurs in composition or `invokrum-fs`. Pack acquisition, publisher authentication, quarantine, and update policy remain host responsibilities.
