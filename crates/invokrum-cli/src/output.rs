use std::fmt;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutputError {
    InvalidPath,
    ParentTraversal,
    ParentSymbolicLink,
    ParentNotDirectory,
    ParentChanged,
    TargetExists,
    TargetSymbolicLink,
    TargetNotRegular,
    UnsupportedPlatform,
    Io,
}

impl fmt::Display for OutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPath => "output path must name a UTF-8 file below an existing directory",
            Self::ParentTraversal => "output path must not contain parent-directory traversal",
            Self::ParentSymbolicLink => "output parent path must not contain symbolic links",
            Self::ParentNotDirectory => "output parent component is not a directory",
            Self::ParentChanged => "output parent changed during the write",
            Self::TargetExists => "output already exists; use --force for explicit replacement",
            Self::TargetSymbolicLink => "output target must not be a symbolic link",
            Self::TargetNotRegular => "output target must be a regular file",
            Self::UnsupportedPlatform => "safe output persistence is supported on Linux only",
            Self::Io => "output operation failed",
        })
    }
}

pub(crate) fn write_atomic(path: &Path, bytes: &[u8], force: bool) -> Result<(), OutputError> {
    write_platform(path, bytes, force)
}

#[cfg(target_os = "linux")]
fn write_platform(path: &Path, bytes: &[u8], force: bool) -> Result<(), OutputError> {
    use std::fs::{self, File, OpenOptions};
    use std::io::Write;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    use std::path::{Component, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or(OutputError::InvalidPath)?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    reject_parent_links(parent)?;
    let canonical_parent = fs::canonicalize(parent).map_err(|_| OutputError::Io)?;
    let pinned_parent = fs::symlink_metadata(&canonical_parent).map_err(|_| OutputError::Io)?;
    if pinned_parent.file_type().is_symlink() {
        return Err(OutputError::ParentSymbolicLink);
    }
    if !pinned_parent.is_dir() {
        return Err(OutputError::ParentNotDirectory);
    }

    let target = canonical_parent.join(file_name);
    check_target(&target, force)?;
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let temporary = canonical_parent.join(format!(
        ".{file_name}.invokrum-{}-{sequence}.tmp",
        std::process::id()
    ));
    let mut cleanup = TemporaryGuard::new(temporary.clone());
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|_| OutputError::Io)?;
    file.write_all(bytes).map_err(|_| OutputError::Io)?;
    file.sync_all().map_err(|_| OutputError::Io)?;
    drop(file);

    verify_parent_identity(&canonical_parent, &pinned_parent)?;
    if force {
        check_target(&target, true)?;
        fs::rename(&temporary, &target).map_err(|_| OutputError::Io)?;
    } else {
        match fs::hard_link(&temporary, &target) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(OutputError::TargetExists);
            }
            Err(_) => return Err(OutputError::Io),
        }
        fs::remove_file(&temporary).map_err(|_| OutputError::Io)?;
    }
    cleanup.disarm();

    let target_metadata = fs::symlink_metadata(&target).map_err(|_| OutputError::Io)?;
    if target_metadata.file_type().is_symlink() {
        return Err(OutputError::TargetSymbolicLink);
    }
    if !target_metadata.is_file() {
        return Err(OutputError::TargetNotRegular);
    }
    if target_metadata.mode() & 0o777 != 0o600 || target_metadata.nlink() != 1 {
        return Err(OutputError::Io);
    }
    verify_parent_identity(&canonical_parent, &pinned_parent)?;
    File::open(&canonical_parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| OutputError::Io)?;
    return Ok(());

    fn reject_parent_links(parent: &Path) -> Result<(), OutputError> {
        let mut current = if parent.is_absolute() {
            PathBuf::from("/")
        } else {
            std::env::current_dir().map_err(|_| OutputError::Io)?
        };
        for component in parent.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::ParentDir => return Err(OutputError::ParentTraversal),
                Component::Normal(segment) => {
                    current.push(segment);
                    let metadata = fs::symlink_metadata(&current).map_err(|_| OutputError::Io)?;
                    if metadata.file_type().is_symlink() {
                        return Err(OutputError::ParentSymbolicLink);
                    }
                    if !metadata.is_dir() {
                        return Err(OutputError::ParentNotDirectory);
                    }
                }
                Component::Prefix(_) => return Err(OutputError::InvalidPath),
            }
        }
        Ok(())
    }

    fn check_target(target: &Path, force: bool) -> Result<(), OutputError> {
        match fs::symlink_metadata(target) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(OutputError::TargetSymbolicLink);
                }
                if !metadata.is_file() {
                    return Err(OutputError::TargetNotRegular);
                }
                if !force {
                    return Err(OutputError::TargetExists);
                }
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(OutputError::Io),
        }
    }

    fn verify_parent_identity(parent: &Path, pinned: &fs::Metadata) -> Result<(), OutputError> {
        let current = fs::symlink_metadata(parent).map_err(|_| OutputError::ParentChanged)?;
        if current.file_type().is_symlink()
            || !current.is_dir()
            || current.dev() != pinned.dev()
            || current.ino() != pinned.ino()
        {
            Err(OutputError::ParentChanged)
        } else {
            Ok(())
        }
    }

    struct TemporaryGuard {
        path: Option<PathBuf>,
    }

    impl TemporaryGuard {
        fn new(path: PathBuf) -> Self {
            Self { path: Some(path) }
        }

        fn disarm(&mut self) {
            self.path = None;
        }
    }

    impl Drop for TemporaryGuard {
        fn drop(&mut self) {
            if let Some(path) = self.path.take() {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn write_platform(_path: &Path, _bytes: &[u8], _force: bool) -> Result<(), OutputError> {
    Err(OutputError::UnsupportedPlatform)
}
