//! Cold prerequisite closure joins existing installed executors only on demand.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest;
use std::collections::{BTreeMap, BTreeSet};
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(super) fn readmit_recorded_prerequisites(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        interest: Option<&WorthQueryOutputDemandInterest>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(interest) = interest else {
            return Ok(());
        };
        let Some(readmission) = interest.durable_readmission() else {
            return Ok(());
        };
        let Some(position) = readmission.durable_position() else {
            return Ok(());
        };
        if readmission
            .recorded_prerequisites
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
        {
            return Ok(());
        }
        let history = self
            .installed_program_support()
            .ok_or_else(unavailable)?
            .obligation_history();
        let mut visited = BTreeSet::new();
        let mut stack = vec![(position, false)];
        let mut bindings = BTreeMap::new();
        while let Some((next, expanded)) = stack.pop() {
            execution.consult().map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ExecutionRequest(crate::domain_computation::primary_graph::application_contribution::advancement_denial(cause)), "recorded prerequisites"))?;
            let row = history.output_demand(next).ok_or_else(unavailable)?;
            if !expanded {
                if !visited.insert(next) {
                    continue;
                }
                stack.push((next, true));
                stack.extend(row.prerequisites.iter().rev().map(|prior| (*prior, false)));
                continue;
            }
            if next != position {
                let installed = self
                    .installed_producers
                    .entries
                    .get(&row.producer)
                    .ok_or_else(unavailable)?;
                let admitted = installed
                    .executor
                    .readmit_recorded_demand(execution, self, principal, request, &row)?;
                bindings.insert(next, admitted);
            }
            let dependent = if next == position {
                interest
            } else {
                &bindings[&next]
            };
            let prerequisites = row
                .prerequisites
                .iter()
                .map(|prior| bindings.get(prior).ok_or_else(unavailable))
                .collect::<Result<Vec<_>, _>>()?;
            dependent.readmit_prerequisite_links(&prerequisites)?;
        }
        *readmission
            .recorded_prerequisites
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        Ok(())
    }
}
fn unavailable() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RequiredSuccessorStateUnavailable,
        "recorded prerequisite closure",
    )
}
