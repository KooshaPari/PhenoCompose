//! `phenocompose-port-types`
//!
//! Shared value types that flow across the three PhenoCompose port
//! traits (Composer, Publisher, Runtime). Defined as a standalone
//! crate so the three port-trait crates can depend on a single
//! canonical type vocabulary without pulling in each other's
//! implementation code or test fixtures.
//!
//! Type inventory:
//!
//! | Type               | Role                                                 |
//! |--------------------|------------------------------------------------------|
//! | [`Manifest`]       | Input to the [`Composer`](crate::Composer) port      |
//! | [`ComposedArtifact`] | Output of `Composer`; input to [`Publisher`](crate::Publisher) |
//! | [`PublishTarget`]  | Where a [`ComposedArtifact`] is sent                 |
//! | [`PublishReceipt`] | Proof of a successful publish                        |
//! | [`ImageRef`]       | Reference to a container image; input to [`Runtime`](crate::Runtime) |
//! | [`ContainerId`]    | Opaque handle returned by `Runtime::spawn`           |
//! | [`ContainerStatus`] | State reported by `Runtime::status`                  |
//!
//! All types in this crate are `Send + Sync` so they can be moved
//! across worker threads and stored in `Box<dyn Trait>` adapters
//! that downstream pheno-* services compose into their dependency
//! graph.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use thiserror::Error;

/// A composition request — describes *what* the
/// [`Composer`](crate::Composer) should produce.
///
/// `Manifest` is intentionally transport-agnostic (no file paths,
/// no URIs, no environment variables). Adapters that need to
/// resolve files or secrets pull those out of the manifest into
/// the local adapter implementation; the port type stays small.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Manifest {
    /// Human-readable name (e.g. `"phenocommand-web"`). Used by
    /// the Composer for log lines and by adapters as a default
    /// artifact name when [`Manifest::artifact_name`] is `None`.
    pub name: String,
    /// Optional explicit artifact name. If `Some`, the
    /// [`Composer`](crate::Composer) MUST use this exact string
    /// as the artifact identifier; if `None`, the composer
    /// derives one from [`Manifest::name`].
    pub artifact_name: Option<String>,
    /// Free-form key/value tags (e.g. `version=0.1.0`,
    /// `channel=stable`). Adapters MUST preserve them on the
    /// resulting [`ComposedArtifact::tags`].
    pub tags: Vec<(String, String)>,
}

impl Manifest {
    /// Construct a manifest with the given name and no explicit
    /// artifact name or tags.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            artifact_name: None,
            tags: Vec::new(),
        }
    }

    /// Builder-style setter for [`Manifest::artifact_name`].
    #[must_use]
    pub fn with_artifact_name(mut self, name: impl Into<String>) -> Self {
        self.artifact_name = Some(name.into());
        self
    }

    /// Builder-style setter for [`Manifest::tags`].
    #[must_use]
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.push((key.into(), value.into()));
        self
    }
}

/// The output of a [`Composer`](crate::Composer) — an artifact
/// ready to be [`Publisher::publish`](crate::Publisher::publish)ed
/// or [`Runtime::spawn`](crate::Runtime::spawn)ed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComposedArtifact {
    /// Stable artifact identifier (typically
    /// `<name>:<tag-digest>`).
    pub id: String,
    /// Image reference for the artifact (consumable by
    /// [`Runtime::spawn`](crate::Runtime::spawn)).
    pub image: ImageRef,
    /// Tags copied from the source [`Manifest::tags`] plus any
    /// new tags the composer wants to attach
    /// (e.g. `content-digest=sha256:...`).
    pub tags: Vec<(String, String)>,
}

impl ComposedArtifact {
    /// Construct an artifact from an id and image ref, with no
    /// tags.
    pub fn new(id: impl Into<String>, image: ImageRef) -> Self {
        Self {
            id: id.into(),
            image,
            tags: Vec::new(),
        }
    }

    /// Builder-style setter for [`ComposedArtifact::tags`].
    #[must_use]
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.push((key.into(), value.into()));
        self
    }

    /// Look up a tag by key. Returns `None` if the key is not
    /// present.
    pub fn tag(&self, key: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// A destination for a [`ComposedArtifact`] — opaque to the port
/// trait; interpreted by the concrete [`Publisher`](crate::Publisher)
/// adapter (e.g. a registry host, a local file path, a Kafka
/// topic, a `std::io::Write` sink).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublishTarget {
    /// Transport identifier (e.g. `"docker-registry"`, `"file"`,
    /// `"kafka"`). Adapters dispatch on this.
    pub kind: String,
    /// Transport-specific locator (e.g.
    /// `"registry.phenotype.internal/phenocommand/web:0.1.0"`,
    /// `"/var/lib/phenocompose/artifacts/phenocommand-web.tar"`,
    /// `"phenocommand-artifacts"`).
    pub locator: String,
}

