//! Request and response types — the only layer that crosses the network.
//!
//! sqlx row types are never serialized here; see `docs/ARCHITECTURE.md`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Build identity of the running service.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Version {
    /// Git SHA the binary was built from, or `dev` for a local build.
    pub revision: String,
    /// Crate version.
    pub version: String,
}

/// RFC 9457 problem details, plus a stable `code` clients can branch on.
///
/// See `docs/API.md`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub status: u16,
    /// Prose. May change between releases.
    pub detail: String,
    /// Stable and machine-readable. Clients branch on this, never on `detail`.
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<BTreeMap<String, Vec<String>>>,
}
