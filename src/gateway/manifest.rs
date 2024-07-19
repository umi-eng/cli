use serde::Deserialize;
use std::collections::HashMap;

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
    schema: String,
    latest: String,
    stable: String,
    binaries: HashMap<String, FirmwareBinary>,
}

impl Manifest {
    /// Return the metadata for the latest firmware binary.
    ///
    /// Note this will return `None` if the version specified as latest is not
    /// in the map of binaries.
    pub fn latest(&self) -> Option<(&String, &FirmwareBinary)> {
        self.binaries.get_key_value(&self.latest)
    }

    /// Return the metadata for the stable firmware binary.
    ///
    /// Note this will return `None` if the version specified as stable is not
    /// in the map of binaries.
    pub fn stable(&self) -> Option<(&String, &FirmwareBinary)> {
        self.binaries.get_key_value(&self.stable)
    }

    /// The metadata for a specific firmware binary version.
    ///
    /// Note this will return `None` if the version specified is not in the map
    /// of binaries.
    pub fn version(&self, ver: &str) -> Option<(&String, &FirmwareBinary)> {
        self.binaries.get_key_value(ver)
    }

    /// Returns the map of firmware binaries.
    ///
    /// Key: version identifier.
    /// Value: firmware binary metadata.
    pub fn binaries(&self) -> &HashMap<String, FirmwareBinary> {
        &self.binaries
    }
}

/// Metadata for a firmware release binary.
///
/// Each firmware binary _must_ specify the minimum required firmware version
/// so devices can step-up to the latest firmware without issues due to
/// breaking changes.
#[derive(Debug, Deserialize)]
pub struct FirmwareBinary {
    file: String,
    min: String,
}

impl FirmwareBinary {
    /// Firmware binary file URL.
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Minimum version required to upgrade to this binary.
    pub fn minimum_supported_version(&self) -> &str {
        &self.min
    }
}
