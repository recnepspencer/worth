//! The runtime's consumer methods over `domain_computation::authorization`; each only calls its subsystem.
use crate::facade::primary_graph::WorthQueryElevationApprovalAuthorizationDenial;
use crate::facade::primary_graph::WorthQueryElevationCloseAuthorizationDenial;
use crate::facade::primary_graph::WorthQueryMandatoryReviewAuthorizationDenial;
use worth_query_installation::facade::ApplicationSchema;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::CanonicalDigestWorkBudget;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRevocationRequest;
use worth_query_declaration::facade::application_capability::ErasedApplicationCapabilityEntitySelector;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_installation::facade::derive_capability_revocation_proposal_identity;
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;
use worth_query_installation::facade::WorthQueryCapabilityRevocationProposalBasis;
use worth_query_installation::facade::WorthQueryInstalledApplicationCapability;
use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
use worth_query_installation::facade::WorthQueryOperationFieldTouchScope;
use worth_query_installation::facade::WorthQueryOperationTouchScope;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRequest;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolution;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial;
use worth_query_installation::facade::ApplicationOperationDecisionReadTarget;
use worth_query_installation::facade::ApplicationOperationProgramTarget;
use crate::domain_computation::primary_graph::WorthQueryElevationApprovalOutcome;
use crate::domain_computation::primary_graph::WorthQueryRequestedElevation;
use crate::domain_computation::primary_graph::WorthQueryApprovedElevation;
use crate::domain_computation::primary_graph::WorthQueryElevationClosureKind;
use crate::domain_computation::primary_graph::WorthQueryMandatoryReview;
use std::collections::BTreeSet;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityCardinalityDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationRequest;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationRequestProjection;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationDimension;
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions;
use worth_query_installation::facade::WorthQueryCanonicalWorkPhases;
use crate::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation as OwnerAdmittedOperation;
use crate::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedOperation;

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn readmit_workflow_approval_authority<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQueryWorkflowApprovalAuthorityBasis,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        <Self as crate::domain_computation::authorization::WorkflowApprovalOps<Schema>>::readmit_workflow_approval_authority(self, execution, authority, admission)
    }

    pub fn authorize_capability_revocation<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation:
            worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            >,
        Input: ApplicationCapabilityRequest<Schema, Capability>
            + ApplicationCapabilityRevocationRequest<Schema, Capability>,
    {
        <Self as crate::domain_computation::authorization::CapabilityRevocationProgressionOps<
            Schema,
        >>::authorize_capability_revocation(
            self,
            phase,
            access,
            capability,
            operation,
            preconditions,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn authorize_capability_delegation<
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
    >(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            CommandCapability,
            Operation,
            Input,
        >,
        target: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation:
            worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            >,
        Input: ApplicationCapabilityRequest<Schema, CommandCapability>
            + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>,
    {
        <Self as crate::domain_computation::authorization::DelegationProgressionOps<Schema>>::authorize_capability_delegation(self, phase, access, target, operation, preconditions)
    }

    pub fn replay_capability_delegation_after_fresh_denial<
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
    >(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            CommandCapability,
            Operation,
            Input,
        >,
        target: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryApplicationCommitOutcome>,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, CommandCapability>
            + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        <Self as crate::domain_computation::authorization::ReplayOps<Schema>>::replay_capability_delegation_after_fresh_denial(self, phase, access, target, operation, preconditions, idempotency)
    }

    pub fn authorize_elevation_approval<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        requested: WorthQueryRequestedElevation,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryElevationApprovalAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
        Input: 'static,
    {
        <Self as crate::domain_computation::authorization::ApprovalOps<Schema>>::authorize_elevation_approval(self, phase, requested, access, operation, preconditions)
    }

    pub fn replay_elevation_approval_after_fresh_denial<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        requested: WorthQueryRequestedElevation,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryElevationApprovalOutcome,
        (
            Option<WorthQueryApplicationIdempotencyResolutionDenial>,
            WorthQueryRequestedElevation,
        ),
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::authorization::ApprovalReplayOps<Schema>>::replay_elevation_approval_after_fresh_denial(self, phase, requested, access, operation, preconditions, idempotency)
    }

    pub fn authorize_elevation_close<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        approved: WorthQueryApprovedElevation,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryElevationCloseAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
        Input: 'static,
    {
        <Self as crate::domain_computation::authorization::ElevationCloseOps<Schema>>::authorize_elevation_close(self, phase, approved, access, operation, preconditions)
    }

    pub fn authorize_mandatory_review<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        mandatory: WorthQueryMandatoryReview,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryMandatoryReviewAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
        Input: 'static,
    {
        <Self as crate::domain_computation::authorization::MandatoryReviewOps<Schema>>::authorize_mandatory_review(self, phase, mandatory, access, operation, preconditions)
    }

    pub fn authorize_elevation_request<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>
            + ApplicationCapabilityElevationRequest<Schema, Operation>
            + 'static,
    {
        <Self as crate::domain_computation::authorization::RequestOps<Schema>>::authorize_elevation_request(self, phase, access, operation, preconditions)
    }

    pub fn authorize_capability_operation<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
    {
        <Self as crate::domain_computation::authorization::OperationProgressionOps<Schema>>::authorize_capability_operation(self, phase, access, operation, preconditions)
    }

    pub fn authorize_borrowed_capability_operation<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: &TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
    {
        <Self as crate::domain_computation::authorization::BorrowedCapabilityOps<Schema>>::authorize_borrowed_capability_operation(self, phase, access, operation, preconditions)
    }
}
