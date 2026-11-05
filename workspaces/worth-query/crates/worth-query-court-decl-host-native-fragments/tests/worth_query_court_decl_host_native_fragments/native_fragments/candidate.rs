use super::authentication::{authenticate_external, live_scope};
use super::*;
use std::time::Duration;
impl World {
    pub(super) fn candidate<O>(
        &self,
        reference: ApplicationOperationRef<Fixture, O, ()>,
    ) -> (world::Program<O>, ObservedTarget<Item>)
    where
        O: ApplicationOperationMarkerIdentity<Fixture, InputBinding = UnitInput> + 'static,
    {
        let request = live_scope();
        let external = authenticate_external(
            self.application.installed_schema(),
            "person",
            Duration::from_secs(60),
            &request,
        );
        self.application
            .with_host_advancement(|phase| {
                let selected = self.selected(&phase);
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        &self.binding,
                        &external,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let target = selected
                    .resolve_record_entity(
                        &phase,
                        Fixture,
                        record_field(Fixture, Item, ItemIdentity, "id").unwrap(),
                        11u64,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let operation = self
                    .application
                    .installed_schema()
                    .installed_operation(reference)
                    .unwrap();
                let admission = selected
                    .authorize_operation(
                        &phase,
                        &principal,
                        &target,
                        &operation,
                        TypedMutationPreconditions::new(),
                        &request,
                    )
                    .unwrap();
                let (target, projection, _) = self
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, projected| {
                            let observed = projected.for_fragment();
                            let mut view = reader.fragment(Fixture).unwrap();
                            work(|work| view.read(&observed, Form, work)).unwrap();
                            view.target(&observed).unwrap()
                        },
                        SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                let candidate = self
                    .application
                    .begin_projected_application_read_attempt(
                        &phase,
                        admission,
                        projection,
                        SystemAllocation,
                    )
                    .unwrap()
                    .complete_projected_dependencies(&phase, SystemAllocation)
                    .unwrap()
                    .begin_effect_program();
                (candidate, target)
            })
            .unwrap()
    }
}
