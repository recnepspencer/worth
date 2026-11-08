//! The framework updates the actual existing activation record, never a replacement.
use super::*;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedHomeProgramTransitionTarget<Schema, Program>
{
    pub fn bind_home_program_transition_activation<
        Before: ApplicationSchemaComposition,
        BeforeProgram,
    >(
        &self,
        predecessor: &ReadmittedExistingHomeNativeMeaning<Before, BeforeProgram>,
        source: &author_source::WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    ) -> Result<WorkerIntentBatch, WorthQueryPrimaryGraphInstallationDenial> {
        use super::super::super::super::{
            bootstrap::read_activation, program_occurrence::program_revision_rendering,
        };
        let (revision, cell) = predecessor.predecessor_activation();
        let (identity, _) = read_activation(
            &source.request(),
            &source.view,
            source.layout.program_activation(),
            usize::MAX,
            "the exact transition source does not activate A",
            |value| *value == program_revision_rendering(revision),
        )?;
        cell.publish(identity)
            .map_err(|_| author_source::denied("A activation is already bound"))?;
        self._pending
            ._state
            .activation
            .publish(identity)
            .map_err(|_| author_source::denied("B activation is already bound"))?;
        let locator = &self
            ._pending
            ._layout
            .program_activation()
            .program_revision_locator;
        let locator = worth_relational::facade::transactions::planned_aspect_field_locator(
            locator.aspect().aspect_key().clone(),
            locator.field_path().clone(),
        );
        Ok(
            WorkerIntentBatch::new("home-program-activation").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: identity,
                    fields: AspectFieldPatch::from_locator(
                        locator,
                        program_revision_rendering(&self._pending._state.support.initial_revision),
                    ),
                }),
            )),
        )
    }
}