impl PublishTarget {
    /// Construct a publish target with the given kind and
    /// locator.
    pub fn new(kind: impl Into<String>, locator: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            locator: locator.into(),
        }
    }
}

/// Proof of a successful publish — returned by
/// [`Publisher::publish`](crate::Publisher::publish).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublishReceipt {
    /// The artifact id that was published (mirrors
    /// [`ComposedArtifact::id`]).
    pub artifact_id: String,
    /// The destination that received the publish (mirrors
    /// [`PublishTarget`] but value-equal).
    pub target: PublishTarget,
    /// Adapter-defined publication locator (e.g. a digest on the
    /// remote side, a tarball path, a Kafka offset). Adapters
    /// SHOULD set this to something an operator can use to verify
    /// the publish post-hoc.
    pub published_at: String,
}

impl PublishReceipt {
    /// Construct a publish receipt.
    pub fn new(
        artifact_id: impl Into<String>,
        target: PublishTarget,
        published_at: impl Into<String>,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            target,
            published_at: published_at.into(),
        }
    }
}

/// A reference to a container image — consumed by
/// [`Runtime::spawn`](crate::Runtime::spawn).
///
/// `ImageRef` is intentionally minimal: just a string in
/// `<repo>[:<tag>][@<digest>]` form (or whatever the underlying
/// runtime accepts). Adapters that need richer addressing (e.g.
/// a separate `tag` and `digest` field) can split the string
/// locally.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageRef {
    /// The full image reference string, exactly as the runtime
    /// should consume it (e.g. `"phenocommand-web:0.1.0"`,
    /// `"registry.phenotype/internal/phenocommard-web@sha256:abc..."`).
    pub reference: String,
}

impl ImageRef {
    /// Construct an image ref from a reference string.
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
        }
    }

    /// Convenience: build an image ref from a repo and a tag
    /// (joined as `"<repo>:<tag>"`).
    pub fn with_tag(repo: impl AsRef<str>, tag: impl AsRef<str>) -> Self {
        Self::new(format!("{}:{}", repo.as_ref(), tag.as_ref()))
    }
}

impl AsRef<str> for ImageRef {
    fn as_ref(&self) -> &str {
        &self.reference
    }
}

impl From<&str> for ImageRef {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ImageRef {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for ImageRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.reference)
    }
}

/// Opaque handle to a running container — returned by
/// [`Runtime::spawn`](crate::Runtime::spawn), consumed by
/// [`Runtime::stop`](crate::Runtime::stop) and
/// [`Runtime::status`](crate::Runtime::status).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContainerId {
    /// The runtime-assigned id (e.g. a Docker container id, a
    /// `systemd-nspawn` machine name).
    pub id: String,
}

impl ContainerId {
    /// Construct a container id.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl AsRef<str> for ContainerId {
    fn as_ref(&self) -> &str {
        &self.id
    }
}

impl From<&str> for ContainerId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ContainerId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for ContainerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.id)
    }
}

/// The state of a container, as reported by
/// [`Runtime::status`](crate::Runtime::status).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerStatus {
    /// The container is running.
    Running,
    /// The container has exited (cleanly or not — see the exit
    /// code on the adapter side if it cares).
    Exited,
    /// The container is paused (SIGSTOP'd, cgroup frozen, etc.).
    Paused,
    /// The container does not exist (the runtime no longer has
    /// any record of the id). Adapters return this for unknown
    /// ids so callers can distinguish "stopped" from "never
    /// existed".
    NotFound,
}

impl ContainerStatus {
    /// Returns `true` if the status indicates an active container
    /// ([`ContainerStatus::Running`] or [`ContainerStatus::Paused`]).
    pub fn is_active(self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

impl std::fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Running => "running",
            Self::Exited => "exited",
            Self::Paused => "paused",
            Self::NotFound => "not_found",
        })
    }
}

