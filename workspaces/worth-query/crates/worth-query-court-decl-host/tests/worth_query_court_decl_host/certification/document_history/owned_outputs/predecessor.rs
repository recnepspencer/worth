//! Public production predecessor reads: the producer grant is not its READ grant.
use super::*;
use std::cell::RefCell;
worth_query_ability!(pub ReadInputs scoped_to Model, in Schema);
worth_query_policy!(pub InputReadPolicy in Schema);
worth_query_operation!(pub ReadOldInputs for Schema, input query::ParametersBinding; access {
    read_entities: [Model]; read_fields: [ModelValue::at("ModelValue")];
});
worth_query_operation!(pub ReadWrongScope for Schema, input query::ParametersBinding; access {
    read_entities: [Model]; read_fields: [ModelValue::at("ModelValue")];
});
worth_query_operation_requires!(ReadOldInputs => [ReadInputs]);
worth_query_operation_requires!(ReadWrongScope => [permission::ReadMember]);
worth_query_operation!(pub UndeclaredRead for Schema, input query::ParametersBinding; access {
    read_entities: [Model]; read_fields: [ModelValue::at("ModelValue")];
});
pub const CONTRACT: WorthQueryPredecessorReadContract<Schema> =
    WorthQueryPredecessorReadContract::new(&[
        WorthQueryPredecessorReadOperation::new(ReadOldInputs::reference()),
        WorthQueryPredecessorReadOperation::new(ReadWrongScope::reference()),
    ]);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(navigation::grant::Owner::reference())
        .forward(ModelMember::reference())
        .where_equal(
            MemberValue::at("MemberValue"),
            ApplicationEncodedScalarValue::try_new(5u64).unwrap(),
        )
        .unwrap()
        .reverse(ModelMember::reference())
        .allow(Model::reference());
    inspection::declare(synchronous::declare(schema))
        .declared_operation::<UndeclaredRead>(
            UndeclaredRead::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .ability(ReadInputs::reference())
        .policy(InputReadPolicy::reference())
        .ability_policy(
            ReadInputs::reference(),
            InputReadPolicy::reference(),
            [path],
        )
        .declared_operation::<ReadOldInputs>(
            ReadOldInputs::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .declared_operation::<ReadWrongScope>(
            ReadWrongScope::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_requires_ability(ReadOldInputs::reference(), ReadInputs::reference())
        .operation_requires_ability(
            ReadWrongScope::reference(),
            permission::ReadMember::reference(),
        )
}
type ReadResult = Result<Option<u64>, WorthQueryPredecessorReadDenial>;
#[derive(Default)]
struct Probe {
    wrong_scope: bool,
    undeclared: bool,
    foreign: bool,
    after_read: Option<Box<dyn FnOnce()>>,
    around_read: Option<Box<dyn FnOnce(&mut dyn FnMut() -> ReadResult) -> ReadResult>>,
    values: Vec<Result<Option<u64>, WorthQueryPredecessorReadDenial>>,
}
thread_local! { static PROBE: RefCell<Probe> = RefCell::new(Probe::default()); }
pub fn observe<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
    source: &WorthQueryInvariantEntityIdentity<Schema, Model>,
) {
    let foreign = PROBE.with(|probe| probe.borrow().foreign);
    // Resolving B observes only its identity/key at C. Its value is read only
    // through the historical operation, so later value edits must not invalidate A.
    let foreign_source = foreign.then(|| {
        reader
            .resolve_entity(ModelKey::at("ModelKey"), "b".to_owned())
            .unwrap()
    });
    let source = foreign_source.as_ref().unwrap_or(source);
    let wrong = PROBE.with(|probe| probe.borrow().wrong_scope);
    let undeclared = PROBE.with(|probe| probe.borrow().undeclared);
    let mut read = || {
        if undeclared {
            reader.predecessor_observation(UndeclaredRead::reference(), |old| {
                let entity = old.entity(Model::reference(), source).unwrap();
                let value = old.field(&entity, ModelValue::at("ModelValue")).unwrap();
                after_read();
                value
            })
        } else if wrong {
            reader.predecessor_observation(ReadWrongScope::reference(), |old| {
                let entity = old.entity(Model::reference(), source).unwrap();
                let value = old.field(&entity, ModelValue::at("ModelValue")).unwrap();
                after_read();
                value
            })
        } else {
            reader.predecessor_observation(ReadOldInputs::reference(), |old| {
                let entity = old.entity(Model::reference(), source).unwrap();
                let value = old.field(&entity, ModelValue::at("ModelValue")).unwrap();
                after_read();
                value
            })
        }
    };
    let around = PROBE.with(|probe| probe.borrow_mut().around_read.take());
    let result = match around {
        Some(around) => around(&mut read),
        None => read(),
    };
    // Deliberately swallow admission denials. The owner must still veto effects.
    PROBE.with(|probe| probe.borrow_mut().values.push(result));
}
mod control;
mod courts;
pub(super) mod inspection;
mod origin_kind;
pub(super) mod program;
pub(super) mod synchronous;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
fn install(worker: bool) -> Host {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    world::install_program::<program::Program>(limits)
}

fn after_read() {
    let action = PROBE.with(|probe| probe.borrow_mut().after_read.take());
    if let Some(action) = action {
        action();
    }
}
