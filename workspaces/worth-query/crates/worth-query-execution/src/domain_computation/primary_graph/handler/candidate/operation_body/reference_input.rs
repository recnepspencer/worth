//! Existing reference replacement emits only the admitted changed endpoints.
use super::*;
use crate::domain_computation::primary_graph::operation_body::reference_input::PreparedBodyReferences;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> CandidateWriter<'_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_references<Entity>(
        &mut self,
        scope: &WorthQueryApplicationEffectEntity<S, Entity>,
        body: &ApplicationOperationBodyProgram,
        prepared: PreparedBodyReferences<S>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let _memory = prepared._memory;
        for (reference, targets) in body.reference_inputs().iter().zip(prepared.targets) {
            if targets.current.as_ref().map(|target| target.entity_id)
                == targets.selected.as_ref().map(|target| target.entity_id)
            {
                continue;
            }
            let [link, unlink] = reference.program_targets();
            if let Some(current) = targets.current {
                let current = self.projected_entity(&current)?;
                self.candidate.unlink_at_target(&unlink, scope, &current)?;
            }
            if let Some(selected) = targets.selected {
                let selected = self.projected_entity(&selected)?;
                self.candidate
                    .link_at_target(&link, &reference.key, scope, &selected)?;
            }
        }
        Ok(())
    }
}
