use sha2::{Digest, Sha256};
use worth_query_decl::facade::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling, ApplicationMutationBinding,
        ApplicationMutationFieldScope, ApplicationMutationIntent, ApplicationQueryMutationSource,
        NoApplicationMutationOutputs,
    },
    application_schema::{
        ApplicationFieldRef, ApplicationPrincipalBindingRef, ApplicationSchemaDeclarationBuilder,
        EqualityPredicate, NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
    },
    worth_query_operation, worth_query_operation_emits, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

use super::{
    IdentityAspect, IdentityIdField, QueryRevisionValueField, QueryTextStatusField,
    WorthUiApplicationSchema, WorthUiExternalPrincipalMapping, WorthUiPrincipal,
    WorthUiPrincipalBinding, WorthUiRecord, WorthUiStatusChanged, WorthUiStatusChangedEffect,
    WorthUiStatusQuery,
};
use crate::WorthUiScalarProjectionSourceRecord;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiStatusUpdateDenial {
    StaleRevision,
    RevisionMismatch,
}

worth_query_structured_value_binding!(pub WorthUiStatusUpdateInputBinding for WorthUiScalarProjectionSourceRecord {
    identity: "worth.ui.status-update-input.v1"
});
worth_query_structured_value_binding!(pub WorthUiStatusUpdateResultBinding for WorthUiScalarProjectionSourceRecord {
    identity: "worth.ui.status-update-result.v1"
});
worth_query_structured_value_binding!(pub WorthUiStatusUpdateDenialBinding for WorthUiStatusUpdateDenial {
    identity: "worth.ui.status-update-denial.v1"
});
worth_query_operation!(pub WorthUiUpdateStatus for WorthUiApplicationSchema,
    input WorthUiStatusUpdateInputBinding);
worth_query_operation_reads!(WorthUiUpdateStatus => [WorthUiRecord, IdentityIdField, QueryTextStatusField, QueryRevisionValueField]);
worth_query_operation_writes!(WorthUiUpdateStatus => [QueryTextStatusField, QueryRevisionValueField]);
worth_query_operation_emits!(WorthUiUpdateStatus => [WorthUiStatusChangedEffect]);

pub struct WorthUiStatusUpdateBinding;

pub(super) type StatusMutationScope = ApplicationMutationFieldScope<
    WorthUiApplicationSchema,
    WorthUiRecord,
    IdentityAspect,
    IdentityIdField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<WorthUiApplicationSchema> for WorthUiStatusUpdateBinding {
    type Input = WorthUiScalarProjectionSourceRecord;
    type InputBinding = WorthUiStatusUpdateInputBinding;
    type Result = WorthUiScalarProjectionSourceRecord;
    type ResultBinding = WorthUiStatusUpdateResultBinding;
    type IdempotencyKey = u64;
    type Operation = WorthUiUpdateStatus;
    type Decision = WorthQueryInvariantMutationTarget<WorthUiApplicationSchema, WorthUiRecord>;
    type Denial = WorthUiStatusUpdateDenial;
    type DenialBinding = WorthUiStatusUpdateDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = StatusMutationScope;
    type PrincipalBinding = WorthUiPrincipalBinding;
    type Mapping = WorthUiExternalPrincipalMapping;
    type Principal = WorthUiPrincipal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<WorthUiStatusQuery>;

    const IDENTITY: &'static str = "worth.ui.status-update.v1";
    const HANDLER_IDENTITY: &'static str = "worth.ui.status-update-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.ui.status-update-command.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 1),
            ApplicationCandidateResourceCeiling::bounded(65_544, 256),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth.ui.status-update-command.v1");
        digest.update(key.to_le_bytes());
        digest.finalize().into()
    }

    fn input_identity(input: &WorthUiScalarProjectionSourceRecord) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth.ui.status-update-input.v1");
        digest.update(input.revision().to_le_bytes());
        digest.update((input.status().len() as u64).to_le_bytes());
        digest.update(input.status().as_bytes());
        digest.finalize().into()
    }

    fn scope_field() -> ApplicationFieldRef<
        WorthUiApplicationSchema,
        WorthUiRecord,
        IdentityAspect,
        IdentityIdField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        IdentityIdField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        WorthUiApplicationSchema,
        WorthUiPrincipalBinding,
        WorthUiExternalPrincipalMapping,
        WorthUiPrincipal,
        u64,
        U64ApplicationValueBinding,
    > {
        WorthUiPrincipalBinding::reference()
    }
}

