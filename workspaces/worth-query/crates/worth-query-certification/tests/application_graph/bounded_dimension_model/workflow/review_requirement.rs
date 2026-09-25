//! A public fixture mutation inserts the native authored applicability relation.

use worth_query_decl::facade::{worth_query_operation_links, worth_query_operation_reads};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
    ApplicationCandidateResourceCeiling, ApplicationMutationBinding, ApplicationMutationFieldScope,
    ApplicationMutationIntent, NoApplicationMutationOutputs, NoApplicationMutationSource,
};
use worth_query_host::facade::declaration::application_schema::{
    ApplicationFieldRef, ApplicationPrincipalBindingRef, ApplicationRelationCardinality,
    ApplicationRelationIntegrity, ApplicationSchemaDeclarationBuilder, EqualityPredicate,
    NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};
use worth_query_host::facade::{
    worth_query_operation, worth_query_relation, worth_query_structured_value_binding,
};

use super::super::dimension_entry::{PART_IDENTITY, RELATED_PART_IDENTITY};
use super::super::host::BoundedDimensionWorkflowRuntime;
use super::super::operator_identity::{authenticate_operator, request_scope};
use super::super::schema::{
    BoundedDimensionSchema, ExternalMapping, Part, PartFacts, PartIdentityField,
    PartPrincipalBinding, Principal,
};

worth_query_relation!(pub ReviewRequired in BoundedDimensionSchema, Part => Part; integrity = ApplicationRelationIntegrity {
    cardinality: ApplicationRelationCardinality::new(None, Some(1), None, None, None, Some(1)),
    ..ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
});

#[derive(Clone, Debug)]
pub struct ReviewRequirementInput {
    pub resource: String,
    pub related: String,
}
worth_query_structured_value_binding!(pub ReviewRequirementInputBinding for ReviewRequirementInput {
    identity: "worth.query.certification.review-requirement-input.v1"
});
worth_query_operation!(pub LinkReviewRequirement for BoundedDimensionSchema, input ReviewRequirementInputBinding);
worth_query_operation_reads!(LinkReviewRequirement => [PartIdentityField]);
worth_query_operation_links!(LinkReviewRequirement => [ReviewRequired]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewRequirementLinked;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewRequirementDenial {}
worth_query_structured_value_binding!(pub ReviewRequirementLinkedBinding for ReviewRequirementLinked {
    identity: "worth.query.certification.review-requirement-linked.v1"
});
worth_query_structured_value_binding!(pub ReviewRequirementDenialBinding for ReviewRequirementDenial {
    identity: "worth.query.certification.review-requirement-denial.v1"
});

pub struct ReviewRequirementBinding;
type ReviewRequirementScope = ApplicationMutationFieldScope<
    BoundedDimensionSchema,
    Part,
    PartFacts,
    PartIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
pub struct ReviewRequirementTargets {
    resource: WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
    related: WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
}

impl ApplicationMutationBinding<BoundedDimensionSchema> for ReviewRequirementBinding {
    type Input = ReviewRequirementInput;
    type InputBinding = ReviewRequirementInputBinding;
    type Result = ReviewRequirementLinked;
    type ResultBinding = ReviewRequirementLinkedBinding;
    type IdempotencyKey = u64;
    type Operation = LinkReviewRequirement;
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

    const IDENTITY: &'static str = "worth.query.certification.review-requirement-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.review-requirement-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.review-requirement-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 1, 0, 0, 0),
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
pub struct ReviewRequirementIntent {
    pub input: ReviewRequirementInput,
}
impl ApplicationMutationIntent<BoundedDimensionSchema> for ReviewRequirementIntent {
    type Binding = ReviewRequirementBinding;
    fn input(&self) -> &ReviewRequirementInput {
        &self.input
    }
    fn scope_binding(&self) -> ReviewRequirementScope {
        ReviewRequirementScope::new(PartIdentityField::reference(), self.input.resource.clone())
    }
}

pub struct ReviewRequirementHandler;
impl OperationHandler<BoundedDimensionSchema, ReviewRequirementBinding>
    for ReviewRequirementHandler
{
    fn decide(
        &self,
        input: &ReviewRequirementInput,
        reader: &mut DecisionReader<'_, '_, '_, BoundedDimensionSchema, ReviewRequirementBinding>,
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
        ReviewRequirementBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        _: &ReviewRequirementInput,
        decision: ReviewRequirementTargets,
        writer: &mut CandidateWriter<'_, BoundedDimensionSchema, ReviewRequirementBinding>,
    ) -> HandlerResult<ReviewRequirementLinked, ReviewRequirementDenial> {
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
        match writer.link(
            ReviewRequired::reference(),
            "review-required",
            &resource,
            &related,
        ) {
            Ok(()) => HandlerResult::Completed(ReviewRequirementLinked),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema>,
) -> ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema> {
    schema
        .relation(
            ReviewRequired::reference(),
            Part::reference(),
            Part::reference(),
        )
        .operation(
            LinkReviewRequirement::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(LinkReviewRequirement::reference(), 16)
        .operation_projection_work_budget(LinkReviewRequirement::reference(), 32)
        .operation_read_field(
            LinkReviewRequirement::reference(),
            PartIdentityField::reference(),
        )
        .operation_link(
            LinkReviewRequirement::reference(),
            ReviewRequired::reference(),
        )
        .application_mutation_binding::<ReviewRequirementBinding>()
}

fn identity(bytes: &[u8]) -> [u8; 32] {
    let mut result = [0; 32];
    for (index, byte) in bytes.iter().enumerate() {
        result[index % 32] ^= byte.wrapping_mul(31);
    }
    result
}

pub fn link_review_requirement(
    application: &BoundedDimensionWorkflowRuntime,
    idempotency: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<ReviewRequirementDenial, ReviewRequirementLinked>,
    WorthQueryApplicationRequestMutationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(ReviewRequirementIntent {
            input: ReviewRequirementInput {
                resource: PART_IDENTITY.to_owned(),
                related: RELATED_PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(application.program_runtime())
}
