//! Crate-boundary internal port for `WorthQueryProductPublicationBinding`.
//! Never exported on `worth-query-execution::facade` or Host.

use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;
use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;
use worth_runtime_world::facade::{
    NoEffectCompositePublication, PreparedCompositePublicationWithoutSignal,
    ProductBranchObservation, ProductUnpublishedRecoveryHandle, RuntimeWorldCancellationToken,
    RuntimeWorldInstant, RuntimeWorldObservationPort, RuntimeWorldPublicationPort,
    RuntimeWorldRecoveryPort, RuntimeWorldSettledRelationalAdoptionDenial, WorldOpenAttemptRefusal,
};

use super::super::application_publication::WorthQueryPreparedCombinedProductPublication;
use super::super::conditional_recovery::WorthQueryPreparedConditionalRecovery;
use super::super::publication_denial::WorthQueryProductPublicationDenial;
use super::super::request_control::{
    WorthQueryPreparedProductPublication, WorthQueryProductPublicationRequestControl,
};
use super::super::runtime::WorthQueryProductRootIdentity;
use super::{
    SelectedProgramSourceRetentionDenial, WorthQueryProductPublicationBinding,
    WorthQuerySelectedProgramInspection,
};

pub trait ProductPublicationBindingInternalPort {
    fn observation(&self) -> &ProductBranchObservation;
    fn publication(&self) -> &RuntimeWorldPublicationPort<(), (), (), (), ()>;
    fn recovery(&self) -> RuntimeWorldRecoveryPort;
    fn observation_port(&self) -> &RuntimeWorldObservationPort;
    fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity>;
    fn deadline(&self, deadline: worth_execution::MonotonicDeadline)
        -> Option<RuntimeWorldInstant>;

    fn retain_selected_program_source(
        &mut self,
        selected: &WorthQuerySelectedProgramInspection,
    ) -> Result<(), SelectedProgramSourceRetentionDenial>;

    fn carry_selected_program_source_from(
        &mut self,
        prior: &Self,
    ) -> Result<(), SelectedProgramSourceRetentionDenial>;

    fn admitted_program_source(
        &self,
    ) -> Result<worth_proof::AdmittedBranchProgramSource, WorldOpenAttemptRefusal>;

    fn request_control(
        &self,
        request: &WorthQueryRequestScope,
    ) -> Option<WorthQueryProductPublicationRequestControl>;

