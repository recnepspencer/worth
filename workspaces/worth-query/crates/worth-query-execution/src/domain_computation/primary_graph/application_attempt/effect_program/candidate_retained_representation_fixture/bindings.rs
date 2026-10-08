//! Real typed handlers for the exact candidate contracts under test.

use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use crate::domain_computation::primary_graph::tests::fixture::MutationFreeNotice;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap;
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding, NoApplicationMutationOutputs,
};
use worth_query_declaration::facade::application_schema::{
    NoApplicationUnit, ReadWrite, U64ApplicationValueBinding,
};

worth_query_declaration::worth_query_mutation_binding!(
    pub ReservedEmissionBinding for MutationFreeEmitInput,
    schema ReservationSchema,
    identity "worth.query.test.reserved-emission.v1",
    input MutationFreeEmitInputBinding,
    operation MutationFreeEmitOperation,
    result MutationFreeEmitInputBinding,
    idempotency String, identity "worth.query.test.reserved-emission-key.v1",
    decision MutationFreeEmitInput,
    denial MutationFreeEmitInputBinding,
    handler identity "worth.query.test.reserved-emission-handler.v1",
    outputs NoApplicationMutationOutputs,
    principal IdentityBinding,
        mapping ExternalMapping,
        principal_entity Principal,
        principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(),
    value emission_scope,
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 0, emits 1,
    resources retained_representation_bytes 2
);

fn emission_scope(_: &MutationFreeEmitInput) -> String {
    "open".to_owned()
}

worth_query_declaration::worth_query_mutation_binding!(
    pub ReservedDeletionBinding for TouchAccountInput,
    schema ReservationSchema,
    identity "worth.query.test.reserved-deletion.v1",
    input TouchAccountInputBinding,
    operation TouchAccountOperation,
    result TouchAccountInputBinding,
    idempotency String, identity "worth.query.test.reserved-deletion-key.v1",
    decision TouchAccountInput,
    denial TouchAccountInputBinding,
    handler identity "worth.query.test.reserved-deletion-handler.v1",
    outputs NoApplicationMutationOutputs,
    principal IdentityBinding,
        mapping ExternalMapping,
        principal_entity Principal,
        principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(),
    value deletion_scope,
    candidates creates 0, deletes 1, links 0, unlinks 0, writes 0, emits 0,
    resources retained_representation_bytes 0
);

fn deletion_scope(_: &TouchAccountInput) -> String {
    "open".to_owned()
}

pub(super) fn install_handlers(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<
        ReservationSchema,
    >,
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<ReservationSchema>,
) {
    bootstrap
        .install_handler(
            &schema
                .installed_mutation_binding::<ReservedEmissionBinding>()
                .unwrap(),
            ReservedEmissionHandler,
        )
        .unwrap();
    bootstrap
        .install_handler(
            &schema
                .installed_mutation_binding::<ReservedDeletionBinding>()
                .unwrap(),
            ReservedDeletionHandler,
        )
        .unwrap();
}

struct ReservedEmissionHandler;
impl OperationHandler<ReservationSchema, ReservedEmissionBinding> for ReservedEmissionHandler {
    fn decide(
        &self,
        input: &MutationFreeEmitInput,
        _: &mut DecisionReader<'_, '_, '_, ReservationSchema, ReservedEmissionBinding>,
    ) -> HandlerResult<MutationFreeEmitInput, MutationFreeEmitInput> {
        HandlerResult::Completed(input.clone())
    }
    fn candidate_requirements(
        &self,
        _: &MutationFreeEmitInput,
        _: &MutationFreeEmitInput,
    ) -> ApplicationCandidateRequirements {
        ReservedEmissionBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        _: &MutationFreeEmitInput,
        decision: MutationFreeEmitInput,
        candidate: &mut CandidateWriter<'_, ReservationSchema, ReservedEmissionBinding>,
    ) -> HandlerResult<MutationFreeEmitInput, MutationFreeEmitInput> {
        match candidate.emit_external(
            MutationFreeExternalEffect::reference(),
            MutationFreeNotice(1),
        ) {
            Ok(()) => HandlerResult::Completed(decision),
            Err(denial) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial)),
        }
    }
}

struct ReservedDeletionHandler;
impl OperationHandler<ReservationSchema, ReservedDeletionBinding> for ReservedDeletionHandler {
    fn decide(
        &self,
        input: &TouchAccountInput,
        reader: &mut DecisionReader<'_, '_, '_, ReservationSchema, ReservedDeletionBinding>,
    ) -> HandlerResult<TouchAccountInput, TouchAccountInput> {
        match reader.resolve_entity(AccountStatus::reference(), deletion_scope(input)) {
            Ok(_) => HandlerResult::Completed(input.clone()),
            Err(denial) => HandlerResult::ExecutionDenied(denial),
        }
    }
    fn candidate_requirements(
        &self,
        _: &TouchAccountInput,
        _: &TouchAccountInput,
    ) -> ApplicationCandidateRequirements {
        ReservedDeletionBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &TouchAccountInput,
        decision: TouchAccountInput,
        candidate: &mut CandidateWriter<'_, ReservationSchema, ReservedDeletionBinding>,
    ) -> HandlerResult<TouchAccountInput, TouchAccountInput> {
        let result = candidate
            .resolve_entity(AccountStatus::reference(), deletion_scope(input))
            .and_then(|account| candidate.delete_entity(Account::reference(), &account));
        match result {
            Ok(()) => HandlerResult::Completed(decision),
            Err(denial) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial)),
        }
    }
}
