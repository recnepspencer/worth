use sha2::{Digest, Sha256};
use worth_query_decl::facade::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling, ApplicationMutationBinding, ApplicationMutationIntent,
        ApplicationQueryMutationSource, NoApplicationMutationOutputs,
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

use super::status_mutation::{
    StatusMutationScope, WorthUiStatusUpdateDenial, WorthUiStatusUpdateDenialBinding,
};
use super::{
    IdentityIdField, QueryRevisionValueField, QueryTextStatusField, WorthUiApplicationSchema,
    WorthUiExternalPrincipalMapping, WorthUiPrincipal, WorthUiPrincipalBinding, WorthUiRecord,
    WorthUiStatusChanged, WorthUiStatusChangedEffect, WorthUiStatusQuery,
};
use crate::{WorthUiStatusActionIdentity, WorthUiStatusActionRequest};

worth_query_structured_value_binding!(pub WorthUiStatusActionInputBinding for WorthUiStatusActionRequest {
    identity: "worth.ui.status-action-input.v1"
});
worth_query_structured_value_binding!(pub WorthUiStatusActionResultBinding for WorthUiStatusActionRequest {
    identity: "worth.ui.status-action-result.v1"
});
worth_query_operation!(pub WorthUiApplyStatusAction for WorthUiApplicationSchema,
    input WorthUiStatusActionInputBinding);
worth_query_operation_reads!(WorthUiApplyStatusAction => [WorthUiRecord, IdentityIdField, QueryTextStatusField, QueryRevisionValueField]);
worth_query_operation_writes!(WorthUiApplyStatusAction => [QueryTextStatusField]);
worth_query_operation_emits!(WorthUiApplyStatusAction => [WorthUiStatusChangedEffect]);

pub struct WorthUiStatusActionBinding;

impl ApplicationMutationBinding<WorthUiApplicationSchema> for WorthUiStatusActionBinding {
    type Input = WorthUiStatusActionRequest;
    type InputBinding = WorthUiStatusActionInputBinding;
    type Result = WorthUiStatusActionRequest;
    type ResultBinding = WorthUiStatusActionResultBinding;
    type IdempotencyKey = WorthUiStatusActionIdentity;
    type Operation = WorthUiApplyStatusAction;
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

    const IDENTITY: &'static str = "worth.ui.status-action.v1";
    const HANDLER_IDENTITY: &'static str = "worth.ui.status-action-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.ui.status-action-command.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 1, 1),
            ApplicationCandidateResourceCeiling::bounded(65_544, 256),
        );

    fn idempotency_key_identity(key: &WorthUiStatusActionIdentity) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth.ui.status-action-command.v1");
        digest.update(key.session().to_le_bytes());
        digest.update(key.lineage().to_le_bytes());
        digest.finalize().into()
    }

    fn input_identity(input: &WorthUiStatusActionRequest) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth.ui.status-action-input.v1");
        digest.update(input.source_revision().to_le_bytes());
        digest.update((input.status().len() as u64).to_le_bytes());
        digest.update(input.status().as_bytes());
        digest.update(input.identity().session().to_le_bytes());
        digest.update(input.identity().lineage().to_le_bytes());
        digest.finalize().into()
    }

    fn scope_field() -> ApplicationFieldRef<
        WorthUiApplicationSchema,
        WorthUiRecord,
        super::IdentityAspect,
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

impl ApplicationMutationIntent<WorthUiApplicationSchema> for WorthUiStatusActionRequest {
    type Binding = WorthUiStatusActionBinding;

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

pub struct WorthUiStatusActionHandler;

impl OperationHandler<WorthUiApplicationSchema, WorthUiStatusActionBinding>
    for WorthUiStatusActionHandler
{
    fn candidate_requirements(
        &self,
        _: &WorthUiStatusActionRequest,
        _: &WorthQueryInvariantMutationTarget<WorthUiApplicationSchema, WorthUiRecord>,
    ) -> ApplicationCandidateRequirements {
        WorthUiStatusActionBinding::CANDIDATES
    }

    fn decide(
        &self,
        input: &WorthUiStatusActionRequest,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            WorthUiApplicationSchema,
            WorthUiStatusActionBinding,
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
        if input.source_revision() != current {
            return HandlerResult::DomainDenied(WorthUiStatusUpdateDenial::StaleRevision);
        }
        if let Err(error) = reader.field(&entity, QueryTextStatusField::reference()) {
            return HandlerResult::ExecutionDenied(error);
        }
        match reader.mutation_target(&entity) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn build_candidate(
        &self,
        input: &WorthUiStatusActionRequest,
        target: WorthQueryInvariantMutationTarget<WorthUiApplicationSchema, WorthUiRecord>,
        writer: &mut CandidateWriter<'_, WorthUiApplicationSchema, WorthUiStatusActionBinding>,
    ) -> HandlerResult<WorthUiStatusActionRequest, WorthUiStatusUpdateDenial> {
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

pub(crate) fn declare_status_action(
    schema: ApplicationSchemaDeclarationBuilder<WorthUiApplicationSchema>,
) -> ApplicationSchemaDeclarationBuilder<WorthUiApplicationSchema> {
    let operation = WorthUiApplyStatusAction::reference();
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
        .operation_emit(operation, WorthUiStatusChangedEffect::reference())
        .application_mutation_binding::<WorthUiStatusActionBinding>()
}