    fn prepare_relational_candidate(
        &self,
        candidate: PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
        successor_observation_requested: bool,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        WorthQueryProductPublicationDenial<NoEffectCompositePublication>,
    >;

    fn prepare_relational_candidate_with_current_read(
        &self,
        candidate: PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
        successor_observation_requested: bool,
        current_read: Option<&ProductBranchObservation>,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        WorthQueryProductPublicationDenial<NoEffectCompositePublication>,
    >;

    fn prepare_combined_candidate(
        &self,
        candidate: PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
        successor_observation_requested: bool,
        current_read: Option<&ProductBranchObservation>,
    ) -> Result<
        WorthQueryPreparedCombinedProductPublication,
        WorthQueryProductPublicationDenial<NoEffectCompositePublication>,
    >;

    fn prepare_conditional_definition_adoption(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
        bridge: &BridgeSealedRuntimeAssembly,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryPreparedConditionalRecovery,
        WorthQueryProductPublicationDenial<RuntimeWorldSettledRelationalAdoptionDenial>,
    >;

    fn prepare_settled_relational_adoption_with(
        &self,
        request: &WorthQueryRequestScope,
        prepare: impl FnOnce(
            &RuntimeWorldCancellationToken,
            Option<RuntimeWorldInstant>,
        ) -> Result<
            PreparedCompositePublicationWithoutSignal,
            RuntimeWorldSettledRelationalAdoptionDenial,
        >,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        WorthQueryProductPublicationDenial<RuntimeWorldSettledRelationalAdoptionDenial>,
    >;
}

impl ProductPublicationBindingInternalPort for WorthQueryProductPublicationBinding {
    fn observation(&self) -> &ProductBranchObservation {
        Self::observation(self)
    }

    fn publication(&self) -> &RuntimeWorldPublicationPort<(), (), (), (), ()> {
        Self::publication(self)
    }

    fn recovery(&self) -> RuntimeWorldRecoveryPort {
        Self::recovery(self)
    }

    fn observation_port(&self) -> &RuntimeWorldObservationPort {
        Self::observation_port(self)
    }

    fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Self::root_identity(self)
    }

    fn deadline(
        &self,
        deadline: worth_execution::MonotonicDeadline,
    ) -> Option<RuntimeWorldInstant> {
        Self::deadline(self, deadline)
    }

    fn retain_selected_program_source(
        &mut self,
        selected: &WorthQuerySelectedProgramInspection,
    ) -> Result<(), SelectedProgramSourceRetentionDenial> {
        Self::retain_selected_program_source(self, selected)
    }

    fn carry_selected_program_source_from(
        &mut self,
        prior: &Self,
    ) -> Result<(), SelectedProgramSourceRetentionDenial> {
        Self::carry_selected_program_source_from(self, prior)
    }

    fn admitted_program_source(
        &self,
    ) -> Result<worth_proof::AdmittedBranchProgramSource, WorldOpenAttemptRefusal> {
        Self::admitted_program_source(self)
    }

    fn request_control(
        &self,
        request: &WorthQueryRequestScope,
    ) -> Option<WorthQueryProductPublicationRequestControl> {
        Self::request_control(self, request)
    }

    fn prepare_relational_candidate(
        &self,
        candidate: PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
        successor_observation_requested: bool,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        WorthQueryProductPublicationDenial<NoEffectCompositePublication>,
    > {
        Self::prepare_relational_candidate(
            self,
            candidate,
            request,
            successor_observation_requested,
        )
    }

    fn prepare_relational_candidate_with_current_read(
        &self,
        candidate: PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
        successor_observation_requested: bool,
        current_read: Option<&ProductBranchObservation>,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        WorthQueryProductPublicationDenial<NoEffectCompositePublication>,
    > {
        Self::prepare_relational_candidate_with_current_read(
            self,
            candidate,
            request,
            successor_observation_requested,
            current_read,
        )
    }

    fn prepare_combined_candidate(
        &self,
        candidate: PreparedRelationalCommitCandidate,
        request: &WorthQueryRequestScope,
        successor_observation_requested: bool,
        current_read: Option<&ProductBranchObservation>,
    ) -> Result<
        WorthQueryPreparedCombinedProductPublication,
        WorthQueryProductPublicationDenial<NoEffectCompositePublication>,
    > {
        Self::prepare_combined_candidate(
            self,
            candidate,
            request,
            successor_observation_requested,
            current_read,
        )
    }

    fn prepare_conditional_definition_adoption(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
        bridge: &BridgeSealedRuntimeAssembly,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryPreparedConditionalRecovery,
        WorthQueryProductPublicationDenial<RuntimeWorldSettledRelationalAdoptionDenial>,
    > {
        Self::prepare_conditional_definition_adoption(self, handle, bridge, request)
    }

    fn prepare_settled_relational_adoption_with(
        &self,
        request: &WorthQueryRequestScope,
        prepare: impl FnOnce(
            &RuntimeWorldCancellationToken,
            Option<RuntimeWorldInstant>,
        ) -> Result<
            PreparedCompositePublicationWithoutSignal,
            RuntimeWorldSettledRelationalAdoptionDenial,
        >,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        WorthQueryProductPublicationDenial<RuntimeWorldSettledRelationalAdoptionDenial>,
    > {
        Self::prepare_settled_relational_adoption_with(self, request, prepare)
    }
}
