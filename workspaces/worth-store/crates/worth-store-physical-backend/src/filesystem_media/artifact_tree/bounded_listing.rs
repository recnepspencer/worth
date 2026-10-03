use std::ffi::OsString;

use worth_store_physical_format::store_namespace::NamespaceEntryType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTreeDirectoryEntry {
    name: OsString,
    entry_type: NamespaceEntryType,
}

impl ArtifactTreeDirectoryEntry {
    pub(super) const fn new(name: OsString, entry_type: NamespaceEntryType) -> Self {
        Self { name, entry_type }
    }

    pub fn name(&self) -> &std::ffi::OsStr {
        &self.name
    }

    pub const fn entry_type(&self) -> NamespaceEntryType {
        self.entry_type
    }

    #[cfg(any(test, feature = "recovery-runtime-owner"))]
    pub(crate) fn into_parts(self) -> (OsString, NamespaceEntryType) {
        (self.name, self.entry_type)
    }
}
