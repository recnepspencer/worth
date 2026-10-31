//! Exact completion retains the original candidate cut and every row disposition.
use super::*;
use worth_execution::ExecutionRequestArrayBuilder;

pub(super) struct RetainedRefinement {
    pub(super) candidates: ExecutionArray<Slot>,
    pub(super) dispositions: ExecutionArray<bool>,
}
impl CandidateSelection<'_, '_> {
    pub(in crate::domain_computation::primary_graph) fn refine(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
        mut evaluate: impl FnMut(Slot) -> Result<bool, Denial>,
    ) -> Result<Self, Denial> {
        if self.frame.posture != worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined {
            return Err(control.refuse(Stop::ForeignCoverage).into());
        }
        let coverage = self
            .coverage
            .as_mut()
            .and_then(Arc::get_mut)
            .ok_or_else(|| control.refuse(Stop::IncompleteCoverage))?;
        if coverage.refinement.is_some() {
            return Err(control.refuse(Stop::ForeignCoverage).into());
        }
        let mut dispositions = ExecutionRequestArrayBuilder::allocate(*execution, self.slots.len())
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        let mut accepted_count = 0;
        for slot in self.slots.elements().iter().copied() {
            control.checkpoint(1)?;
            let matches = evaluate(slot)?;
            dispositions
                .push(matches)
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
            accepted_count += usize::from(matches);
        }
        let dispositions = dispositions
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        let mut accepted = ExecutionRequestArrayBuilder::allocate(*execution, accepted_count)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for (slot, matches) in self
            .slots
            .elements()
            .iter()
            .copied()
            .zip(dispositions.elements())
        {
            control.checkpoint(1)?;
            if *matches {
                accepted
                    .push(slot)
                    .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
            }
        }
        let accepted = accepted
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        control.poll()?;
        coverage.refinement = Some(RetainedRefinement {
            candidates: std::mem::replace(&mut self.slots, accepted),
            dispositions,
        });
        Ok(self)
    }
}
