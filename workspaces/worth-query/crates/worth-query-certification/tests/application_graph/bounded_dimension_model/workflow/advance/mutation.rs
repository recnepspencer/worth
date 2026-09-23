use worth_query_host::facade::declaration::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling, ApplicationCapabilityMutationBinding,
        ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationIntent,
        NoApplicationMutationOutputs, NoApplicationMutationSource,
    },
    application_schema::{
        ApplicationFieldRef, ApplicationPrincipalBindingRef, ApplicationSchemaDeclarationBuilder,
        EqualityPredicate, NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
    },
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};
use worth_query_host::facade::worth_query_structured_value_binding;

use super::super::super::schema::{
    BoundedDimensionSchema, ExternalMapping, Part, PartFacts, PartIdentityField,
    PartPrincipalBinding, Principal,
};
use super::declaration::WorkflowAdvanceInputBinding;
use super::{
    WorkflowAdvanceCapability, WorkflowAdvanceInput, WorkflowAdvanceOperation,
    WorkflowApprovalCapability,
};

#[derive(Clone, Debug)]
pub struct WorkflowAdvanceIntent {
    pub input: WorkflowAdvanceInput,
}

#[derive(Clone, Debug)]
pub struct WorkflowApprovalIntent {
    pub input: WorkflowAdvanceInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowAdvanceAccepted;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowAdvanceDenial {}

worth_query_structured_value_binding!(pub WorkflowAdvanceAcceptedBinding for WorkflowAdvanceAccepted {
    identity: "worth.query.certification.workflow-advance-accepted.v1"
});
worth_query_structured_value_binding!(pub WorkflowAdvanceDenialBinding for WorkflowAdvanceDenial {
    identity: "worth.query.certification.workflow-advance-denial.v1"
});

pub struct WorkflowAdvanceBinding;
pub struct WorkflowApprovalBinding;

type WorkflowAdvanceScope = ApplicationMutationFieldScope<
    BoundedDimensionSchema,
    Part,
    PartFacts,
    PartIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<BoundedDimensionSchema> for WorkflowAdvanceBinding {
    type Input = WorkflowAdvanceInput;
    type InputBinding = WorkflowAdvanceInputBinding;
    type Result = WorkflowAdvanceAccepted;
    type ResultBinding = WorkflowAdvanceAcceptedBinding;
    type IdempotencyKey = u64;
    type Operation = WorkflowAdvanceOperation;
    type Decision = WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>;
    type Denial = WorkflowAdvanceDenial;
    type DenialBinding = WorkflowAdvanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = WorkflowAdvanceScope;
    type PrincipalBinding = PartPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.workflow-advance-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.workflow-advance-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.workflow-advance.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(8, 0, 8, 2, 64, 0),
            ApplicationCandidateResourceCeiling::bounded(256 * 1024, 131_072),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        content_identity(&key.to_le_bytes())
    }

    fn input_identity(input: &WorkflowAdvanceInput) -> [u8; 32] {
        content_identity(input.part_identity.as_bytes())
    }

    fn scope_field() -> ApplicationFieldRef<
        BoundedDimensionSchema,
        Part,
        PartFacts,
        PartIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        PartIdentityField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BoundedDimensionSchema,
        PartPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PartPrincipalBinding::reference()
    }
}

impl ApplicationCapabilityMutationBinding<BoundedDimensionSchema> for WorkflowAdvanceBinding {
    type Capability = WorkflowAdvanceCapability;
}

impl ApplicationMutationBinding<BoundedDimensionSchema> for WorkflowApprovalBinding {
    type Input = WorkflowAdvanceInput;
    type InputBinding = WorkflowAdvanceInputBinding;
    type Result = WorkflowAdvanceAccepted;
    type ResultBinding = WorkflowAdvanceAcceptedBinding;
    type IdempotencyKey = u64;
    type Operation = WorkflowAdvanceOperation;
    type Decision = WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>;
    type Denial = WorkflowAdvanceDenial;
    type DenialBinding = WorkflowAdvanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = WorkflowAdvanceScope;
    type PrincipalBinding = PartPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.workflow-approval-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.workflow-approval-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.workflow-approval.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements = WorkflowAdvanceBinding::CANDIDATES;

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        content_identity(&key.to_le_bytes())
    }

    fn input_identity(input: &WorkflowAdvanceInput) -> [u8; 32] {
        content_identity(input.part_identity.as_bytes())
    }

    fn scope_field() -> ApplicationFieldRef<
        BoundedDimensionSchema,
        Part,
        PartFacts,
        PartIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        PartIdentityField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BoundedDimensionSchema,
        PartPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PartPrincipalBinding::reference()
    }
}

