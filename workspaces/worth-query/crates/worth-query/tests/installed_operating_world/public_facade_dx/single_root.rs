use crate::suite::installed_operation_fixture::{
    operating_world_family_workspace, BooleanFamily, BooleanOperation, ConstructFamily,
    ConstructOperation, GeometryDomain, RouteFamily, RouteOperation, TransformFamily,
    TransformOperation,
};

#[test]
fn one_root_binds_real_construct_boolean_transform_and_cross_domain_operations() {
    let mut workspace =
        operating_world_family_workspace("installed-public-facade-family-root").unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let root = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let construct = root
        .family(ConstructFamily)
        .bind(&domain, ConstructOperation)
        .unwrap();
    let boolean = root
        .family(BooleanFamily)
        .bind(&domain, BooleanOperation)
        .unwrap();
    let transform = root
        .family(TransformFamily)
        .bind(&domain, TransformOperation)
        .unwrap();
    let route = root
        .family(RouteFamily)
        .bind(&domain, RouteOperation)
        .unwrap();

    let identities = [
        construct.binding_identity(),
        boolean.binding_identity(),
        transform.binding_identity(),
        route.binding_identity(),
    ];
    for (index, identity) in identities.iter().enumerate() {
        assert!(
            identities[..index].iter().all(|prior| prior != identity),
            "each real installed operation must retain a distinct binding"
        );
    }
    assert_eq!(
        route.required_domain_roles().collect::<Vec<_>>(),
        ["auxiliary"]
    );

    let construct = construct
        .admit_execution_resources(
            Default::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap();
    let construct_admission = construct.resources().counters();
    let construct_counters = construct.execute(&mut workspace).unwrap().counters();
    let boolean = boolean
        .admit_execution_resources(
            Default::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap();
    let boolean_admission = boolean.resources().counters();
    let boolean_counters = boolean.execute(&mut workspace).unwrap().counters();
    let transform = transform
        .admit_execution_resources(
            Default::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap();
    let transform_admission = transform.resources().counters();
    let transform_counters = transform.execute(&mut workspace).unwrap().counters();
    let route = route
        .admit_execution_resources(
            Default::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap();
    let route_admission = route.resources().counters();
    let route_counters = route.execute(&mut workspace).unwrap().counters();
    for counters in [
        construct_admission,
        boolean_admission,
        transform_admission,
        route_admission,
    ] {
        assert_eq!(counters.runtime_authority_checks, 1);
        assert_eq!(counters.provider_session_mints, 1);
    }
    for counters in [
        construct_counters,
        boolean_counters,
        transform_counters,
        route_counters,
    ] {
        assert_eq!(counters.executor_contacts, 1);
    }
}
