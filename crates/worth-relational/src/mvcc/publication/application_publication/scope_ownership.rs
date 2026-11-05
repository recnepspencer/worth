//! Immutable installed producer declarations; source references do not opt in.
use super::RelationalInstalledApplicationPublicationAuthority;
use crate::{identity::data::KindId, storage::overlay::PartitionAccess};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, Default)]
pub(crate) struct InstalledSourceScopeOwnership(Arc<BTreeMap<String, KindId>>);

impl RelationalInstalledApplicationPublicationAuthority {
    pub(crate) fn require_current_ownership_storage(
        &self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        source: crate::identity::data::EntityId,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        if basis
            .inner
            .root
            .schema_authority()
            .registry()
            .current_publication_storage
            .is_none()
        {
            return Err(crate::transactions::data::TransactionCommitError::publication(
                crate::publication::data::PublicationError::authored_retirement(
                    crate::publication::data::AuthoredRetirementRefusal::CurrentOwnershipStorageMissing,
                    crate::transactions::data::RecordRef::Entity(source),
                ),
            ));
        }
        Ok(())
    }

    /// Compiler installation supplies only producers whose declared source query
    /// is exactly its entity scope. This is not an ordinary publication control.
    /// The installed schema must also use `register_current_publication_storage`;
    /// declaration alone cannot admit an owned publication without that substrate.
    pub fn with_source_scope_owned_producers(
        mut self,
        producers: impl IntoIterator<Item = (String, KindId)>,
    ) -> Self {
        self.source_scope_ownership =
            InstalledSourceScopeOwnership(Arc::new(producers.into_iter().collect()));
        self
    }
}

impl InstalledSourceScopeOwnership {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub(crate) fn lookup_work(&self, producer: &str) -> usize {
        if self.0.is_empty() {
            return 0;
        }
        let height = usize::BITS as usize - self.0.len().max(1).leading_zeros() as usize;
        self.0
            .len()
            .min(11)
            .saturating_mul(height)
            .saturating_add(1)
            .saturating_mul(producer.len().saturating_add(1))
    }
    pub(crate) fn scope_kind(&self, producer: &str) -> Option<KindId> {
        self.0.get(producer).copied()
    }
}

impl RelationalInstalledApplicationPublicationAuthority {
    pub(crate) fn demand_source_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        publication: &super::RequiredOutputPublication,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        use super::RequiredOutputPublication as Publication;
        let meaning = match publication {
            Publication::Birth { meaning, .. } | Publication::Preserve { meaning, .. } => meaning,
            Publication::AuthoredRetirement { predecessor, .. } => &predecessor.origin.meaning,
            _ => return Ok(()),
        };
        let Some((producer, _)) = publication.lineage_attribution() else {
            return Ok(());
        };
        if self.source_scope_ownership.scope_kind(producer).is_some() {
            basis
                .inner
                .root
                .demand_full_partition(execution, meaning.source_scope.partition_id)
                .map_err(super::custody::execution_stop)?;
        }
        Ok(())
    }
}