impl ApplicationCapabilityMutationBinding<BoundedDimensionSchema> for WorkflowApprovalBinding {
    type Capability = WorkflowApprovalCapability;
}

impl ApplicationMutationIntent<BoundedDimensionSchema> for WorkflowAdvanceIntent {
    type Binding = WorkflowAdvanceBinding;

    fn input(&self) -> &WorkflowAdvanceInput {
        &self.input
    }

    fn scope_binding(&self) -> WorkflowAdvanceScope {
        WorkflowAdvanceScope::new(
            PartIdentityField::reference(),
            self.input.part_identity.clone(),
        )
    }
}

impl ApplicationMutationIntent<BoundedDimensionSchema> for WorkflowApprovalIntent {
    type Binding = WorkflowApprovalBinding;

    fn input(&self) -> &WorkflowAdvanceInput {
        &self.input
    }

    fn scope_binding(&self) -> WorkflowAdvanceScope {
        WorkflowAdvanceScope::new(
            PartIdentityField::reference(),
            self.input.part_identity.clone(),
        )
    }
}

pub struct WorkflowAdvanceHandler;
pub struct WorkflowApprovalHandler;

impl OperationHandler<BoundedDimensionSchema, WorkflowAdvanceBinding> for WorkflowAdvanceHandler {
    fn decide(
        &self,
        input: &WorkflowAdvanceInput,
        reader: &mut DecisionReader<'_, '_, '_, BoundedDimensionSchema, WorkflowAdvanceBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        WorkflowAdvanceDenial,
    > {
        let part = match reader
            .resolve_entity(PartIdentityField::reference(), input.part_identity.clone())
        {
            Ok(part) => part,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.mutation_target(&part) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &WorkflowAdvanceInput,
        _: &WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
    ) -> ApplicationCandidateRequirements {
        WorkflowAdvanceBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &WorkflowAdvanceInput,
        _: WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        _: &mut CandidateWriter<'_, BoundedDimensionSchema, WorkflowAdvanceBinding>,
    ) -> HandlerResult<WorkflowAdvanceAccepted, WorkflowAdvanceDenial> {
        HandlerResult::Completed(WorkflowAdvanceAccepted)
    }
}

impl OperationHandler<BoundedDimensionSchema, WorkflowApprovalBinding> for WorkflowApprovalHandler {
    fn decide(
        &self,
        input: &WorkflowAdvanceInput,
        reader: &mut DecisionReader<'_, '_, '_, BoundedDimensionSchema, WorkflowApprovalBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        WorkflowAdvanceDenial,
    > {
        let part = match reader
            .resolve_entity(PartIdentityField::reference(), input.part_identity.clone())
        {
            Ok(part) => part,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.mutation_target(&part) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &WorkflowAdvanceInput,
        _: &WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
    ) -> ApplicationCandidateRequirements {
        WorkflowApprovalBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &WorkflowAdvanceInput,
        _: WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        _: &mut CandidateWriter<'_, BoundedDimensionSchema, WorkflowApprovalBinding>,
    ) -> HandlerResult<WorkflowAdvanceAccepted, WorkflowAdvanceDenial> {
        HandlerResult::Completed(WorkflowAdvanceAccepted)
    }
}

pub(super) fn install_binding(
    schema: ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema>,
) -> ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema> {
    schema
        .application_mutation_binding::<WorkflowAdvanceBinding>()
        .application_mutation_binding::<WorkflowApprovalBinding>()
}

fn content_identity(bytes: &[u8]) -> [u8; 32] {
    let mut identity = [0_u8; 32];
    let mut accumulator = 0xcbf2_9ce4_8422_2325_u64;
    for (index, byte) in bytes.iter().enumerate() {
        accumulator = (accumulator ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        identity[index % identity.len()] ^= (accumulator >> ((index % 8) * 8)) as u8;
    }
    identity
}
