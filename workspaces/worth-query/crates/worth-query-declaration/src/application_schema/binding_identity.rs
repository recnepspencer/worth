use worth_foundational::facade::CanonicalDigestId;

/// Descriptive identity of the installed schema against which an intent was
/// authored.
///
/// This value is not installation authority. Execution must validate it
/// against the opaque installed schema handle.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ApplicationSchemaBindingIdentity {
    runtime_ordinal: u64,
    generation: u64,
    package_identity: CanonicalDigestId,
    schema_identity: CanonicalDigestId,
}

impl ApplicationSchemaBindingIdentity {
    #[doc(hidden)]
    pub fn from_installed_parts(
        runtime_ordinal: u64,
        generation: u64,
        package_identity: CanonicalDigestId,
        schema_identity: CanonicalDigestId,
    ) -> Self {
        Self {
            runtime_ordinal,
            generation,
            package_identity,
            schema_identity,
        }
    }

    pub const fn runtime_ordinal(&self) -> u64 {
        self.runtime_ordinal
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn package_identity(&self) -> &CanonicalDigestId {
        &self.package_identity
    }

    pub const fn schema_identity(&self) -> &CanonicalDigestId {
        &self.schema_identity
    }
}
