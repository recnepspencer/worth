//! One decision-read operation, its principal-scoped mutation binding and the
//! handler that completes without writing.
use crate::domain_computation::primary_graph::tests::fixture::decision_read_world::queries::*;
use crate::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::*;
use worth_query_declaration::facade::application_operation::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_execution::facade::domain_computation::CandidateWriter;
use worth_query_execution::facade::domain_computation::DecisionReader;
use worth_query_execution::facade::domain_computation::HandlerResult;
use worth_query_execution::facade::domain_computation::OperationHandler;

worth_query_declaration::worth_query_operation!(pub DecisionReadOperation for DecisionReadSchema, input UnitBinding);
worth_query_declaration::worth_query_operation_reads!(DecisionReadOperation => [PrincipalIdentity]);

pub struct DecisionReadBinding;
impl ApplicationMutationAdmissionBinding<DecisionReadSchema> for DecisionReadBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DecisionReadSchema> for DecisionReadBinding {
    type Input = ();
    type InputBinding = UnitBinding;
    type Operation = DecisionReadOperation;
    type Result = ();
    type ResultBinding = UnitBinding;
    type IdempotencyKey = u64;
    type Decision = ();
    type Denial = ();
    type DenialBinding = UnitBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationPrincipalScope<
        DecisionReadSchema,
        Principal,
        PrincipalFacts,
        PrincipalIdentity,
        u64,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = DecisionPrincipalBinding;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.tests.decision-read-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.tests.decision-read-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.tests.decision-read-key.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        DecisionReadSchema,
        Principal,
        PrincipalFacts,
        PrincipalIdentity,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        PrincipalIdentity::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DecisionReadSchema,
        DecisionPrincipalBinding,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DecisionPrincipalBinding::reference()
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DecisionReadSchema>,
) -> ApplicationSchemaDeclarationBuilder<DecisionReadSchema> {
    let operation = DecisionReadOperation::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(operation, PrincipalIdentity::reference())
        .operation_managed_index_read::<ParentBinding, _, _>(operation)
        .application_mutation_binding::<DecisionReadBinding>()
}

/// Reaching `decide` is the whole observation; nothing is read or written.
pub struct DecisionReadHandler;
impl OperationHandler<DecisionReadSchema, DecisionReadBinding> for DecisionReadHandler {
    fn decide(
        &self,
        _: &(),
        _: &mut DecisionReader<'_, '_, '_, DecisionReadSchema, DecisionReadBinding>,
    ) -> HandlerResult<(), ()> {
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        _: &(),
        _: (),
        _: &mut CandidateWriter<'_, DecisionReadSchema, DecisionReadBinding>,
    ) -> HandlerResult<(), ()> {
        HandlerResult::Completed(())
    }
}
