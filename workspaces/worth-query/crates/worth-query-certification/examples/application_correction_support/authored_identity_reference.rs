//! An authored relation retains its original physical endpoint across restoration.
use super::{
    authentication::*,
    authored_identity_model::{AuthoredReference, Target},
    edit::{DenialBinding, Edited, MissingValue, ResultBinding},
    host,
    schema::*,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    application_entry::*,
    declaration::{application_operation::*, application_program::*, application_schema::*},
    primary_graph::*,
    runtime::ExecutionAllocationPolicy,
    *,
};
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Inspect {
    pub key: String,
    pub original: ((u32, u64), u32),
}
impl Inspect {
    fn scope_key(input: &Self) -> String {
        input.key.clone()
    }
}
worth_query_structured_value_binding!(pub Input for Inspect { identity: "authored-identity.inspect.input.v1" });
worth_query_operation!(pub Operation for NumericSchema, input Input);
worth_query_operation_reads!(Operation => [Number, Key, Primary, AuthoredReference]);
worth_query_operation_writes!(Operation => [Primary]);
pub struct Decision {
    target: Target,
    value: u64,
}
worth_query_mutation_binding!(
    pub Binding for Inspect, schema NumericSchema,
    identity "authored-identity.inspect.v1", input Input, operation Operation, result ResultBinding,
    idempotency String, identity "authored-identity.command.v1",
    decision Decision, denial DenialBinding,
    handler identity "authored-identity.inspect.v1", outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value Inspect::scope_key,
);
static STALE: AtomicUsize = AtomicUsize::new(0);
pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &Inspect,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<Decision, MissingValue> {
        let parent = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(entity) => entity,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let references = match reader
            .reader()
            .decision_relations_from(AuthoredReference::reference(), &parent)
        {
            Ok(references) => references,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        if references.is_empty() {
            STALE.fetch_add(1, Ordering::SeqCst);
            return HandlerResult::DomainDenied(MissingValue);
        }
        let [original] = references.as_slice() else {
            panic!("the original authored reference is unique");
        };
        assert_eq!(
            identity_coordinates(original.to().entity_id()),
            input.original,
            "the live reference must name the full original receipt identity"
        );
        let value = match reader.field(original.to(), Primary::reference()) {
            Ok(Some(value)) => value,
            Ok(None) => return HandlerResult::DomainDenied(MissingValue),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match reader.field(&parent, Primary::reference()) {
            Ok(Some(_)) => (),
            Ok(None) => return HandlerResult::DomainDenied(MissingValue),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        match reader.mutation_target(&parent) {
            Ok(target) => HandlerResult::Completed(Decision { target, value }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Inspect,
        decision: Decision,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let parent = match writer.projected_entity(&decision.target) {
            Ok(entity) => entity,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        match writer.write_field(&parent, Primary::reference(), decision.value) {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .operation(
            Operation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_entity(Operation::reference(), Number::reference())
        .operation_read_field(Operation::reference(), Key::reference())
        .operation_read_field(Operation::reference(), Primary::reference())
        .operation_read_relation(Operation::reference(), AuthoredReference::reference())
        .operation_write(Operation::reference(), Primary::reference())
        .application_mutation_binding::<Binding>()
}
pub struct Feature;
impl ApplicationFeature<NumericSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "authored-identity.reference.feature.v1";
}
pub struct Completion;
impl ApplicationActionDelivery<NumericSchema, Binding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "authored-identity.inspect.completion.v1";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .action::<Binding, Completion>()
        .finish_action()
        .finish()
}
fn identity_coordinates(identity: EntityId) -> ((u32, u64), u32) {
    (
        (identity.partition_value(), identity.local_slot_value()),
        identity.generation_value(),
    )
}
pub fn inspect(application: &host::Host, original: EntityId, expect_stale: bool) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let key = format!("inspect-original-stale-{expect_stale}");
    let stale = STALE.load(Ordering::SeqCst);
    let head = super::required_output_courts::head(application);
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Inspect {
            key: "numeric".to_owned(),
            original: identity_coordinates(original),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation);
    if expect_stale {
        assert_eq!(
            STALE.load(Ordering::SeqCst),
            stale + 1,
            "retired original relation must be absent from native live adjacency: {outcome:?}"
        );
        assert!(
            matches!(
                outcome,
                Ok(WorthQueryApplicationMutationOutcome::DomainDenied(
                    MissingValue
                ))
            ),
            "the original reference consumer must refuse its missing live relationship"
        );
        assert_eq!(super::required_output_courts::head(application), head);
    } else {
        assert!(
            matches!(
                outcome,
                Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
            ),
            "live original reference must be usable: {outcome:?}"
        );
        assert_eq!(STALE.load(Ordering::SeqCst), stale);
        assert_eq!(host::read(application)[0].primary, 41);
    }
}
