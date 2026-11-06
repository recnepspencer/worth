//! Courts enter the authenticated, contract-limited installed operation projection.
use super::authentication::{authenticate_external, live_scope};
use super::*;
use std::time::Duration;
impl World {
    pub(super) fn admitted_projection<O, Output>(
        &self,
        reference: ApplicationOperationRef<Fixture, O, ()>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Fixture, O>,
            &ObservedEntity<Item>,
        ) -> Output,
    ) -> Result<Output, HandlerExecutionDenial>
    where
        O: ApplicationOperationMarkerIdentity<Fixture, InputBinding = UnitInput> + 'static,
    {
        self.admitted_projection_when(reference, projection, |_| true)
    }
    pub(super) fn admitted_read<O, Output>(
        &self,
        reference: ApplicationOperationRef<Fixture, O, ()>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Fixture, O>,
            &ObservedEntity<Item>,
        ) -> Result<Output, FragmentDenial>,
    ) -> Result<Result<Output, FragmentDenial>, HandlerExecutionDenial>
    where
        O: ApplicationOperationMarkerIdentity<Fixture, InputBinding = UnitInput> + 'static,
    {
        self.admitted_projection_when(reference, projection, Result::is_ok)
    }
    fn admitted_projection_when<O, Output>(
        &self,
        reference: ApplicationOperationRef<Fixture, O, ()>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Fixture, O>,
            &ObservedEntity<Item>,
        ) -> Output,
        should_complete: impl FnOnce(&Output) -> bool,
    ) -> Result<Output, HandlerExecutionDenial>
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
                let completed = self
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, projected| projection(reader, &projected.for_fragment()),
                        SystemAllocation,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
                let (output, projection, _) = completed.into_parts();
                // A refused SDK read has no successful attempt to complete. The
                // relation courts still demand completion to catch swallowed failures.
                if !should_complete(&output) {
                    return Ok(output);
                }
                self.application
                    .begin_projected_application_read_attempt(
                        &phase,
                        admission,
                        projection,
                        SystemAllocation,
                    )
                    .map_err(HandlerExecutionDenial::new)?
                    .complete_projected_dependencies(&phase, SystemAllocation)
                    .map_err(HandlerExecutionDenial::new)?;
                Ok(output)
            })
            .unwrap()
    }
}
