//! Associate the exact Query head with custody readmitted by Native, never history.
use super::*;
mod roles;
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    identity::{EntityId, PartitionId},
    mvcc::{
        RelationalInstalledApplicationPublicationAuthority, RequiredOutputPublicationMeaning,
        RequiredOutputRoleTarget, TransactionCommitError,
    },
    runtime::RelationalRuntime,
    transactions::EntityReference,
};

pub enum RequiredPublicationReadmissionStop {
    Selection(Stop),
    Native(TransactionCommitError),
    Allocation(worth_execution::ExecutionAllocationDenial),
}

/// Only the selected-head owner constructs this exact association. It proves
/// publication custody, not current computation, a receipt or a fresh grant.
pub struct ReadmittedRequiredPublication {
    selected: Arc<OnceLock<RecordedOutput>>,
    publication: RelationalRequiredOutputPublication,
}
impl ReadmittedRequiredPublication {
    pub(in crate::domain_computation::primary_graph) fn publication(
        &self,
    ) -> &RelationalRequiredOutputPublication {
        &self.publication
    }
    pub(in crate::domain_computation::primary_graph::output_lineage) fn matches_cell(
        &self,
        cell: &Arc<OnceLock<RecordedOutput>>,
    ) -> bool {
        Arc::ptr_eq(&self.selected, cell)
    }
}

impl SelectedRequiredPublicationHead {
    /// An ordinary recorded computation is not a Native publication. Only
    /// Native's current route owner can establish that distinction on this C.
    pub(in crate::domain_computation::primary_graph) fn readmit_if_published(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        family: &str,
        scope: WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<Option<ReadmittedRequiredPublication>, RequiredPublicationReadmissionStop> {
        let warm = self.warm_publication();
        let cold;
        let meaning = match warm.as_ref() {
            Some(publication) => publication.meaning(),
            None => {
                let prior = self
                    .cold_predecessor()
                    .map_err(RequiredPublicationReadmissionStop::Selection)?;
                cold = RequiredOutputPublicationMeaning {
                    implementation: prior.implementation,
                    family: family.to_owned(),
                    source_scope: EntityId::new(
                        PartitionId::new(scope.partition_id()),
                        scope.local_slot(),
                        scope.generation(),
                    ),
                    source_partition: partition,
                    source_epoch: prior.source_epoch,
                };
                &cold
            }
        };
        if runtime
            .readmit_required_output_route_absence(execution, installed, basis, meaning)
            .map_err(RequiredPublicationReadmissionStop::Native)?
            .is_some()
        {
            return Ok(None);
        }
        self.readmit(
            runtime, execution, installed, basis, family, scope, partition, layout,
        )
        .map(Some)
    }

    pub(in crate::domain_computation::primary_graph) fn readmit(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        family: &str,
        scope: WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<ReadmittedRequiredPublication, RequiredPublicationReadmissionStop> {
        use RequiredPublicationReadmissionStop as Denial;
        let publication = match self.warm_publication() {
            Some(publication) => runtime
                .readmit_carried_required_output_publication(
                    execution,
                    installed,
                    basis,
                    &publication,
                )
                .map_err(Denial::Native)?,
            None => {
                let prior = self.cold_predecessor().map_err(Denial::Selection)?;
                let targets = roles::retain(execution, layout, &prior.correspondence, family)?;
                let meaning = RequiredOutputPublicationMeaning {
                    implementation: prior.implementation,
                    family: family.to_owned(),
                    source_scope: EntityId::new(
                        PartitionId::new(scope.partition_id()),
                        scope.local_slot(),
                        scope.generation(),
                    ),
                    source_partition: partition,
                    source_epoch: prior.source_epoch,
                };
                runtime
                    .readmit_required_output_publication(
                        execution,
                        installed,
                        basis,
                        prior.producer,
                        &prior.dependency,
                        &meaning,
                        &targets.targets,
                    )
                    .map_err(Denial::Native)?
            }
        };
        Ok(ReadmittedRequiredPublication {
            selected: Arc::clone(&self.selected),
            publication,
        })
    }
}