/// Errors that can arise in any of the three port-trait
/// adapters. Each variant carries the adapter-defined
/// contextual string (typically the adapter's own error type
/// rendered via `Display`).
///
/// This is intentionally a single error type so that downstream
/// `Box<dyn Trait>` storage can produce a single `Result<_,
/// PortError>` shape across all three port traits without
/// forcing the caller to learn three error enums.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PortError {
    /// The input failed validation (e.g. a [`Manifest`] with an
    /// empty name, a [`PublishTarget`] with an empty locator).
    #[error("validation error: {0}")]
    Validation(String),
    /// The request referred to a resource the adapter could not
    /// find (e.g. a container id that does not exist on
    /// `Runtime::status`).
    #[error("not found: {0}")]
    NotFound(String),
    /// The underlying transport or backend failed (network
    /// error, registry 5xx, runtime daemon offline, etc.).
    #[error("transport error: {0}")]
    Transport(String),
    /// The operation is not supported by this adapter (e.g. a
    /// `stop` on a read-only runtime). Adapters should return
    /// this rather than panicking or returning a generic
    /// `Transport` error so callers can branch on the cause.
    #[error("unsupported: {0}")]
    Unsupported(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_new_sets_name_and_leaves_others_empty() {
        let m = Manifest::new("phenocommand-web");
        assert_eq!(m.name, "phenocommand-web");
        assert!(m.artifact_name.is_none());
        assert!(m.tags.is_empty());
    }

    #[test]
    fn manifest_builder_set_artifact_name_and_tags() {
        let m = Manifest::new("phenocommand-web")
            .with_artifact_name("phenocommand-web:0.1.0")
            .with_tag("channel", "stable")
            .with_tag("version", "0.1.0");
        assert_eq!(m.artifact_name.as_deref(), Some("phenocommand-web:0.1.0"));
        assert_eq!(m.tags.len(), 2);
        assert_eq!(m.tags[0], ("channel".to_string(), "stable".to_string()));
        assert_eq!(m.tags[1], ("version".to_string(), "0.1.0".to_string()));
    }

    #[test]
    fn composed_artifact_tag_lookup() {
        let a = ComposedArtifact::new("phenocommand-web:0.1.0", ImageRef::new("phenocommand-web:0.1.0"))
            .with_tag("content-digest", "sha256:abc");
        assert_eq!(a.tag("content-digest"), Some("sha256:abc"));
        assert_eq!(a.tag("missing"), None);
    }

    #[test]
    fn image_ref_with_tag_joins_correctly() {
        let r = ImageRef::with_tag("phenocommand-web", "0.1.0");
        assert_eq!(r.reference, "phenocommand-web:0.1.0");
        assert_eq!(r.as_ref(), "phenocommand-web:0.1.0");
        assert_eq!(format!("{r}"), "phenocommand-web:0.1.0");
    }

    #[test]
    fn image_ref_from_str_and_string() {
        let from_str: ImageRef = "phenocommand-web:0.1.0".into();
        let from_string: ImageRef = String::from("phenocommand-web:0.2.0").into();
        assert_eq!(from_str.reference, "phenocommand-web:0.1.0");
        assert_eq!(from_string.reference, "phenocommand-web:0.2.0");
    }

    #[test]
    fn container_id_display_and_as_ref() {
        let id = ContainerId::new("abc123");
        assert_eq!(id.as_ref(), "abc123");
        assert_eq!(format!("{id}"), "abc123");
    }

    #[test]
    fn container_status_is_active_and_display() {
        assert!(ContainerStatus::Running.is_active());
        assert!(ContainerStatus::Paused.is_active());
        assert!(!ContainerStatus::Exited.is_active());
        assert!(!ContainerStatus::NotFound.is_active());

        assert_eq!(format!("{}", ContainerStatus::Running), "running");
        assert_eq!(format!("{}", ContainerStatus::Exited), "exited");
        assert_eq!(format!("{}", ContainerStatus::Paused), "paused");
        assert_eq!(format!("{}", ContainerStatus::NotFound), "not_found");
    }

    #[test]
    fn publish_target_new() {
        let t = PublishTarget::new("docker-registry", "registry.phenotype/phenocommand/web:0.1.0");
        assert_eq!(t.kind, "docker-registry");
        assert_eq!(t.locator, "registry.phenotype/phenocommand/web:0.1.0");
    }

    #[test]
    fn publish_receipt_new() {
        let t = PublishTarget::new("file", "/var/lib/phenocompose/x.tar");
        let r = PublishReceipt::new("phenocommand-web:0.1.0", t.clone(), "/var/lib/phenocompose/x.tar");
        assert_eq!(r.artifact_id, "phenocommand-web:0.1.0");
        assert_eq!(r.target, t);
        assert_eq!(r.published_at, "/var/lib/phenocompose/x.tar");
    }

    #[test]
    fn port_error_display_mentions_kind_and_context() {
        let e = PortError::Validation("empty name".to_string());
        let s = format!("{e}");
        assert!(s.contains("validation"));
        assert!(s.contains("empty name"));
    }
}
