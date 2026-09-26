use super::*;

#[test]
fn guarded_binding_cannot_install_without_a_program_owner() {
    let result =
        worth_query_host::facade::application_installation::in_memory::<BoundedDimensionSchema>(
            BoundedDimensionSchema::declaration().expect("the fixture schema is valid"),
            ((),),
            host_limits(),
            |_, _| Ok(()),
        );
    assert!(matches!(
        result,
        Err(WorthQueryInMemoryApplicationDenial::WorkflowAuthorityRequiresProgram)
    ));
}
