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
        let (program, target, _) = self.candidate_observing(reference, false, false);
        (program, target)
    }
    pub(super) fn column_candidate(
        &self,
    ) -> (
        world::Program<ChangeForm>,
        WorthQueryApplicationEffectEntity<Fixture, Item>,
    ) {
        let (program, _, target) = self.candidate_observing(ChangeForm::reference(), true, false);
        let target = program.projected_entity(&target).unwrap();
        (program, target)
    }
    pub(super) fn candidate_observing<O>(
        &self,
        reference: ApplicationOperationRef<Fixture, O, ()>,
        observe_note: bool,
        observe_extensions: bool,
    ) -> (
        world::Program<O>,
        ObservedTarget<Item>,
        WorthQueryInvariantMutationTarget<Fixture, Item>,
    )
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
                    .resolve_entity(
                        &phase,
                        select_field(Fixture, Item, ItemIdentity, "id"),
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
                let ((target, native_target), projection, _) = self
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, projected| {
                            let native_target = reader.mutation_target(projected).unwrap();
                            let observed = projected.for_fragment();
                            let mut view = reader.fragment(Fixture).unwrap();
                            work(|work| view.read(&observed, Form, work)).unwrap();
                            if observe_note {
                                work(|work| view.read(&observed, Note, work)).unwrap();
                            }
                            let target = view.target(&observed).unwrap();
                            if observe_extensions {
                                use worth_query_testkit_decl::fragment_extension_fixture::*;
                                work(|work| {
                                    reader
                                        .fragment(Annotations)
                                        .unwrap()
                                        .read(&observed, Annotation, work)
                                })
                                .unwrap();
                                work(|work| {
                                    reader.fragment(Classifications).unwrap().read(
                                        &observed,
                                        Classification,
                                        work,
                                    )
                                })
                                .unwrap();
                            }
                            (target, native_target)
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
                (candidate, target, native_target)
            })
            .unwrap()
    }
}
