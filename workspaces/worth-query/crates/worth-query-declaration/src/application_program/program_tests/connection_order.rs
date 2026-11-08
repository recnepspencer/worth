//! Scheduling indices retain the validator's exact connection-instance key.
use super::*;
use crate::application_program::ApplicationConnectionDeclaration;
fn foreign(source: &'static str, target: &'static str) -> ApplicationConnectionDeclaration {
    let actual = FlatConnection::declaration();
    ApplicationConnectionDeclaration::new(
        actual.identity(),
        source,
        actual.source_feature(),
        actual.source_port(),
        target,
        actual.target_feature(),
        actual.target_port(),
        actual.target_required(),
        actual.exports_across_instances(),
    )
}
#[test]
fn a_validated_connection_keeps_its_order_when_the_index_is_retained() {
    let program = ApplicationProgramAuthoring::<TestSchema, ConnectedProgram>::begin()
        .validated_program()
        .unwrap();
    let order = program.connection_order().clone();
    drop(program);
    assert_eq!(order.rank(&FlatConnection::declaration()), Some(1));
    assert!(order.lookup_work(&FlatConnection::declaration()).unwrap() > 0);
}
#[test]
fn an_unrostered_source_instance_cannot_alias_a_connection_with_the_same_identity() {
    let program = ApplicationProgramAuthoring::<TestSchema, ConnectedProgram>::begin()
        .validated_program()
        .unwrap();
    let actual = FlatConnection::declaration();
    let other = foreign("foreign-source", actual.target_instance());
    assert_eq!(program.connection_order().rank(&other), None);
}
#[test]
fn an_unrostered_target_instance_cannot_alias_a_connection_with_the_same_identity() {
    let program = ApplicationProgramAuthoring::<TestSchema, ConnectedProgram>::begin()
        .validated_program()
        .unwrap();
    let actual = FlatConnection::declaration();
    let other = foreign(actual.source_instance(), "foreign-target");
    assert_eq!(program.connection_order().rank(&other), None);
}
