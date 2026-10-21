//! Pure contract installation, not an ordinary discovered execution proof.
use super::*;
use crate::application_program::discovered_contracts::*;
use worth_query_decl::facade::application_program::{
    ApplicationAction, ApplicationActionDeliveryDeclaration, ApplicationProgramAuthoring,
    ApplicationProgramValidationDenialKind,
};
use worth_query_topology_entry::PlanarSourceAdjustmentBinding;
type Binding = PlanarSourceAdjustmentBinding<ConsumerSchema>;

#[test]
fn discovered_installed_concrete_delivery_and_occurrence_are_exact() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, Program<QueryDelivery>>::begin()
        .validated_program()
        .unwrap();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<Program<QueryDelivery>>(
            "model-owner",
            "model-owner",
        ),
    ];
    let world = installation::install_selected_program(
        &crate::installed_schema(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &grants,
        program,
    )
    .unwrap();
    let installed = world.application.installed_program();
    let actual = installed
        .resolve_action::<Binding, Action<QueryDelivery>>()
        .unwrap();
    assert_eq!(
        actual.delivery().unwrap(),
        Action::<QueryDelivery>::selector().delivery()
    );
    assert!(actual.required_output_source());
    let counterfeit = ApplicationActionDeliveryDeclaration::discovered::<
        ConsumerSchema,
        Binding,
        CounterfeitDelivery,
    >();
    assert_eq!(
        actual.delivery().unwrap().canonical_meaning(),
        counterfeit.canonical_meaning()
    );
    assert_ne!(
        actual.delivery().unwrap(),
        &counterfeit,
        "runtime concrete selector identity stays outside canonical bytes"
    );
    assert_eq!(
        installed
            .resolve_action::<Binding, Action<CounterfeitDelivery>>()
            .unwrap_err(),
        worth_query_host::facade::domain::WorthQueryActionResolutionDenial::DeliveryMismatch
    );
    type WrongInstance = worth_query_decl::facade::application_program::ApplicationActionReference<
        ConsumerSchema,
        worth_query_decl::facade::application_program::ApplicationRootComposition,
        worth_query_topology_entry::PlanarSourceFeature,
        Binding,
        QueryDelivery,
    >;
    assert!(installed
        .resolve_action::<Binding, WrongInstance>()
        .is_err());
}

#[test]
fn discovered_closed_projection_shapes_and_empty_batch_authoring() {
    assert!(
        ApplicationProgramAuthoring::<ConsumerSchema, Program<BatchDelivery<2>>>::begin()
            .validated_program()
            .is_ok()
    );
    assert!(
        ApplicationProgramAuthoring::<ConsumerSchema, Program<NoProjectionDelivery>>::begin()
            .validated_program()
            .is_ok()
    );
    let denial =
        match ApplicationProgramAuthoring::<ConsumerSchema, Program<BatchDelivery<0>>>::begin()
            .validated_program()
        {
            Err(denial) => denial,
            Ok(_) => panic!("empty declared batch accepted"),
        };
    assert_eq!(
        denial.kind(),
        ApplicationProgramValidationDenialKind::InvalidActionDelivery
    );
    let scalar =
        ApplicationActionDeliveryDeclaration::discovered::<ConsumerSchema, Binding, QueryDelivery>(
        );
    let batch = ApplicationActionDeliveryDeclaration::discovered::<
        ConsumerSchema,
        Binding,
        BatchDelivery<2>,
    >();
    let triple = ApplicationActionDeliveryDeclaration::discovered::<
        ConsumerSchema,
        Binding,
        BatchDelivery<3>,
    >();
    let none = ApplicationActionDeliveryDeclaration::discovered::<
        ConsumerSchema,
        Binding,
        NoProjectionDelivery,
    >();
    assert_ne!(scalar.canonical_meaning(), batch.canonical_meaning());
    assert_ne!(batch.canonical_meaning(), triple.canonical_meaning());
    assert_ne!(none.canonical_meaning(), scalar.canonical_meaning());
    assert!(none.output().unwrap().read().is_none());
    assert!(scalar.fixed_output().is_none());
}

#[test]
fn discovered_authoring_refuses_wrong_occurrence_and_missing_actual_root() {
    let wrong = match ApplicationProgramAuthoring::<ConsumerSchema, WrongOccurrenceProgram>::begin()
        .validated_program()
    {
        Err(denial) => denial,
        Ok(_) => panic!("wrong source occurrence accepted"),
    };
    assert_eq!(
        wrong.kind(),
        ApplicationProgramValidationDenialKind::InvalidActionDelivery
    );
    let missing = match ApplicationProgramAuthoring::<ConsumerSchema, MissingRootProgram>::begin()
        .validated_program()
    {
        Err(denial) => denial,
        Ok(_) => panic!("undeclared discovered root accepted"),
    };
    assert_eq!(
        missing.kind(),
        ApplicationProgramValidationDenialKind::InvalidActionDelivery
    );
    let two = ApplicationProgramAuthoring::<ConsumerSchema, Program<BatchDelivery<2>>>::begin()
        .validated_program()
        .unwrap();
    let three = ApplicationProgramAuthoring::<ConsumerSchema, Program<BatchDelivery<3>>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(two.identity(), three.identity());
    assert_ne!(
        two.revision(),
        three.revision(),
        "batch cardinality participates in real program revision"
    );
}
