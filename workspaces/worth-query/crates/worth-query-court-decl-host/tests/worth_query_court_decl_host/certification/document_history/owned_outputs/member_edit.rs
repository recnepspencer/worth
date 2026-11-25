//! An owned-member edit that leaves the producer model source unchanged.
use super::*;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub key: String,
    pub value: u64,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum Denial {
    ValueOutOfRange,
}
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "owned-outputs.member-edit-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-outputs.member-edit-denial.v1" });
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.edit-member.v1" });
worth_query_operation!(pub PatchMember for Schema, input Input; access { read_fields: [MemberKey::at("MemberKey"), MemberValue::at("MemberValue")]; writes: [MemberValue::at("MemberValue")]; });
fn scope(input: &Intent) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.edit-member.v1", input Input, operation PatchMember, result ResultBinding,
    idempotency String, identity "owned-outputs.edit-member.v1", decision WorthQueryInvariantMutationTarget<Schema, Member>, denial DenialBinding,
    handler identity "owned-outputs.edit-member.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope MemberKey, field MemberKey::at("MemberKey"), value scope,
);
thread_local! {
    static STOP_DURING_DECIDE: std::cell::RefCell<Option<WorthQueryCancellationSource>> =
        const { std::cell::RefCell::new(None) };
}
/// The next member decision on this thread cancels this request and stops.
pub(super) fn stop_request_during_next_decide(cancellation: WorthQueryCancellationSource) {
    STOP_DURING_DECIDE.with(|stop| *stop.borrow_mut() = Some(cancellation));
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn declared_durable_capabilities(
    ) -> Option<&'static [worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability]> {
        Some(&[worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability::RelationalMutation])
    }

    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Member>, Denial> {
        if let Some(cancellation) = STOP_DURING_DECIDE.with(|stop| stop.borrow_mut().take()) {
            cancellation.cancel();
            return HandlerResult::Cancelled;
        }
        if input.value > 1000 {
            return HandlerResult::DomainDenied(Denial::ValueOutOfRange);
        }
        let result = (|| {
            let entity = reader.resolve_entity(MemberKey::at("MemberKey"), input.key.clone())?;
            reader.field(&entity, MemberValue::at("MemberValue"))?;
            reader.mutation_target(&entity)
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &Intent,
        target: WorthQueryInvariantMutationTarget<Schema, Member>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, Denial> {
        let result = writer.projected_entity(&target).and_then(|entity| {
            writer.write_field(&entity, MemberValue::at("MemberValue"), input.value)
        });
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<PatchMember>(
            PatchMember::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
