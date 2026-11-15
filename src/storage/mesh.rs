//! Account-owned mesh identity; never part of original-data bundles.
//! @ref LLP 0008#multiple-accounts-on-one-machine
use super::sync::Replica;
use crate::error::{Error, Result};
use iroh::SecretKey;
use libsql::{TransactionBehavior, params};
use std::path::{Path, PathBuf};

mod control;

/// No Debug/Serialize: this value must never enter status output or diagnostics.
pub struct Identity {
    pub key: SecretKey,
    pub service: String,
    pub receipt_key: Option<String>,
    pub head: Option<String>,
    pub enabled: bool,
    pub pending_leave: bool,
    pub conflict: bool,
}

/// Preserve store overrides, but reject unsafe locations before opening libSQL
/// or generating/loading a network secret. Existing directories are not chmodded.
pub fn private_store(path: &Path) -> Result<PathBuf> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)?;
    let parent = std::fs::canonicalize(parent)?;
    let requested = parent.join(
        path.file_name()
            .ok_or_else(|| Error::Invalid("store must name a file".into()))?,
    );
    let resolved = if requested.exists() {
        std::fs::canonicalize(&requested)?
    } else {
        requested
    };
    let parent = resolved
        .parent()
        .ok_or_else(|| Error::Invalid("store has no parent".into()))?;
    let directory = std::fs::symlink_metadata(parent)?;
    let uid = unsafe { libc::geteuid() };
    if !directory.is_dir() || directory.uid() != uid || directory.mode() & 0o777 != 0o700 {
        return Err(Error::Invalid("mesh store parent must be account-owned and mode 0700; choose a private application directory".into()));
    }
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&resolved)
    {
        Ok(_) => (),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(error) => return Err(error.into()),
    }
    let file = std::fs::symlink_metadata(&resolved)?;
    if !file.is_file() || file.uid() != uid || file.mode() & 0o777 != 0o600 || file.nlink() != 1 {
        return Err(Error::Invalid(
            "mesh store must be an account-owned mode-0600 regular file with one link".into(),
        ));
    }
    // SQLite can retain the secret in journals/WAL; check any existing sidecars.
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut name = resolved.as_os_str().to_owned();
        name.push(suffix);
        match std::fs::symlink_metadata(Path::new(&name)) {
            Ok(sidecar)
                if !sidecar.is_file()
                    || sidecar.uid() != uid
                    || sidecar.mode() & 0o077 != 0
                    || sidecar.nlink() != 1 =>
            {
                return Err(Error::Invalid(
                    "mesh SQLite sidecar is not account-private".into(),
                ));
            }
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(resolved)
}

pub fn service_origin(value: &str) -> Result<String> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| Error::Invalid("invalid mesh service URL".into()))?;
    let local = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if (url.scheme() != "https" && !(url.scheme() == "http" && local))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || url.port() == Some(0)
    {
        return Err(Error::Invalid("mesh service must be an HTTPS origin (HTTP is allowed only on loopback for development)".into()));
    }
    Ok(url.origin().ascii_serialization())
}

