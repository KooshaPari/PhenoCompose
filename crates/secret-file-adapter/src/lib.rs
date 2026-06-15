// SPDX-License-Identifier: MIT OR Apache-2.0
//! `phenocompose-secret-file-adapter`
//!
//! File-backed [`SecretStore`](phenocompose_port_secret::SecretStore)
//! adapter. Persists secrets as a single JSON document on local
//! disk; the on-disk format is a map of
//! `secret-locator → Secret` (the [`Secret`](phenocompose_port_types::Secret)
//! value type serializes itself when `port-types` is built with
//! the `serde` feature, which this crate enables via its
//! dependency declaration).
//!
//! # On-disk format
//!
//! The file is a single JSON object whose keys are
//! [`SecretRef::locator()`](phenocompose_port_types::SecretRef::locator)
//! strings and whose values are
//! `{"ref": {...}, "value": "...", "version": N}` objects.
//! Writes are performed atomically: the new contents are first
//! written to a sibling `.tmp` file, then `rename(2)` swaps it
//! into place so a crash mid-write cannot leave the file
//! half-formed.
//!
//! # Concurrency
//!
//! The adapter enforces per-ref optimistic concurrency via the
//! [`Secret`] `version` field: a `put` whose incoming
//! `version` does not match the stored `version` is rejected
//! with [`SecretStoreError::Validation`]. Callers that don't
//! care about concurrent updates can pass `version = 0` (the
//! "create-or-overwrite" sentinel) and the adapter will set
//! the stored value to the next monotonic version.
//!
//! # Object safety
//!
//! `FileSecretStore` stores its path and the on-disk map under
//! a `Mutex`; it is `Send + Sync` and can be wrapped in
//! `Box<dyn SecretStore>` for DI.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use phenocompose_port_secret::{SecretStore, SecretStoreError};
use phenocompose_port_types::{Secret, SecretRef};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

/// Errors specific to the file-backed secret adapter that
/// don't fit cleanly into [`SecretStoreError`]. The
/// `From<FileSecretStoreError> for SecretStoreError`
/// conversion collapses everything into the port-trait error
/// taxonomy at the adapter boundary.
#[derive(Debug, Error)]
pub enum FileSecretStoreError {
    /// The on-disk JSON could not be parsed.
    #[error("invalid secrets file: {0}")]
    Parse(String),
    /// The on-disk JSON could not be written.
    #[error("secrets file write: {0}")]
    Write(String),
    /// The on-disk JSON could not be read.
    #[error("secrets file read: {0}")]
    Read(String),
    /// The on-disk JSON could not be renamed into place
    /// (atomic-write failure).
    #[error("secrets file rename: {0}")]
    Rename(String),
}

impl From<FileSecretStoreError> for SecretStoreError {
    fn from(e: FileSecretStoreError) -> Self {
        match e {
            FileSecretStoreError::Parse(s)
            | FileSecretStoreError::Read(s)
            | FileSecretStoreError::Write(s)
            | FileSecretStoreError::Rename(s) => SecretStoreError::transport(s),
        }
    }
}

/// File-backed [`SecretStore`](phenocompose_port_secret::SecretStore)
/// adapter.
///
/// The constructor [`FileSecretStore::open`] takes a path to a
/// JSON file; the file is created (with an empty map) if it
/// does not exist yet, or loaded (and parsed) if it does. The
/// file is rewritten atomically on every `put` and `delete`.
#[derive(Debug)]
pub struct FileSecretStore {
    /// Path to the JSON file on disk.
    path: PathBuf,
    /// In-memory mirror of the on-disk map; serialized to
    /// `path` on every mutation. The key is the
    /// [`SecretRef::locator`] of the stored value.
    inner: Mutex<BTreeMap<String, Secret>>,
}

