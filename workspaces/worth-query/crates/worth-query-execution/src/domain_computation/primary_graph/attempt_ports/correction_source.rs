//! `PreparedCorrectionSourcePort` implementation for `WorthQueryPreparedCorrectionSource`.

use std::any::Any;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::CorrectionMaterialPinIdentity;
use worth_proof::HomeIncarnationDenial;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_aftermath::DeclaredAuthoredCorrection,
    application_operation::ApplicationMutationOutputPosture,
};
use worth_query_execution_attempt::facade::ports::{
    AuthoredCorrectionCandidateDenial as PortCandidateDenial, PreparedCorrectionSourcePort,
};
use worth_query_installation::facade::InstalledPreImageDemand;
use worth_relational::facade::{
    history::{BranchId, RelationalCommitReceipt},
    identity::EntityId,
    mvcc::ValidatedRelationalProposal,
    transactions::{CreatedEntityRef, EntityReference, WorkerIntentBatch},
};
use worth_runtime_world::facade::{CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding;
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::{
    product_operation::correction_source::{
        AuthoredCorrectionCandidateDenial, WorthQueryPreparedCorrectionSource,
    },
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationCommitDenial,
    WorthQueryPrimaryGraphLayout,
};
use crate::domain_computation::WorthQueryProviderSessionFailure;

fn map_candidate_denial(denial: AuthoredCorrectionCandidateDenial) -> PortCandidateDenial {
    match denial {
        AuthoredCorrectionCandidateDenial::Identity(subject) => {
            PortCandidateDenial::Identity(subject)
        }
        AuthoredCorrectionCandidateDenial::Interrupted(interruption) => {
            PortCandidateDenial::Interrupted(interruption)
        }
    }
}

pub fn downcast_prepared_correction_source(
    port: &dyn PreparedCorrectionSourcePort,
) -> &WorthQueryPreparedCorrectionSource {
    port.as_any()
        .downcast_ref::<WorthQueryPreparedCorrectionSource>()
        .expect("prepared correction source port is always WorthQueryPreparedCorrectionSource")
}

impl PreparedCorrectionSourcePort for WorthQueryPreparedCorrectionSource {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_source(&self, other: &dyn PreparedCorrectionSourcePort) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|rhs| self == rhs)
    }

    fn lineage_binding(&self) -> &WorthQueryCorrectionLineageBinding {
        self.lineage_binding()
    }

    fn target_parent(&self) -> &RelationalCommitReceipt {
        self.target_parent()
    }

    fn target_branch(&self) -> &BranchId {
        self.target_branch()
    }

    fn source_world(&self) -> &CompositeCommitIdentity {
        self.source_world()
    }

    fn source_commit(&self) -> &RelationalCommitReceipt {
        self.source_commit()
    }

    fn intent_causal_key(&self) -> String {
        self.intent().causal_key()
    }

    fn authored_declaration(&self) -> Option<&DeclaredAuthoredCorrection> {
        self.authored_declaration()
    }

    fn authored_retirement_anchor(&self) -> Option<EntityId> {
        self.authored_retirement_anchor()
    }

    fn authored_retirement_co_anchors(&self) -> Vec<EntityId> {
        self.authored_retirement_co_anchors()
    }

    fn authored_restoration_anchor(
        &self,
        roles: &[(
            &str,
            ApplicationMutationOutputPosture,
            &str,
            &EntityReference,
        )],
        batch: &WorkerIntentBatch,
        request: &WorthQueryRequestScope,
    ) -> Result<Option<(EntityId, CreatedEntityRef)>, PortCandidateDenial> {
        self.authored_restoration_anchor(roles.iter().copied(), batch, request)
            .map_err(map_candidate_denial)
    }

    fn authored_restoration_co_anchors(
        &self,
        roles: &[(
            &str,
            ApplicationMutationOutputPosture,
            &str,
            &EntityReference,
        )],
    ) -> Vec<(EntityId, CreatedEntityRef)> {
        self.authored_restoration_co_anchors(roles.iter().copied())
    }

    fn matches_admission_basis(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        principal: &WorthQueryOperationScopeEntityBinding,
        scope: &WorthQueryOperationScopeEntityBinding,
        target: Option<&WorthQueryProductBranchReadIdentity>,
    ) -> bool {
        self.matches_admission_basis(runtime, principal, scope, target)
    }

    fn admit_home_receiver(
        &self,
        owner: RuntimeWorldOwnerIdentity,
    ) -> Result<(), HomeIncarnationDenial> {
        self.admit_home_receiver(owner)
    }

    fn admit_receiver(
        &self,
        owner: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryApplicationCommitDenial> {
        self.admit_receiver(owner)
    }

    fn material_pin(&self) -> Option<CorrectionMaterialPinIdentity> {
        self.material_pin()
    }

    fn validate_authored_candidate(
        &self,
        roles: &[(
            &str,
            ApplicationMutationOutputPosture,
            &str,
            &EntityReference,
        )],
        batch: &WorkerIntentBatch,
        request: &WorthQueryRequestScope,
    ) -> Result<(), PortCandidateDenial> {
        self.validate_authored_candidate(roles.iter().copied(), batch, request)
            .map_err(map_candidate_denial)
    }

    fn validate_workflow_candidate(
        &self,
        batch: &WorkerIntentBatch,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.validate_workflow_candidate(batch)
    }

    fn validate_relation_inverse(
        &self,
        candidate: &ValidatedRelationalProposal,
        execution: &ExecutionRequest<'_, '_>,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.validate_relation_inverse(candidate, execution, layout)
    }

    fn changed_field_capture(&self) -> Result<Option<(InstalledPreImageDemand, EntityId)>, ()> {
        self.changed_field_capture()
    }

    fn capture_coupled_outputs(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Option<crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemands>
{
        self.capture_coupled_outputs(execution)
    }

    fn content(&self) -> &worth_relational::facade::content::RelationalContentAttachments {
        self.content()
    }
}