impl Replica {
    pub async fn mesh_enrolled(&self) -> Result<bool> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = tx
            .query("SELECT 1 FROM mesh_identity WHERE singleton=1", ())
            .await?;
        Ok(rows.next().await?.is_some())
    }
    pub async fn open_private_mesh(path: &Path) -> Result<Self> {
        Self::open(&private_store(path)?).await
    }

    pub async fn mesh_identity(&self) -> Result<Option<Identity>> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = tx.query("SELECT secret,service,receipt_key,head,enabled,pending_leave,conflict FROM mesh_identity WHERE singleton=1", ()).await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let bytes: Vec<u8> = row.get(0)?;
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| Error::Invalid("corrupt mesh identity".into()))?;
        Ok(Some(Identity {
            key: SecretKey::from_bytes(&bytes),
            service: row.get(1)?,
            receipt_key: row.get(2)?,
            head: row.get(3)?,
            enabled: row.get::<i64>(4)? != 0,
            pending_leave: row.get::<i64>(5)? != 0,
            conflict: row.get::<i64>(6)? != 0,
        }))
    }

    pub async fn mesh_create_identity(&self, service: &str) -> Result<Identity> {
        private_store(&self.path)?;
        let service = service_origin(service)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut rows = tx
            .query("SELECT service FROM mesh_identity WHERE singleton=1", ())
            .await?;
        if let Some(row) = rows.next().await? {
            if row.get::<String>(0)? != service {
                return Err(Error::Invalid("this store is enrolled with another mesh service; reset identity explicitly before changing it".into()));
            }
        } else {
            let key = SecretKey::generate();
            tx.execute(
                "INSERT INTO mesh_identity(singleton,secret,service) VALUES(1,?,?)",
                params![key.to_bytes().to_vec(), service],
            )
            .await?;
        }
        drop(rows);
        tx.commit().await?;
        self.mesh_identity()
            .await?
            .ok_or_else(|| Error::Invalid("mesh identity was not persisted".into()))
    }

    /// Called only after network jobs have drained. Original records/tombstones
    /// and publication state are deliberately outside this transaction.
    pub async fn mesh_reset_identity(&self) -> Result<()> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        tx.execute_batch(
            "DELETE FROM mesh_pending; DELETE FROM mesh_roots; DELETE FROM mesh_event_ids; DELETE FROM mesh_successors; DELETE FROM mesh_membership_states; DELETE FROM mesh_events; DELETE FROM mesh_identity;",
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn identities_are_per_store_persistent_and_not_recreated_on_read() {
        let root = tempfile::tempdir().unwrap();
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let a = root.path().join("a.db");
        let b = root.path().join("b.db");
        let replica = Replica::open_private_mesh(&a).await.unwrap();
        assert!(replica.mesh_identity().await.unwrap().is_none());
        let first = replica
            .mesh_create_identity("https://example.test/")
            .await
            .unwrap();
        let again = replica
            .mesh_create_identity("https://example.test")
            .await
            .unwrap();
        assert_eq!(first.key.public(), again.key.public());
        assert!(!first.enabled);
        assert!(
            replica
                .mesh_create_identity("https://other.test")
                .await
                .is_err()
        );
        let other = Replica::open_private_mesh(&b)
            .await
            .unwrap()
            .mesh_create_identity("https://example.test")
            .await
            .unwrap();
        assert_ne!(first.key.public(), other.key.public());
        drop(replica);
        let replica = Replica::open_private_mesh(&a).await.unwrap();
        assert_eq!(
            first.key.public(),
            replica.mesh_identity().await.unwrap().unwrap().key.public()
        );
        replica.mesh_reset_identity().await.unwrap();
        assert!(replica.mesh_identity().await.unwrap().is_none());
        let reset = replica
            .mesh_create_identity("https://other.test")
            .await
            .unwrap();
        assert_ne!(first.key.public(), reset.key.public());
    }

    #[test]
    fn unsafe_existing_paths_fail_without_changing_permissions() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("store.db");
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(private_store(&path).is_err());
        assert!(!path.exists());
        assert_eq!(
            std::fs::metadata(root.path()).unwrap().permissions().mode() & 0o777,
            0o755
        );
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        private_store(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(private_store(&path).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let alias = root.path().join("alias.db");
        std::fs::hard_link(&path, &alias).unwrap();
        assert!(private_store(&path).is_err());
    }

    #[test]
    fn service_urls_are_origins_not_commands_or_credentials() {
        for bad in [
            "ssh://example.test",
            "http://example.test",
            "https://a:b@example.test",
            "https://example.test/path",
            "https://example.test/?token=x",
            "https://example.test/#x",
        ] {
            assert!(service_origin(bad).is_err(), "{bad}");
        }
        assert_eq!(
            service_origin("https://EXAMPLE.test:443/").unwrap(),
            "https://example.test"
        );
        assert_eq!(
            service_origin("http://127.0.0.1:8787").unwrap(),
            "http://127.0.0.1:8787"
        );
    }
}
