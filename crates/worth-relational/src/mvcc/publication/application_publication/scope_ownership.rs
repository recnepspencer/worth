//! Immutable installed producer declarations; source references do not opt in.
use super::RelationalInstalledApplicationPublicationAuthority;
use crate::identity::data::KindId;
use crate::schema::installed_source_scopes::InstalledSourceScopeOwnership;
use crate::storage::overlay::PartitionAccess;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

impl RelationalInstalledApplicationPublicationAuthority {
    pub(crate) fn require_current_ownership_storage(
        &self,
        basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
        source: crate::identity::data::EntityId,
    ) -> Result<
        (),
        crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    > {
        if (*(*basis.owner_inner())
            .root
            .schema_authority()
            .registry()
            .owner_current_publication_storage())
        .is_none()
        {
            return Err(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                crate::publication::data::PublicationError::authored_retirement(
                    crate::publication::data::AuthoredRetirementRefusal::CurrentOwnershipStorageMissing,
                    crate::transactions::data::primitives::RecordRef::Entity(source),
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
        self.source_scope_ownership = InstalledSourceScopeOwnership::new(producers);
        self
    }

    /// Whether this installed publisher owns any producer's source scope. Only
    /// then can a graph selection meet producer custody that it must restore.
    pub fn owns_source_scopes(&self) -> bool {
        !self.source_scope_ownership.is_empty()
    }
}

impl RelationalInstalledApplicationPublicationAuthority {
    pub(crate) fn demand_source_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
        publication: &super::RequiredOutputPublication,
    ) -> Result<
        (),
        crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    > {
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
            (*basis.owner_inner())
                .root
                .demand_full_partition(execution, meaning.source_scope.partition_id)
                .map_err(super::custody::execution_stop)?;
        }
        Ok(())
    }
}
