//! Unix runtime paths, private coordination and detached bootstrap.
//! @ref LLP 0007#stable-owner-directory
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub account: u32,
    #[serde(with = "crate::native_path")]
    pub path: PathBuf,
}
/// Resolve a routing path without opening or creating the database.
pub fn resolve_store(path: &Path) -> Result<PathBuf> {
    use std::{collections::VecDeque, ffi::OsString, path::Component};
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut pending = absolute
        .components()
        .map(|c| c.as_os_str().to_owned())
        .collect::<VecDeque<OsString>>();
    let mut resolved = PathBuf::from("/");
    let mut links = 0;
    while let Some(component) = pending.pop_front() {
        if component == "/" || component == "." {
            continue;
        }
        if component == ".." {
            resolved.pop();
            continue;
        }
        resolved.push(&component);
        match std::fs::symlink_metadata(&resolved) {
            Ok(meta) if meta.file_type().is_symlink() => {
                links += 1;
                if links > 40 {
                    return Err(Error::Invalid("too many store path symlinks".into()));
                }
                let target = std::fs::read_link(&resolved)?;
                resolved.pop();
                if target.is_absolute() {
                    resolved = PathBuf::from("/");
                }
                let components = target
                    .components()
                    .filter(|c| !matches!(c, Component::RootDir))
                    .map(|c| c.as_os_str().to_owned())
                    .collect::<Vec<_>>();
                for part in components.into_iter().rev() {
                    pending.push_front(part);
                }
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    if resolved.file_name().is_none() {
        return Err(Error::Invalid("store must name a database file".into()));
    }
    // Native canonicalization also restores filesystem spelling on macOS.
    // Canonicalize the existing prefix without opening/creating the database.
    let mut ancestor = resolved.as_path();
    let mut suffix = Vec::new();
    loop {
        match std::fs::canonicalize(ancestor) {
            Ok(mut canonical) => {
                canonical.extend(suffix.into_iter().rev());
                return Ok(canonical);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                suffix.push(ancestor.file_name().ok_or(error)?.to_owned());
                ancestor = ancestor.parent().unwrap();
            }
            Err(error) => return Err(error.into()),
        }
    }
}
pub fn identity(path: &Path) -> Result<Identity> {
    Ok(Identity {
        account: unsafe { libc::geteuid() },
        path: resolve_store(path)?,
    })
}
/// Physical identity is observed only by the daemon, around application work.
pub fn file_identity(path: &Path) -> Result<Option<(u64, u64)>> {
    let m = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if !m.is_file() {
        return Err(Error::Invalid("store must be a regular file".into()));
    }
    if m.nlink() != 1 {
        return Err(Error::Invalid(
            "hard-linked SQLite stores are unsupported; use one canonical database path".into(),
        ));
    }
    Ok(Some((m.dev(), m.ino())))
}
pub fn key(id: &Identity) -> Result<String> {
    let bytes = Sha256::digest(serde_json::to_vec(id)?);
    let mut out = String::new();
    let mut bits = 0u32;
    let mut n = 0;
    for b in &bytes[..16] {
        bits = (bits << 8) | u32::from(*b);
        n += 8;
        while n >= 5 {
            n -= 5;
            out.push(b"abcdefghijklmnopqrstuvwxyz234567"[((bits >> n) & 31) as usize] as char);
        }
    }
    if n > 0 {
        out.push(b"abcdefghijklmnopqrstuvwxyz234567"[((bits << (5 - n)) & 31) as usize] as char);
    }
    Ok(out)
}
pub fn owner_root(store: &Path) -> Result<PathBuf> {
    let path = store
        .parent()
        .ok_or_else(|| Error::Invalid("store has no parent".into()))?
        .join(format!(".linn-runtime-{}", unsafe { libc::geteuid() }));
    private_dir(&path, true)?;
    Ok(path)
}
pub fn socket_root() -> Result<PathBuf> {
    if let Some(root) = std::env::var_os("XDG_RUNTIME_DIR").filter(|s| !s.is_empty()) {
        let root = PathBuf::from(root);
        if !root.is_absolute() {
            return Err(Error::Invalid("XDG_RUNTIME_DIR must be absolute".into()));
        }
        private_dir(&root, false)?;
        let path = root.join("linn");
        private_dir(&path, true)?;
        return Ok(path);
    }
    #[cfg(target_os = "macos")]
    let root = {
        let len = unsafe { libc::confstr(libc::_CS_DARWIN_USER_TEMP_DIR, std::ptr::null_mut(), 0) };
        if len == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let mut buf = vec![0u8; len];
        unsafe { libc::confstr(libc::_CS_DARWIN_USER_TEMP_DIR, buf.as_mut_ptr().cast(), len) };
        buf.pop();
        PathBuf::from(std::ffi::OsStr::from_bytes(&buf)).join("linn")
    };
    #[cfg(not(target_os = "macos"))]
    let root = {
        eprintln!("linn: XDG_RUNTIME_DIR is unset; using private /tmp runtime directory");
        PathBuf::from(format!("/tmp/linn-{}", unsafe { libc::geteuid() }))
    };
    private_dir(&root, true)?;
    Ok(root)
}
pub fn private_dir(path: &Path, create: bool) -> Result<()> {
    if create {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)?;
    }
    let m = std::fs::symlink_metadata(path)?;
    if !m.is_dir() || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o777 != 0o700 {
        return Err(Error::Invalid(format!(
            "unsafe runtime directory: {} (requires account ownership and mode 0700)",
            path.display()
        )));
    }
    let file = File::open(path)?;
    let mut fs = std::mem::MaybeUninit::<libc::statfs>::uninit();
    if unsafe { libc::fstatfs(std::os::fd::AsRawFd::as_raw_fd(&file), fs.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let fs = unsafe { fs.assume_init() };
    #[cfg(target_os = "macos")]
    if fs.f_flags & (libc::MNT_LOCAL as u32) == 0 {
        return Err(Error::Invalid(
            "daemon coordination requires local storage".into(),
        ));
    }
    #[cfg(target_os = "linux")]
    if [0x6969u64, 0xff534d42, 0xfe534d42, 0x65735546].contains(&(fs.f_type as u64)) {
        return Err(Error::Invalid(
            "daemon coordination requires local storage".into(),
        ));
    }
    Ok(())
}
pub fn private_file(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let m = file.metadata()?;
    if !m.is_file() || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o777 != 0o600 {
        return Err(Error::Invalid("unsafe daemon coordination file".into()));
    }
    Ok(file)
}
pub fn check_socket(path: &Path) -> Result<()> {
    let len = path.as_os_str().as_bytes().len();
    let socket = std::mem::MaybeUninit::<libc::sockaddr_un>::zeroed();
    let capacity = unsafe { (*socket.as_ptr()).sun_path.len() };
    if len >= capacity {
        return Err(Error::Invalid(format!(
            "runtime socket path is too long: {}; choose a shorter XDG_RUNTIME_DIR",
            path.display()
        )));
    }
    Ok(())
}
pub fn detach() -> Result<()> {
    // Fresh executable bootstrap only, before creating threads or libSQL handles.
    #[cfg(target_os = "macos")]
    {
        let descriptors = open_descriptors().map_err(|error| {
            Error::Invalid(format!("cannot enumerate inherited descriptors: {error}"))
        })?;
        // This fresh bootstrap has no threads or concurrent descriptor opens.
        for fd in descriptors.into_iter().filter(|fd| *fd >= 3) {
            unsafe {
                libc::close(fd);
            }
        }
    }
    #[cfg(target_os = "linux")]
    unsafe {
        if libc::syscall(libc::SYS_close_range, 3u32, u32::MAX, 0u32) != 0 {
            let descriptors = open_descriptors().map_err(|error| {
                Error::Invalid(format!("cannot enumerate inherited descriptors: {error}"))
            })?;
            for fd in descriptors.into_iter().filter(|fd| *fd >= 3) {
                libc::close(fd);
            }
        }
    }
    daemonize::Daemonize::new()
        .umask(0o077)
        .working_directory("/")
        .start()
        .map_err(|e| Error::Invalid(format!("daemon detachment failed: {e}")))?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub(super) fn open_descriptors() -> std::io::Result<Vec<i32>> {
    // Enumerate actual descriptors, including those above a lowered soft limit.
    // Drop the directory handle before closing anything in the fresh bootstrap.
    std::fs::read_dir("/proc/self/fd")?
        .map(|entry| {
            entry?
                .file_name()
                .to_string_lossy()
                .parse::<i32>()
                .map_err(std::io::Error::other)
        })
        .collect()
}

// @ref LLP 0007#automatic-detached-lifecycle
#[cfg(target_os = "macos")]
pub(super) fn open_descriptors() -> std::io::Result<Vec<i32>> {
    let invalid = || std::io::Error::other("incomplete process descriptor list");
    let entry_size = std::mem::size_of::<libc::proc_fdinfo>();
    // libproc returns the required byte count, including spare entries. The
    // single-threaded bootstrap does not open descriptors between these calls.
    let size = unsafe {
        libc::proc_pidinfo(
            libc::getpid(),
            libc::PROC_PIDLISTFDS,
            0,
            std::ptr::null_mut(),
            0,
        )
    };
    if size <= 0 {
        return Err(std::io::Error::last_os_error());
    }
    let count = size as usize / entry_size;
    if count == 0 || !(size as usize).is_multiple_of(entry_size) {
        return Err(invalid());
    }
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(count)
        .map_err(std::io::Error::other)?;
    entries.resize(
        count,
        libc::proc_fdinfo {
            proc_fd: 0,
            proc_fdtype: 0,
        },
    );
    // The allocation is aligned for proc_fdinfo and covers exactly size bytes.
    let filled = unsafe {
        libc::proc_pidinfo(
            libc::getpid(),
            libc::PROC_PIDLISTFDS,
            0,
            entries.as_mut_ptr().cast(),
            size,
        )
    };
    if filled <= 0 || filled >= size || !(filled as usize).is_multiple_of(entry_size) {
        return Err(invalid());
    }
    entries.truncate(filled as usize / entry_size);
    if entries.iter().any(|entry| entry.proc_fd < 0) {
        return Err(invalid());
    }
    Ok(entries.into_iter().map(|entry| entry.proc_fd).collect())
}
