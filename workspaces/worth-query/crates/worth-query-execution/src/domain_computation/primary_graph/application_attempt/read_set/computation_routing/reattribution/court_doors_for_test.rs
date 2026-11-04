//! Sealed test doors over this module's private members.
use super::*;

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn reattribute_for_test(
        &self,
        calls: &BTreeSet<ComputationRead>,
        fresh: &BTreeMap<ComputationRead, Vec<&WorthQueryApplicationFactKey>>,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<Self>, StoreDenial> {
        Self::reattribute(self, calls, fresh, control)
    }
}
