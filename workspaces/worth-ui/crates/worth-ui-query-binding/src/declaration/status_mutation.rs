use worth_query_decl::facade::{
    application_operation::{
        ApplicationConventionalAdmission, ApplicationMutationAdmissionBinding,
        ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationIntent,
        ApplicationQueryMutationSource, ApplicationWorkflowExecutionRequirements,
        NoApplicationMutationOutputs,
    },
    application_schema::{
        ApplicationPrincipalBindingRef, ApplicationSchemaDeclarationBuilder,
        U64ApplicationValueBinding,
    },
    worth_query_operation, worth_query_operation_emits, worth_query_operation_reads,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

use super::{
    IdentityIdField, QueryRevisionValueField, QueryTextStatusField, WorthUiApplicationSchema,
    WorthUiExternalPrincipalMapping, WorthUiPrincipal, WorthUiPrincipalBinding, WorthUiRecord,
    WorthUiStatusChanged, WorthUiStatusChangedEffect, WorthUiStatusQuery,
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
worth_query_operation_reads!(WorthUiUpdateStatus => [WorthUiRecord]);

worth_query_operation_emits!(WorthUiUpdateStatus => [WorthUiStatusChangedEffect]);

pub struct WorthUiStatusUpdateBinding;

pub(super) type StatusMutationScope = ApplicationMutationFieldScope<IdentityIdField>;

impl ApplicationMutationAdmissionBinding<WorthUiApplicationSchema> for WorthUiStatusUpdateBinding {
    type Admission = ApplicationConventionalAdmission;
}

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
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    fn scope_field() -> IdentityIdField {
        IdentityIdField::at("IdentityIdField")
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
            IdentityIdField::at("IdentityIdField"),
            "platform.pulse.status".to_owned(),
        )
    }
}

pub struct WorthUiStatusUpdateHandler;

impl OperationHandler<WorthUiApplicationSchema, WorthUiStatusUpdateBinding>
    for WorthUiStatusUpdateHandler
{
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
            IdentityIdField::at("IdentityIdField"),
            "platform.pulse.status".to_owned(),
        ) {
            Ok(entity) => entity,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let current = match reader.field(
            &entity,
            QueryRevisionValueField::at("QueryRevisionValueField"),
        ) {
            Ok(Some(current)) => current,
            Ok(None) => {
                return HandlerResult::DomainDenied(WorthUiStatusUpdateDenial::StaleRevision);
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        if let Err(error) = reader.field(&entity, QueryTextStatusField::at("QueryTextStatusField"))
        {
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
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        };
        if let Err(error) = writer.write_field(
            &entity,
            QueryTextStatusField::at("QueryTextStatusField"),
            input.status().to_owned(),
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        if let Err(error) = writer.write_field(
            &entity,
            QueryRevisionValueField::at("QueryRevisionValueField"),
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
        .operation_projection_work_budget(operation, 16)
        .operation_read_entity(operation, WorthUiRecord::reference())
        .operation_read_field(operation, IdentityIdField::at("IdentityIdField"))
        .operation_read_field(operation, QueryTextStatusField::at("QueryTextStatusField"))
        .operation_read_field(
            operation,
            QueryRevisionValueField::at("QueryRevisionValueField"),
        )
        .operation_write(operation, QueryTextStatusField::at("QueryTextStatusField"))
        .operation_write(
            operation,
            QueryRevisionValueField::at("QueryRevisionValueField"),
        )
        .operation_emit(operation, WorthUiStatusChangedEffect::reference())
        .application_mutation_binding::<WorthUiStatusUpdateBinding>()
}
