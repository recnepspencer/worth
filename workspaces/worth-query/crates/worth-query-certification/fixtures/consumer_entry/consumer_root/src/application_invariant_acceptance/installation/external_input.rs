use super::ConsumerWorld;
use worth_query_topology_entry::EditPlanar;

pub(super) fn assert_external_inputs(world: &ConsumerWorld) {
    let external = world
        .application
        .installed_program()
        .external_input_provider::<
            EditPlanar,
            crate::application_program::external_input::NeutralExternalProvider,
        >()
        .expect("the neutral external provider slot is installed on the action");
    let provider = crate::application_program::external_input::NeutralExternalProvider::new(7);
    let changed = external.resolve(&provider, "material").unwrap();
    assert_eq!(changed.resolution().provenance(), &"neutral-catalog");
    provider.change(8);
    assert_eq!(
        changed.admit(&provider).err(),
        Some(crate::application_program::external_input::NeutralExternalDenial::Changed)
    );
    let removed = external.resolve(&provider, "material").unwrap();
    provider.remove();
    assert_eq!(
        removed.admit(&provider).err(),
        Some(crate::application_program::external_input::NeutralExternalDenial::Removed)
    );
    provider.change(9);
    let invalid = external.resolve(&provider, "material").unwrap();
    provider.invalidate();
    assert_eq!(
        invalid.admit(&provider).err(),
        Some(crate::application_program::external_input::NeutralExternalDenial::Invalid)
    );
    provider.change(10);
    let admitted = external
        .resolve(&provider, "material")
        .unwrap()
        .admit(&provider)
        .unwrap()
        .into_resolution();
    assert_eq!(admitted.values(), &10);
    assert_eq!(admitted.revision(), &10);

    // This ordinary action has no requirement or output correspondence.
    use worth_query_topology_entry::ReplacePlanarVertex;
    let action = world
        .application
        .installed_program()
        .actions()
        .iter()
        .find(|action| action.operation_type() == std::any::TypeId::of::<ReplacePlanarVertex>())
        .expect("the standalone vertex action is declared");
    assert!(action.evaluated_requirement().is_none());
    assert!(action.correspondence().is_none());
    let standalone = world.application.installed_program()
        .external_input_provider::<ReplacePlanarVertex, crate::application_program::external_input::NeutralExternalProvider>()
        .expect("ordinary external input is installed without a fabricated requirement");
    let captured = standalone.resolve(&provider, "vertices").unwrap();
    provider.change(11);
    assert_eq!(
        captured.admit(&provider).err(),
        Some(crate::application_program::external_input::NeutralExternalDenial::Changed)
    );
    let admitted = standalone
        .resolve(&provider, "vertices")
        .unwrap()
        .admit(&provider)
        .unwrap()
        .into_resolution();
    assert_eq!(admitted.values(), &11);
}
