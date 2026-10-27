//! Eager manifest definition inventories preserve installed numeric identities.
use super::*;
use crate::indexes::data::DerivedIndexDefinition;
impl IndexAccess<'_> {
    /// Finds the installed definition with the same semantic identity.
    ///
    /// The caller-supplied numeric identity is deliberately ignored: a
    /// recovered owner must bind the identity restored with the checkpoint,
    /// not allocate a look-alike definition after recovery.
    pub fn matching_definition(
        &self,
        expected: &DerivedIndexDefinition,
    ) -> Option<DerivedIndexDefinition> {
        self.runtime
            .indexes
            .definitions()
            .into_iter()
            .find(|candidate| {
                candidate.name == expected.name
                    && candidate.kind == expected.kind
                    && candidate.branch_scoped == expected.branch_scoped
            })
            .map(|definition| definition.as_ref().clone())
    }

    /// Detaches one exact-definition inventory for callers binding several
    /// restored indexes. Each lookup preserves the owner's numeric identity.
    pub fn definition_lookup_snapshot(&self) -> DerivedIndexDefinitionLookup {
        DerivedIndexDefinitionLookup::new(self.runtime.indexes.definitions())
    }

    pub(crate) fn definitions_snapshot(&self) -> Vec<DerivedIndexDefinition> {
        self.runtime
            .indexes
            .definitions()
            .into_iter()
            .map(|definition| definition.as_ref().clone())
            .collect()
    }
}