impl ApplicationMutationIntent<WorthUiApplicationSchema> for WorthUiScalarProjectionSourceRecord {
    type Binding = WorthUiStatusUpdateBinding;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> StatusMutationScope {
        StatusMutationScope::new(
            IdentityIdField::reference(),
            "platform.pulse.status".to_owned(),
        )
    }
}

pub struct WorthUiStatusUpdateHandler;

impl OperationHandler<WorthUiApplicationSchema, WorthUiStatusUpdateBinding>
    for WorthUiStatusUpdateHandler
{
    fn candidate_requirements(
        &self,
        _: &WorthUiScalarProjectionSourceRecord,
        _: &WorthQueryInvariantMutationTarget<WorthUiApplicationSchema, WorthUiRecord>,
    ) -> ApplicationCandidateRequirements {
        WorthUiStatusUpdateBinding::CANDIDATES
    }

    fn decide(
        &self,
        input: &WorthUiScalarProjectionSourceRecord,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            WorthUiApplicationSchema,
            WorthUiStatusUpdateBinding,
        >,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<WorthUiApplicationSchema, WorthUiRecord>,
        WorthUiStatusUpdateDenial,
    > {
        let entity = match reader.resolve_entity(
            IdentityIdField::reference(),
            "platform.pulse.status".to_owned(),
        ) {
            Ok(entity) => entity,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let current = match reader.field(&entity, QueryRevisionValueField::reference()) {
            Ok(Some(current)) => current,
            Ok(None) => {
                return HandlerResult::DomainDenied(WorthUiStatusUpdateDenial::StaleRevision)
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        if let Err(error) = reader.field(&entity, QueryTextStatusField::reference()) {
            return HandlerResult::ExecutionDenied(error);
        }
        if input.revision() <= current {
            return HandlerResult::DomainDenied(WorthUiStatusUpdateDenial::StaleRevision);
        }
        match reader.mutation_target(&entity) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn build_candidate(
        &self,
        input: &WorthUiScalarProjectionSourceRecord,
        target: WorthQueryInvariantMutationTarget<WorthUiApplicationSchema, WorthUiRecord>,
        writer: &mut CandidateWriter<'_, WorthUiApplicationSchema, WorthUiStatusUpdateBinding>,
    ) -> HandlerResult<WorthUiScalarProjectionSourceRecord, WorthUiStatusUpdateDenial> {
        let entity = match writer.projected_entity(&target) {
            Ok(entity) => entity,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.write_field(
            &entity,
            QueryTextStatusField::reference(),
            input.status().to_owned(),
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        if let Err(error) = writer.write_field(
            &entity,
            QueryRevisionValueField::reference(),
            input.revision(),
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        if let Err(error) = writer.emit(
            WorthUiStatusChangedEffect::reference(),
            WorthUiStatusChanged {
                identity: "platform.pulse.status".to_owned(),
            },
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(input.clone())
    }
}

pub(crate) fn declare_status_mutation(
    schema: ApplicationSchemaDeclarationBuilder<WorthUiApplicationSchema>,
) -> ApplicationSchemaDeclarationBuilder<WorthUiApplicationSchema> {
    let operation = WorthUiUpdateStatus::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 16)
        .operation_projection_work_budget(operation, 16)
        .operation_read_entity(operation, WorthUiRecord::reference())
        .operation_read_field(operation, IdentityIdField::reference())
        .operation_read_field(operation, QueryTextStatusField::reference())
        .operation_read_field(operation, QueryRevisionValueField::reference())
        .operation_write(operation, QueryTextStatusField::reference())
        .operation_write(operation, QueryRevisionValueField::reference())
        .operation_emit(operation, WorthUiStatusChangedEffect::reference())
        .application_mutation_binding::<WorthUiStatusUpdateBinding>()
}
