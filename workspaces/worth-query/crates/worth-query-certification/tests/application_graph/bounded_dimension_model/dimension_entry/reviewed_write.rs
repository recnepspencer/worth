//! The guarded counterpart of the ordinary dimension write used by workflow courts.

use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationCandidateRequirements, ApplicationMutationBinding,
            ApplicationMutationIntent, NoApplicationMutationOutputs, NoApplicationMutationSource,
        },
        application_schema::{
            ApplicationFieldRef, ApplicationPrincipalBindingRef, EqualityPredicate,
            NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
        },
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget,
    },
};

use super::super::schema::{
    BoundedDimensionSchema, ExternalMapping, Part, PartDimensionField, PartFacts,
    PartIdentityField, PartPrincipalBinding, Principal, SetPartDimension,
    SetPartDimensionInputBinding,
};
use super::{
    PartDimensionWritten, SetPartDimensionBinding, SetPartDimensionDenial, SetPartDimensionInput,
    SetPartDimensionScope,
};

#[derive(Clone, Debug)]
pub struct ReviewedSetPartDimensionIntent {
    pub input: SetPartDimensionInput,
}

pub struct ReviewedSetPartDimensionBinding;

impl ApplicationMutationBinding<BoundedDimensionSchema> for ReviewedSetPartDimensionBinding {
    type Input = SetPartDimensionInput;
    type InputBinding = SetPartDimensionInputBinding;
    type Result = PartDimensionWritten;
    type ResultBinding = super::PartDimensionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = SetPartDimension;
    type Decision = WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>;
    type Denial = SetPartDimensionDenial;
    type DenialBinding = super::SetPartDimensionDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = SetPartDimensionScope;
    type PrincipalBinding = PartPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str =
        "worth.query.certification.bounded-dimension.reviewed-set-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.bounded-dimension.reviewed-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.bounded-dimension.reviewed-command.v1";
    const REQUIRES_WORKFLOW_AUTHORITY: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements = SetPartDimensionBinding::CANDIDATES;

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        SetPartDimensionBinding::idempotency_key_identity(key)
    }

    fn input_identity(input: &SetPartDimensionInput) -> [u8; 32] {
        SetPartDimensionBinding::input_identity(input)
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

impl ApplicationMutationIntent<BoundedDimensionSchema> for ReviewedSetPartDimensionIntent {
    type Binding = ReviewedSetPartDimensionBinding;

    fn input(&self) -> &SetPartDimensionInput {
        &self.input
    }

    fn scope_binding(&self) -> SetPartDimensionScope {
        SetPartDimensionScope::new(PartIdentityField::reference(), self.input.identity.clone())
    }
}

pub struct ReviewedSetPartDimensionHandler;

impl OperationHandler<BoundedDimensionSchema, ReviewedSetPartDimensionBinding>
    for ReviewedSetPartDimensionHandler
{
    fn decide(
        &self,
        input: &SetPartDimensionInput,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            BoundedDimensionSchema,
            ReviewedSetPartDimensionBinding,
        >,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        SetPartDimensionDenial,
    > {
        let part =
            match reader.resolve_entity(PartIdentityField::reference(), input.identity.clone()) {
                Ok(part) => part,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
        match reader.field(&part, PartDimensionField::reference()) {
            Ok(Some(current)) if current == input.dimension => {
                return HandlerResult::DomainDenied(SetPartDimensionDenial::DimensionUnchanged)
            }
            Ok(_) => {}
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&part) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &SetPartDimensionInput,
        _: &WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
    ) -> ApplicationCandidateRequirements {
        ReviewedSetPartDimensionBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &SetPartDimensionInput,
        target: WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        writer: &mut CandidateWriter<'_, BoundedDimensionSchema, ReviewedSetPartDimensionBinding>,
    ) -> HandlerResult<PartDimensionWritten, SetPartDimensionDenial> {
        super::candidate_tracking::record_candidate(input.dimension);
        let part = match writer.projected_entity(&target) {
            Ok(part) => part,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) =
            writer.write_field(&part, PartDimensionField::reference(), input.dimension)
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(PartDimensionWritten {
            dimension: input.dimension,
        })
    }
}