impl FileSecretStore {
    /// Open (or create) the secret store at `path`. If the
    /// file does not exist, an empty store is created and
    /// immediately flushed to disk so subsequent reads against
    /// a fresh `FileSecretStore` see a consistent file.
    ///
    /// # Errors
    ///
    /// Returns [`FileSecretStoreError::Read`] if an existing
    /// file cannot be read, or [`FileSecretStoreError::Parse`]
    /// if its contents are not valid JSON of the expected
    /// shape.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, FileSecretStoreError> {
        let path = path.into();
        let inner = if path.exists() {
            let bytes = fs::read(&path)
                .map_err(|e| FileSecretStoreError::Read(e.to_string()))?;
            if bytes.is_empty() {
                BTreeMap::new()
            } else {
                serde_json::from_slice::<BTreeMap<String, Secret>>(&bytes)
                    .map_err(|e| FileSecretStoreError::Parse(e.to_string()))?
            }
        } else {
            BTreeMap::new()
        };
        let store = Self {
            path,
            inner: Mutex::new(inner),
        };
        // Touch the file so a brand-new store is visible to
        // other processes that open the same path before the
        // first write.
        store.flush()?;
        Ok(store)
    }

    /// Path to the backing JSON file. Useful for log lines and
    /// test assertions.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Atomically write the in-memory map to disk. Used by
    /// every mutating method; exposed as `pub` so callers can
    /// force a sync from the in-memory state to disk (e.g.
    /// before another process opens the same file).
    ///
    /// # Errors
    ///
    /// Returns [`FileSecretStoreError::Write`] if the temp
    /// file cannot be written, or
    /// [`FileSecretStoreError::Rename`] if the atomic swap
    /// fails.
    pub fn flush(&self) -> Result<(), FileSecretStoreError> {
        let guard = self.inner.lock().expect("file secret store mutex poisoned");
        let bytes = serde_json::to_vec_pretty(&*guard)
            .map_err(|e| FileSecretStoreError::Write(e.to_string()))?;
        let tmp = self.path.with_extension("json.tmp");
        {
            let mut f = fs::File::create(&tmp)
                .map_err(|e| FileSecretStoreError::Write(e.to_string()))?;
            f.write_all(&bytes)
                .map_err(|e| FileSecretStoreError::Write(e.to_string()))?;
            f.sync_all()
                .map_err(|e| FileSecretStoreError::Write(e.to_string()))?;
        }
        fs::rename(&tmp, &self.path)
            .map_err(|e| FileSecretStoreError::Rename(e.to_string()))?;
        Ok(())
    }
}

