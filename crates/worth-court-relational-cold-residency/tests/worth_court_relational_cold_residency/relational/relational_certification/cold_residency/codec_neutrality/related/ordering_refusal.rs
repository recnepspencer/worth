//! Ordering semantics are checked before codec restore changes its destination.
use super::{
    super::{
        fixture::{capture, locator},
        serial_request,
    },
    fixture::*,
};
use worth_relational::facade::identity::KindId;

#[test]
fn changed_ordering_key_meaning_refuses_before_destination_restore() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution);
    // Replace the actual registered definition with the changed payload key,
    // rather than manufacturing a transition-permission declaration.
    world
        .runtime
        .index_authority()
        .register(worth_relational::facade::indexes::DerivedIndexDefinition {
            index_id: worth_relational::facade::indexes::DerivedIndexId(99),
            name: "changed derived ordering".into(),
            branch_scoped: true,
            unique_entity_kind: None,
            kind: worth_relational::facade::indexes::DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: KindId(4),
                parent_endpoint:
                    worth_relational::facade::indexes::RelatedEntityEndpoint::SourceParent,
                child_kind: KindId(3),
                ordering: vec![worth_relational::facade::indexes::RelatedEntityOrderingField::new(
                    locator("derived"),
                    worth_relational::facade::indexes::RelatedEntityOrderingDirection::Ascending,
                )],
            },
        })
        .expect("index fixture has identity capacity");
    let (mut target, _) = installed(2);
    let before = capture(execution, &target).bytes().to_vec();
    let owner = target.runtime_instance_id();
    let denial = crate::relational::relational_certification::cold_residency::codec_neutrality::fixture::restore_codec_transition(&mut target, execution, &capture(execution, &world.runtime))
        .unwrap_err();
    assert!(
        matches!(&denial, worth_relational::facade::durability::RelationalRecoveryStop::Durability(error)
        if error.class == worth_relational::facade::durability::RecoveryFailureClass::SchemaMismatch && error.detail.contains("ordering")),
        "{denial:?}"
    );
    assert_eq!(target.runtime_instance_id(), owner);
    assert_eq!(capture(execution, &target).bytes(), before);
}
