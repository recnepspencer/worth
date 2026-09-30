//! A mutation whose output contract declares one exactly-one role and one
//! at-most-one role. The input names which roles the handler binds, so the
//! same installed binding shows an optional output left unbound, bound once,
//! bound twice, and bound through the wrong token, and a required output left
//! unbound.

use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOptionalOutputRole, WorthQueryApplicationOutputRole,
    WorthQueryPreserveOutput,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
    ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaDeclarationBuilder, NoApplicationUnit, ReadWrite, U64ApplicationValueBinding,
};

/// Which output roles the handler binds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub(in crate::domain_computation::primary_graph) enum OptionalOutputPlan {
    RequiredOnly,
    RequiredAndOptional,
    RequiredAndOptionalTwice,
    /// Binds the at-most-one role through an exactly-one token.
    RequiredAndOptionalAsRequired,
    OptionalOnly,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(in crate::domain_computation::primary_graph) struct OptionalOutputInput {
    pub(in crate::domain_computation::primary_graph) status: String,
    pub(in crate::domain_computation::primary_graph) plan: OptionalOutputPlan,
}

worth_query_declaration::worth_query_portable_type!(
    OptionalOutputInput => "worth.query.test.optional-output-input.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub(in crate::domain_computation::primary_graph) OptionalOutputInputBinding for OptionalOutputInput {
        identity: "worth.query.test.optional-output-input.v1"
    }
);
worth_query_declaration::worth_query_operation!(
    pub(in crate::domain_computation::primary_graph) OptionalOutputOperation for IdentityExecutionSchema,
    input OptionalOutputInputBinding
);
worth_query_declaration::worth_query_operation_requires!(OptionalOutputOperation => [ViewAccount]);
worth_query_declaration::worth_query_operation_reads!(OptionalOutputOperation => [AccountStatus]);
worth_query_declaration::worth_query_operation_writes!(OptionalOutputOperation => [AccountStatus]);

/// The role every completed mutation binds exactly once.
pub(in crate::domain_computation::primary_graph) const SUBJECT_OUTPUT:
    WorthQueryApplicationOutputRole<
        OptionalOutputMutationBinding,
        Account,
        WorthQueryPreserveOutput,
    > = WorthQueryApplicationOutputRole::for_entity::<IdentityExecutionSchema>("subject");
/// The role a completed mutation binds at most once.
pub(in crate::domain_computation::primary_graph) const COMPANION_OUTPUT:
    WorthQueryApplicationOptionalOutputRole<
        OptionalOutputMutationBinding,
        Account,
        WorthQueryPreserveOutput,
    > = WorthQueryApplicationOptionalOutputRole::for_entity::<IdentityExecutionSchema>("companion");
/// A stray exactly-one declaration of the at-most-one role's name. The output
/// contract never lists it, so Query must refuse it by cardinality.
pub(in crate::domain_computation::primary_graph) const COMPANION_AS_REQUIRED_OUTPUT:
    WorthQueryApplicationOutputRole<
        OptionalOutputMutationBinding,
        Account,
        WorthQueryPreserveOutput,
    > = WorthQueryApplicationOutputRole::for_entity::<IdentityExecutionSchema>("companion");

pub(in crate::domain_computation::primary_graph) struct OptionalOutputs;

impl ApplicationMutationOutputContract<IdentityExecutionSchema> for OptionalOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[SUBJECT_OUTPUT.descriptor(), COMPANION_OUTPUT.descriptor()];
}

worth_query_declaration::worth_query_mutation_binding!(
    pub(in crate::domain_computation::primary_graph) OptionalOutputMutationBinding for OptionalOutputInput,
    schema IdentityExecutionSchema,
    identity "worth.query.test.optional-output-mutation.v1",
    input OptionalOutputInputBinding,
    operation OptionalOutputOperation,
    result OptionalOutputInputBinding,
    idempotency String, identity "worth.query.test.optional-output-key.v1",
    decision OptionalOutputInput,
    denial OptionalOutputInputBinding,
    handler identity "worth.query.test.optional-output-handler.v1",
    outputs OptionalOutputs,
    principal IdentityBinding,
        mapping ExternalMapping,
        principal_entity Principal,
        principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(),
    value optional_output_scope,
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 1, emits 0,
    resources retained_representation_bytes 128
);

fn optional_output_scope(input: &OptionalOutputInput) -> String {
    input.status.clone()
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let operation = OptionalOutputOperation::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 1)
        .operation_projection_work_budget(operation, 8)
        .operation_requires_ability(operation, ViewAccount::reference())
        .operation_write(operation, AccountStatus::reference())
        .operation_read_field(operation, AccountStatus::reference())
        .application_mutation_binding::<OptionalOutputMutationBinding>()
}

/// Rewrites the scoped account's status and binds the roles its input plans.
pub(super) struct OptionalOutputHandler;

impl OperationHandler<IdentityExecutionSchema, OptionalOutputMutationBinding>
    for OptionalOutputHandler
{
    fn decide(
        &self,
        input: &OptionalOutputInput,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            IdentityExecutionSchema,
            OptionalOutputMutationBinding,
        >,
    ) -> HandlerResult<OptionalOutputInput, OptionalOutputInput> {
        // The decision observes the scoped account, so the candidate may target it.
        match reader.resolve_entity(AccountStatus::reference(), input.status.clone()) {
            Ok(_) => HandlerResult::Completed(input.clone()),
            Err(denial) => HandlerResult::ExecutionDenied(denial),
        }
    }

    fn candidate_requirements(
        &self,
        _: &OptionalOutputInput,
        _: &OptionalOutputInput,
    ) -> ApplicationCandidateRequirements {
        <OptionalOutputMutationBinding as ApplicationMutationBinding<IdentityExecutionSchema>>::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &OptionalOutputInput,
        decision: OptionalOutputInput,
        writer: &mut CandidateWriter<'_, IdentityExecutionSchema, OptionalOutputMutationBinding>,
    ) -> HandlerResult<OptionalOutputInput, OptionalOutputInput> {
        let (required, optional) = match decision.plan {
            OptionalOutputPlan::RequiredOnly => (true, 0),
            OptionalOutputPlan::RequiredAndOptional => (true, 1),
            OptionalOutputPlan::RequiredAndOptionalTwice => (true, 2),
            OptionalOutputPlan::RequiredAndOptionalAsRequired => (true, 0),
            OptionalOutputPlan::OptionalOnly => (false, 1),
        };
        let misnamed = decision.plan == OptionalOutputPlan::RequiredAndOptionalAsRequired;
        let bound = writer
            .resolve_entity(AccountStatus::reference(), decision.status.clone())
            .and_then(|account| {
                writer.write_field(&account, AccountStatus::reference(), "reviewed".to_owned())?;
                if required {
                    writer.preserve_output(SUBJECT_OUTPUT, &account)?;
                }
                for _ in 0..optional {
                    writer.preserve_output(COMPANION_OUTPUT, &account)?;
                }
                if misnamed {
                    writer.preserve_output(COMPANION_AS_REQUIRED_OUTPUT, &account)?;
                }
                Ok(())
            });
        match bound {
            Ok(()) => HandlerResult::Completed(decision),
            Err(denial) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial)),
        }
    }
}
