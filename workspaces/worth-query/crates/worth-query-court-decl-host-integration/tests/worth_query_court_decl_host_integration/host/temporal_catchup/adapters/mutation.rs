//! Ordinary application mutation handler for installing, cancelling, and amending reminders.

use worth_query_decl::facade::{
    application_operation::*,
    application_schema::{
        fragment::Registration, ApplicationPrincipalBindingRef,
        ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
        U64ApplicationValueBinding,
    },
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget, WorthQueryOperationDurableCapability,
};

use super::super::schema::*;

pub struct MutateReminderBinding;
impl Registration<DesktopReminderSchema> for MutateReminderBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
pub type ReminderScope = ApplicationMutationFieldScope<ReminderStringField>;
impl ApplicationMutationAdmissionBinding<DesktopReminderSchema> for MutateReminderBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<DesktopReminderSchema> for MutateReminderBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = MutateReminderInput;
    type InputBinding = MutateReminderInputBinding;
    type Result = u64;
    type ResultBinding = MutateReminderResultBinding;
    type IdempotencyKey = u64;
    type Operation = MutateReminder;
    type Decision = WorthQueryInvariantMutationTarget<DesktopReminderSchema, ReminderIntent>;
    type Denial = ();
    type DenialBinding = MutateReminderDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ReminderScope;
    type PrincipalBinding = ReminderPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.court.desktop-reminder.mutate.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.court.desktop-reminder.mutate-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.court.desktop-reminder.mutate-cmd.v1";
    fn scope_field() -> ReminderStringField {
        identity_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DesktopReminderSchema,
        ReminderPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ReminderPrincipalBinding::reference()
    }
}

#[derive(Clone)]
pub struct MutateReminderIntent(pub MutateReminderInput);
impl ApplicationMutationIntent<DesktopReminderSchema> for MutateReminderIntent {
    type Binding = MutateReminderBinding;
    fn input(&self) -> &MutateReminderInput {
        &self.0
    }
    fn scope_binding(&self) -> ReminderScope {
        ReminderScope::new(identity_field(), self.0.identity.clone())
    }
}

pub struct MutateReminderHandler;
impl OperationHandler<DesktopReminderSchema, MutateReminderBinding> for MutateReminderHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &MutateReminderInput,
        reader: &mut DecisionReader<'_, '_, '_, DesktopReminderSchema, MutateReminderBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<DesktopReminderSchema, ReminderIntent>, ()>
    {
        let entity = match reader.resolve_entity(identity_field(), input.identity.clone()) {
            Ok(entity) => entity,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        for field in [
            revision_field(),
            due_field(),
            interval_field(),
            max_fires_field(),
            fired_count_field(),
        ] {
            if let Err(cause) = reader.field(&entity, field) {
                return HandlerResult::ExecutionDenied(cause);
            }
        }
        if let Err(cause) = reader.field(&entity, lifecycle_field()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&entity) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &MutateReminderInput,
        target: WorthQueryInvariantMutationTarget<DesktopReminderSchema, ReminderIntent>,
        writer: &mut CandidateWriter<'_, DesktopReminderSchema, MutateReminderBinding>,
    ) -> HandlerResult<u64, ()> {
        let entity = match writer.projected_entity(&target) {
            Ok(entity) => entity,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        if let Err(cause) = writer.write_field(&entity, revision_field(), input.revision) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        if let Err(cause) = writer.write_field(&entity, due_field(), input.due) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        if let Err(cause) = writer.write_field(&entity, interval_field(), input.interval) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        if let Err(cause) = writer.write_field(&entity, max_fires_field(), input.max_fires) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        if let Err(cause) = writer.write_field(&entity, fired_count_field(), input.fired_count) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        if let Err(cause) = writer.write_field(&entity, lifecycle_field(), input.lifecycle.clone())
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        HandlerResult::Completed(input.revision)
    }
}
