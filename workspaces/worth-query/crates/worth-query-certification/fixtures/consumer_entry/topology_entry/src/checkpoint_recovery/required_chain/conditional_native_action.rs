//! Joint native binding membership and exact installed conditional ownership.
use super::*;
use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding, application_program::*,
};
use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
mod installation;
mod program;
use program::{JointProgram, JointSchema};

#[test]
fn required_conditional_native_binding_assembles_with_its_real_installed_owner() {
    assert!(<PlanarMutationBinding<JointSchema> as ApplicationMutationBinding<JointSchema>>::REQUIRES_APPLICATION_PROGRAM);
    assert!(
        !<PlanarMutationBinding<CheckpointSchema> as ApplicationMutationBinding<
            CheckpointSchema,
        >>::REQUIRES_APPLICATION_PROGRAM
    );
    let application =
        installation::install::<0>().expect("both independent guards admit the real binding");
    let binding = application
        .installed_schema()
        .installed_mutation_binding::<PlanarMutationBinding<JointSchema>>()
        .expect("the required planar producer operation is installed");
    assert_eq!(
        binding.identity(),
        <PlanarMutationBinding<JointSchema> as ApplicationMutationBinding<JointSchema>>::IDENTITY
    );
    assert!(application
        .installed_schema()
        .installed_mutation_binding_inventory()
        .any(|entry| entry.binding_type()
            == std::any::TypeId::of::<PlanarMutationBinding<JointSchema>>()
            && entry.requires_application_program()));
    assert_eq!(std::any::TypeId::of::<<InitialPlanarProducer<JointSchema> as worth_query_host::facade::application_contribution::WorthQueryApplicationProducerBinding<JointSchema>>::Operation>(), std::any::TypeId::of::<PlanarMutationBinding<JointSchema>>());
}

#[test]
fn operation_only_conditional_does_not_answer_required_binding_membership() {
    let denied = installation::install::<1>()
        .err()
        .expect("operation identity is not binding membership");
    let WorthQueryInMemoryApplicationDenial::Program(denied) = denied else {
        panic!("required concrete membership must be the rejecting owner: {denied:?}")
    };
    assert!(
        denied.support_denial().is_none(),
        "operation support already passed"
    );
    assert_eq!(denied.subject(), format!("undeclared program binding: {}", <PlanarMutationBinding<JointSchema> as ApplicationMutationBinding<JointSchema>>::IDENTITY));
}

#[test]
fn nonconditional_native_and_ordinary_delivery_overlap_refuse() {
    let attempts = [
        (
            "nonconditional native overlap",
            installation::install::<2>().err(),
        ),
        (
            "ordinary delivery overlap",
            installation::install::<3>().err(),
        ),
    ];
    assert!(
        attempts.iter().all(|(_, denied)| matches!(denied,
            Some(WorthQueryInMemoryApplicationDenial::ConditionalProgramMismatch))),
        "each overlap must reach its exact conditional guard; actual returned denials: {attempts:?}"
    );
}

#[test]
fn exact_conditional_operation_set_and_wrong_binding_remain_strict() {
    let attempts = [
        (
            "missing conditional operation",
            installation::install::<4>().err(),
        ),
        (
            "extra conditional operation",
            installation::install::<5>().err(),
        ),
        (
            "wrong-operation installed binding",
            installation::install::<6>().err(),
        ),
    ];
    assert!(
        attempts.iter().all(|(_, denied)| matches!(denied,
            Some(WorthQueryInMemoryApplicationDenial::ConditionalProgramMismatch))),
        "each exact-set variant must reach its conditional guard; actual returned denials: {attempts:?}"
    );
}

#[test]
fn uninstalled_conditional_native_binding_refuses_at_support_owner() {
    let denied = installation::install::<7>()
        .err()
        .expect("typed declaration cannot install its own binding");
    let WorthQueryInMemoryApplicationDenial::Program(denied) = denied else {
        panic!("uninstalled concrete binding must fail support: {denied:?}")
    };
    assert!(
        matches!(denied.support_denial(), Some(worth_query_host::facade::domain::WorthQueryProgramSupportDenial::UnsupportedAction { binding, .. })
        if binding == <custody_source::ValueSourceBinding<JointSchema> as ApplicationMutationBinding<JointSchema>>::IDENTITY)
    );
}

#[test]
fn conditional_native_builder_and_dsl_preserve_complete_binding_meaning() {
    let builder = ApplicationFeatureSpec::root::<JointSchema, PlanarSourceFeature>()
        .conditional_native_action::<PlanarMutationBinding<JointSchema>>()
        .finish_action()
        .finish();
    let dsl = worth_query_decl::facade::worth_query_feature_spec!(root(JointSchema, PlanarSourceFeature);
        conditional_native_action(PlanarMutationBinding<JointSchema>) {};
    );
    assert_eq!(builder, dsl);
    struct Nested;
    impl ApplicationCompositionInstance for Nested {
        const PATH: &'static str = "conditional-native/nested";
    }
    let nested_builder = ApplicationFeatureSpec::at::<JointSchema, Nested, PlanarSourceFeature>()
        .conditional_native_action::<PlanarMutationBinding<JointSchema>>()
        .finish_action()
        .finish();
    let nested_dsl = worth_query_decl::facade::worth_query_feature_spec!(at(JointSchema, Nested, PlanarSourceFeature);
        conditional_native_action(PlanarMutationBinding<JointSchema>) {};
    );
    assert_eq!(nested_builder, nested_dsl);
    assert_eq!(nested_dsl.actions()[0].composition_instance(), Nested::PATH);
    let native = ApplicationFeatureSpec::root::<JointSchema, PlanarSourceFeature>()
        .native_action::<PlanarMutationBinding<JointSchema>>()
        .finish_action()
        .finish();
    let action = &dsl.actions()[0];
    let original = &native.actions()[0];
    assert!(action.conditional_only());
    assert!(!original.conditional_only());
    assert_eq!(action.action_type(), original.action_type());
    assert_eq!(
        action.required_output_source(),
        original.required_output_source()
    );
    assert!(action.delivery().is_none());
    assert_eq!(
        action.mutation_binding_type(),
        original.mutation_binding_type()
    );
    assert_eq!(action.operation_type(), original.operation_type());
    assert_eq!(
        action.operation_input_type(),
        original.operation_input_type()
    );
    assert_eq!(
        action.operation_input_identity(),
        original.operation_input_identity()
    );
    assert_eq!(
        action.workflow_execution_requirements(),
        original.workflow_execution_requirements()
    );
    assert_eq!(action.binding(), original.binding());
    assert_eq!(
        action.composition_instance(),
        original.composition_instance()
    );
    assert_eq!(action.feature(), original.feature());
    let program = ApplicationProgramAuthoring::<JointSchema, JointProgram<0>>::begin()
        .validated_program()
        .unwrap();
    assert!(program
        .actions()
        .iter()
        .any(|installed| installed == action));
}
