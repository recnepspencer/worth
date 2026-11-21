//! Thin forwarding shims for `elevation_progression` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_elevation_progression_approval {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use worth_query_execution_attempt::facade::domain_computation::authorization::elevation_progression::approval::*;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryRequestedElevation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn authorize_elevation_approval<Capability, Operation, Input>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            requested: WorthQueryRequestedElevation,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
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
            self.inner
                .authorize_elevation_approval::<Capability, Operation, Input>(
                    phase,
                    requested,
                    access,
                    operation,
                    preconditions,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_elevation_progression_elevation_close {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use worth_query_execution_attempt::facade::domain_computation::authorization::elevation_progression::elevation_close::*;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryApprovedElevation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn authorize_elevation_close<Capability, Operation, Input>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            approved: WorthQueryApprovedElevation,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
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
            self.inner
                .authorize_elevation_close::<Capability, Operation, Input>(
                    phase,
                    approved,
                    access,
                    operation,
                    preconditions,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_elevation_progression_idempotency_observation {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolution;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial as Denial;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn resolve_capability_lifecycle_idempotency<Capability, Operation, Input>(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
            operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
            preconditions: TypedMutationPreconditions<Schema, Operation, Input::Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> Result<WorthQueryApplicationIdempotencyResolution, Denial>
        where
            Operation: ApplicationOperationMarkerIdentity<Schema>,
            Input: ApplicationCapabilityRequest<Schema, Capability> + Clone + Send + Sync + 'static,
        {
            self.inner
                .resolve_capability_lifecycle_idempotency::<Capability, Operation, Input>(
                    phase,
                    access,
                    operation,
                    preconditions,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_elevation_progression_mandatory_review {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use worth_query_execution_attempt::facade::domain_computation::authorization::elevation_progression::mandatory_review::*;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryMandatoryReview;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn authorize_mandatory_review<Capability, Operation, Input>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            mandatory: WorthQueryMandatoryReview,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
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
            self.inner
                .authorize_mandatory_review::<Capability, Operation, Input>(
                    phase,
                    mandatory,
                    access,
                    operation,
                    preconditions,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_elevation_progression_request {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationRequest;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn authorize_elevation_request<Capability, Operation, Input>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
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
            self.inner
                .authorize_elevation_request::<Capability, Operation, Input>(
                    phase,
                    access,
                    operation,
                    preconditions,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_elevation_progression_approval_replay {
    use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
    use worth_query_execution_attempt::facade::domain_computation::authorization::elevation_progression::approval::*;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial;
    use crate::domain_computation::primary_graph::WorthQueryElevationApprovalOutcome;
    use crate::domain_computation::primary_graph::WorthQueryRequestedElevation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn replay_elevation_approval_after_fresh_denial<Capability, Operation, Input>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            requested: WorthQueryRequestedElevation,
            access: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >,
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
            self.inner
                .replay_elevation_approval_after_fresh_denial::<Capability, Operation, Input>(
                    phase,
                    requested,
                    access,
                    operation,
                    preconditions,
                    idempotency,
                )
        }
    }
}
