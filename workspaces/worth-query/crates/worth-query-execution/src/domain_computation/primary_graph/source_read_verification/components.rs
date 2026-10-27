//! Whole scoped component validity shares this extent's actual selected pin.
use super::*;
use crate::domain_computation::primary_graph::component_selection::ComponentSelection;
impl SourceReadVerification<'_> {
    pub(in crate::domain_computation::primary_graph) fn check_component(
        &mut self,
        body: &ComponentSelection,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        let poll = || {
            execution.consult().map_err(Cause::from)?;
            if let Some(control) = control {
                control()?;
            }
            Ok::<(), Cause>(())
        };
        if !body
            .layout()
            .matches_native_definition(&self.pin, &poll)
            .map_err(Failure::Execution)?
        {
            return Ok(false);
        }
        for aggregate in body.aggregates() {
            if !self.check_aggregate(aggregate, execution, control)? {
                return Ok(false);
            }
        }
        for source in body.facts() {
            if let Some(walks) = &source.walks {
                for walk in walks.values() {
                    if !self.check_walk(walk, execution, control)? {
                        return Ok(false);
                    }
                }
            }
        }
        let mut equal = true;
        body.visit_physical(|fact| {
            poll().map_err(Failure::Execution)?;
            if !equal {
                return Ok(());
            }
            equal = fact.comparison(self.runtime, execution, self.pin.snapshot())?;
            Ok(())
        })?;
        poll().map_err(Failure::Execution)?;
        Ok(equal)
    }
}