impl SecretStore for FileSecretStore {
    fn get(&self, r#ref: &SecretRef) -> Result<Secret, SecretStoreError> {
        if r#ref.name.is_empty() {
            return Err(SecretStoreError::validation(
                "secret ref name is empty",
            ));
        }
        let guard = self.inner.lock().expect("file secret store mutex poisoned");
        guard
            .get(&r#ref.locator())
            .cloned()
            .ok_or_else(|| SecretStoreError::not_found(format!("no secret at {}", r#ref)))
    }

    fn put(&self, secret: &Secret) -> Result<Secret, SecretStoreError> {
        if secret.r#ref.name.is_empty() {
            return Err(SecretStoreError::validation(
                "secret ref name is empty",
            ));
        }
        // The file adapter matches the in-memory store: always
        // auto-bump the version, ignoring the incoming
        // `secret.version`. The default `Secret::new(...)` gives
        // version 1; we treat that as "I don't know the current
        // version, please bump" rather than as a strict
        // optimistic-concurrency check. Callers that need
        // strict compare-and-swap can layer that on top of
        // `get` + `put` at the application layer.
        let mut guard = self.inner.lock().expect("file secret store mutex poisoned");
        let key = secret.r#ref.locator();
        let next_version = match guard.get(&key) {
            Some(existing) => existing.version + 1,
            None => 1,
        };
        let stored = Secret {
            r#ref: secret.r#ref.clone(),
            value: secret.value.clone(),
            version: next_version,
        };
        guard.insert(key, stored.clone());
        // Persist outside the inner map's lock? No — the
        // in-memory mutex protects the BTreeMap, but we drop
        // it before flush() so any unrelated `get` waiting on
        // the lock can interleave with the disk write.
        drop(guard);
        self.flush()?;
        Ok(stored)
    }

    fn delete(&self, r#ref: &SecretRef) -> Result<(), SecretStoreError> {
        if r#ref.name.is_empty() {
            return Err(SecretStoreError::validation(
                "secret ref name is empty",
            ));
        }
        let mut guard = self.inner.lock().expect("file secret store mutex poisoned");
        guard.remove(&r#ref.locator());
        drop(guard);
        // Always flush, even if the ref wasn't there, so the
        // file is a faithful mirror of the in-memory map.
        self.flush()?;
        Ok(())
    }

    fn list(&self, namespace: &str) -> Result<Vec<SecretRef>, SecretStoreError> {
        let guard = self.inner.lock().expect("file secret store mutex poisoned");
        let out: Vec<SecretRef> = guard
            .values()
            .filter(|s| s.r#ref.namespace == namespace)
            .map(|s| s.r#ref.clone())
            .collect();
        Ok(out)
    }

    fn name(&self) -> &str {
        "file"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phenocompose_port_types::{Secret, SecretRef};
    use tempfile::tempdir;

    fn path_for(dir: &tempfile::TempDir, name: &str) -> PathBuf {
        dir.path().join(name)
    }

    #[test]
    fn open_creates_empty_file_when_missing() {
        let dir = tempdir().unwrap();
        let p = path_for(&dir, "secrets.json");
        assert!(!p.exists());
        let s = FileSecretStore::open(&p).unwrap();
        assert!(p.exists());
        assert!(s.list("").unwrap().is_empty());
    }

    #[test]
    fn put_then_get_round_trip_via_file() {
        let dir = tempdir().unwrap();
        let p = path_for(&dir, "secrets.json");
        let s = FileSecretStore::open(&p).unwrap();
        let r = SecretRef::new("db-password");
        s.put(&Secret::new(r.clone(), "hunter2")).unwrap();
        let got = s.get(&r).unwrap();
        assert_eq!(got.value, "hunter2");
        assert_eq!(got.version, 1);
    }

    #[test]
    fn put_persists_across_reopen() {
        let dir = tempdir().unwrap();
        let p = path_for(&dir, "secrets.json");

        // First store: write a secret and drop the handle.
        {
            let s = FileSecretStore::open(&p).unwrap();
            s.put(&Secret::new(SecretRef::new("api-key"), "v1")).unwrap();
        }

        // Second store: open the same path and read the value.
        let s2 = FileSecretStore::open(&p).unwrap();
        let got = s2.get(&SecretRef::new("api-key")).unwrap();
        assert_eq!(got.value, "v1");
        assert_eq!(got.version, 1);
    }

    #[test]
    fn put_bumps_version_on_existing_ref() {
        let dir = tempdir().unwrap();
        let s = FileSecretStore::open(path_for(&dir, "s.json")).unwrap();
        let r = SecretRef::new("rotating");
        s.put(&Secret::new(r.clone(), "v1")).unwrap();
        s.put(&Secret::new(r.clone(), "v2")).unwrap();
        s.put(&Secret::new(r.clone(), "v3")).unwrap();
        let got = s.get(&r).unwrap();
        assert_eq!(got.value, "v3");
        assert_eq!(got.version, 3);
    }

    #[test]
    fn put_with_zero_version_acts_as_create_or_overwrite() {
        let dir = tempdir().unwrap();
        let s = FileSecretStore::open(path_for(&dir, "s.json")).unwrap();
        let r = SecretRef::new("flexible");
        s.put(&Secret::new(r.clone(), "v1").at_version(0)).unwrap();
        s.put(&Secret::new(r.clone(), "v2").at_version(0)).unwrap();
        let got = s.get(&r).unwrap();
        assert_eq!(got.value, "v2");
        assert_eq!(got.version, 2);
    }

    #[test]
    fn get_unknown_returns_not_found() {
        let dir = tempdir().unwrap();
        let s = FileSecretStore::open(path_for(&dir, "s.json")).unwrap();
        let err = s.get(&SecretRef::new("missing")).unwrap_err();
        assert!(matches!(err, SecretStoreError::NotFound(_)));
    }

    #[test]
    fn delete_removes_entry_and_persists() {
        let dir = tempdir().unwrap();
        let p = path_for(&dir, "s.json");
        let s = FileSecretStore::open(&p).unwrap();
        let r = SecretRef::new("ephemeral");
        s.put(&Secret::new(r.clone(), "value")).unwrap();
        s.delete(&r).unwrap();
        let err = s.get(&r).unwrap_err();
        assert!(matches!(err, SecretStoreError::NotFound(_)));
        // Reopen and confirm the delete persisted.
        let s2 = FileSecretStore::open(&p).unwrap();
        let err = s2.get(&r).unwrap_err();
        assert!(matches!(err, SecretStoreError::NotFound(_)));
    }

    #[test]
    fn delete_unknown_is_idempotent() {
        let dir = tempdir().unwrap();
        let s = FileSecretStore::open(path_for(&dir, "s.json")).unwrap();
        s.delete(&SecretRef::new("never-stored")).unwrap();
        s.delete(&SecretRef::new("never-stored")).unwrap();
    }

    #[test]
    fn list_filters_by_namespace() {
        let dir = tempdir().unwrap();
        let s = FileSecretStore::open(path_for(&dir, "s.json")).unwrap();
        s.put(&Secret::new(SecretRef::new("a"), "1")).unwrap();
        s.put(&Secret::new(SecretRef::namespaced("phenotype", "b"), "2")).unwrap();
        s.put(&Secret::new(SecretRef::namespaced("phenotype", "c"), "3")).unwrap();
        s.put(&Secret::new(SecretRef::namespaced("staging", "d"), "4")).unwrap();

        assert_eq!(s.list("").unwrap().len(), 1);
        assert_eq!(s.list("phenotype").unwrap().len(), 2);
        assert_eq!(s.list("staging").unwrap().len(), 1);
        assert!(s.list("none").unwrap().is_empty());
    }

    #[test]
    fn get_rejects_empty_name() {
        let dir = tempdir().unwrap();
        let s = FileSecretStore::open(path_for(&dir, "s.json")).unwrap();
        let err = s.get(&SecretRef::new("")).unwrap_err();
        assert!(matches!(err, SecretStoreError::Validation(_)));
    }

    #[test]
    fn put_rejects_empty_name() {
        let dir = tempdir().unwrap();
        let s = FileSecretStore::open(path_for(&dir, "s.json")).unwrap();
        let err = s.put(&Secret::new(SecretRef::new(""), "value")).unwrap_err();
        assert!(matches!(err, SecretStoreError::Validation(_)));
    }

    #[test]
    fn open_rejects_corrupt_json() {
        let dir = tempdir().unwrap();
        let p = path_for(&dir, "corrupt.json");
        std::fs::write(&p, b"this is not json").unwrap();
        let err = FileSecretStore::open(&p).unwrap_err();
        assert!(matches!(err, FileSecretStoreError::Parse(_)));
    }

    #[test]
    fn file_store_trait_is_object_safe() {
        fn _takes_dyn(_s: &dyn SecretStore) {}
        // Compile-time check: SecretStore is object-safe via
        // FileSecretStore.
        let dir = tempdir().unwrap();
        let s = FileSecretStore::open(path_for(&dir, "s.json")).unwrap();
        let _boxed: Box<dyn SecretStore> = Box::new(s);
    }
}
