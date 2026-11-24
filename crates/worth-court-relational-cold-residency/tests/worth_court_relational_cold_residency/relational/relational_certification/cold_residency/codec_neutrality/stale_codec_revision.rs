//! A genuine checkpoint refuses changed codec meaning with a retained old revision.
use super::fixture::*;
use worth_foundational::facade::{AspectContract, AspectShape};
use worth_relational::facade::durability::{RecoveryFailureClass, RelationalRecoveryStop};
use worth_relational::facade::identity::KindId;

#[test]
fn actual_checkpoint_refuses_stale_codec_revision_without_replacing_target_state() {
    let request = super::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let checkpoint = capture(execution, &world.runtime);
    let original_bytes = checkpoint.bytes().to_vec();
    let source_before = protected_payload(execution, &world.runtime, world.authored);
    let source_commit = world.runtime.history().historical_latest_commit();
    let old_revision = world
        .runtime
        .config()
        .schema
        .registry
        .entity_registration(KindId(3))
        .unwrap()
        .aspect_contract_declarations
        .aspects[0]
        .contract
        .revision();
    let target = registry(2);
    let mut target_schema = worth_relational::facade::schema::RelationalSchemaRegistry::new();
    for id in [KindId(1), KindId(2)] {
        target_schema = target_schema
            .register_entity_kind(target.entity_registration(id).unwrap().clone())
            .unwrap();
    }
    let mut derived = target.entity_registration(KindId(3)).unwrap().clone();
    let binding = &mut derived.aspect_contract_declarations.aspects[0];
    let current = &binding.contract;
    let AspectShape::Struct(shape) = current.shape() else {
        panic!("fixture declares scalar struct fields")
    };
    // Native configuration is the lawful owner of this adversarial declaration.
    // The genuine source checkpoint, publication and Original image stay intact.
    binding.contract = AspectContract::struct_aspect(
        current.key().clone(),
        current.identity(),
        old_revision,
        shape.clone(),
    );
    let (mut target, _) = installed_with(
        target_schema.register_entity_kind(derived).unwrap(),
        Vec::new(),
    );
    let initial_id = target.runtime_instance_id();
    let initial_schema = target.config().schema.registry;
    let initial_history = target.history().historical_latest_commit();
    let refusal = target
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap_err();
    let RelationalRecoveryStop::Durability(refusal) = refusal else {
        panic!("expected typed schema recovery refusal: {refusal:?}")
    };
    assert_eq!(refusal.class, RecoveryFailureClass::SchemaMismatch);
    // SchemaMismatch also names the later transition-required stop; this exact
    // existing owner detail proves that the stale fingerprint stops first.
    assert_eq!(
        refusal.detail,
        "recovered field codec meaning changed without a changed semantic aspect revision"
    );
    assert_eq!(target.runtime_instance_id(), initial_id);
    assert_eq!(target.config().schema.registry, initial_schema);
    assert_eq!(target.history().historical_latest_commit(), initial_history);
    assert_eq!(checkpoint.bytes(), original_bytes);
    assert_eq!(
        protected_payload(execution, &world.runtime, world.authored),
        source_before
    );
    assert_eq!(
        world.runtime.history().historical_latest_commit(),
        source_commit
    );
}
