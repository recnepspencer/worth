//! A real native unlink lets the certification court observe relation ABA.

use worth_query_decl::facade::{worth_query_operation_reads, worth_query_operation_unlinks};
use worth_query_host::facade::worth_query_operation;
use worth_query_host::facade::worth_query_structured_value_binding;

use super::*;

worth_query_operation!(pub UnlinkReviewRequirement for BoundedDimensionSchema, input ReviewRequirementInputBinding);
worth_query_operation_reads!(UnlinkReviewRequirement => [PartIdentityField, ReviewRequired]);
worth_query_operation_unlinks!(UnlinkReviewRequirement => [ReviewRequired]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewRequirementUnlinked;
worth_query_structured_value_binding!(pub ReviewRequirementUnlinkedBinding for ReviewRequirementUnlinked {
    identity: "worth.query.certification.review-requirement-unlinked.v1"
});

pub struct UnlinkReviewRequirementBinding;
impl ApplicationMutationBinding<BoundedDimensionSchema> for UnlinkReviewRequirementBinding {
    type Input = ReviewRequirementInput;
    type InputBinding = ReviewRequirementInputBinding;
    type Result = ReviewRequirementUnlinked;
    type ResultBinding = ReviewRequirementUnlinkedBinding;
    type IdempotencyKey = u64;
    type Operation = UnlinkReviewRequirement;
    type Decision = ReviewRequirementTargets;
    type Denial = ReviewRequirementDenial;
    type DenialBinding = ReviewRequirementDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ReviewRequirementScope;
    type PrincipalBinding = PartPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.review-requirement-unlink-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.review-requirement-unlink-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.review-requirement-unlink-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 1, 0, 0),
            ApplicationCandidateResourceCeiling::bounded(1024, 1024),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        identity(&key.to_le_bytes())
    }
    fn input_identity(input: &ReviewRequirementInput) -> [u8; 32] {
        identity(
            format!(
                "{}:{}:{}:{}",
                input.resource.len(),
                input.resource,
                input.related.len(),
                input.related
            )
            .as_bytes(),
        )
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

#[derive(Clone)]
pub struct UnlinkReviewRequirementIntent {
    pub input: ReviewRequirementInput,
}
impl ApplicationMutationIntent<BoundedDimensionSchema> for UnlinkReviewRequirementIntent {
    type Binding = UnlinkReviewRequirementBinding;
    fn input(&self) -> &ReviewRequirementInput {
        &self.input
    }
    fn scope_binding(&self) -> ReviewRequirementScope {
        ReviewRequirementScope::new(PartIdentityField::reference(), self.input.resource.clone())
    }
}

pub struct UnlinkReviewRequirementHandler;
impl OperationHandler<BoundedDimensionSchema, UnlinkReviewRequirementBinding>
    for UnlinkReviewRequirementHandler
{
    fn decide(
        &self,
        input: &ReviewRequirementInput,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            BoundedDimensionSchema,
            UnlinkReviewRequirementBinding,
        >,
    ) -> HandlerResult<ReviewRequirementTargets, ReviewRequirementDenial> {
        let resource =
            match reader.resolve_entity(PartIdentityField::reference(), input.resource.clone()) {
                Ok(value) => value,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
        let related =
            match reader.resolve_entity(PartIdentityField::reference(), input.related.clone()) {
                Ok(value) => value,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
        match reader.relations_from(ReviewRequired::reference(), &resource) {
            Ok(relations) if relations.len() == 1 && relations[0].to() == &related => {}
            Ok(_) => return HandlerResult::DomainDenied(ReviewRequirementDenial::RelationMismatch),
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        }
        let resource = match reader.mutation_target(&resource) {
            Ok(value) => value,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let related = match reader.mutation_target(&related) {
            Ok(value) => value,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        HandlerResult::Completed(ReviewRequirementTargets { resource, related })
    }
    fn candidate_requirements(
        &self,
        _: &ReviewRequirementInput,
        _: &ReviewRequirementTargets,
    ) -> ApplicationCandidateRequirements {
        UnlinkReviewRequirementBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        _: &ReviewRequirementInput,
        decision: ReviewRequirementTargets,
        writer: &mut CandidateWriter<'_, BoundedDimensionSchema, UnlinkReviewRequirementBinding>,
    ) -> HandlerResult<ReviewRequirementUnlinked, ReviewRequirementDenial> {
        let resource = match writer.projected_entity(&decision.resource) {
            Ok(value) => value,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        let related = match writer.projected_entity(&decision.related) {
            Ok(value) => value,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match writer.unlink(ReviewRequired::reference(), &resource, &related) {
            Ok(()) => HandlerResult::Completed(ReviewRequirementUnlinked),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema>,
) -> ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema> {
    let operation = UnlinkReviewRequirement::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 16)
        .operation_projection_work_budget(operation, 32)
        .operation_read_field(operation, PartIdentityField::reference())
        .operation_read_relation(operation, ReviewRequired::reference())
        .operation_unlink(operation, ReviewRequired::reference())
        .application_mutation_binding::<UnlinkReviewRequirementBinding>()
}

pub fn unlink_review_requirement(
    application: &BoundedDimensionWorkflowRuntime,
    idempotency: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<ReviewRequirementDenial, ReviewRequirementUnlinked>,
    WorthQueryApplicationRequestMutationDenial,
> {
    unlink_review_requirement_on(
        application,
        application.runtime().current_world(),
        idempotency,
    )
}

/// Issues the same mutation on one exact World product occurrence.
pub fn unlink_review_requirement_on(
    application: &BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
    idempotency: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<ReviewRequirementDenial, ReviewRequirementUnlinked>,
    WorthQueryApplicationRequestMutationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(UnlinkReviewRequirementIntent {
            input: ReviewRequirementInput {
                resource: PART_IDENTITY.to_owned(),
                related: RELATED_PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(application.program_runtime())
}
