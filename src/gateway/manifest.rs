use serde::Deserialize;
use std::collections::HashMap;

/// Manifest schema.
///
/// Only deserializes the `schema` field.
#[derive(Debug, Deserialize)]
pub struct ManifestSchema {
    pub schema: String,
}

/// Manifest file format.
///
/// # Example
///
/// ```json
/// {
///     "schema": "v0.1.0",
///     "latest": "v0.3.0",
///     "stable": "v0.2.0",
///     "binaries": {
///         "v0.2.0": { ... },
///         "v0.3.0": { ... },
///     }
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// Schema version.
    pub schema: String,
    /// Latest firmware version.
    pub latest: String,
    /// Stable firmware version.
    pub stable: String,
    /// List of available binaries.
    pub binaries: HashMap<String, FirmwareBinary>,
}

/// Metadata for a firmware release binary.
///
/// Each firmware binary _must_ specify the minimum required firmware version
/// so devices can step-up to the latest firmware without issues due to
/// breaking changes.
#[derive(Debug, Deserialize)]
pub struct FirmwareBinary {
    /// File link.
    pub file: String,
    /// Minimum supported version.
    pub min: String,
}
