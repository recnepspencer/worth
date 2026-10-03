use crate::domain_computation::primary_graph::{
    application_attempt::{WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure},
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// Internal choice; callers can only request an output through demand
/// admission. FreshRequired never grants authority to preserve an output.
pub(in crate::domain_computation::primary_graph) enum OutputDependencySelection {
    Reuse,
    FreshRequired,
}

pub(in crate::domain_computation::primary_graph) fn compare_retained_output_dependencies(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    identity_current: bool,
    facts: Option<&[WorthQueryApplicationObservedFact]>,
    remaining_work: &mut usize,
) -> Result<OutputDependencySelection, WorthQueryOutputDemandDenial> {
    let facts = match reusable_facts(identity_current, facts) {
        Ok(facts) => facts,
        Err(fresh) => return Ok(fresh),
    };
    for fact in facts {
        let available = *remaining_work;
        let prepaid = fact
            .exact_probe_work()
            .map_err(|_| work_denial())?
            .unwrap_or(0);
        if prepaid > *remaining_work {
            return Err(work_denial());
        }
        *remaining_work -= prepaid;
        let (current, work) = fact
            .source_currentness_in(runtime, snapshot, available)
            .map_err(|failure| match failure {
                WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        "output dependency comparison exceeded the admitted work budget",
                    )
                }
                WorthQuerySourceCurrentnessFailure::Unavailable => {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                        fact.locator_identity(),
                    )
                }
            })?;
        *remaining_work -= work.saturating_sub(prepaid);
        if !current {
            return Ok(OutputDependencySelection::FreshRequired);
        }
    }
    Ok(OutputDependencySelection::Reuse)
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "output dependency comparison exceeded the admitted work budget",
    )
}

fn reusable_facts(
    identity_current: bool,
    facts: Option<&[WorthQueryApplicationObservedFact]>,
) -> Result<&[WorthQueryApplicationObservedFact], OutputDependencySelection> {
    if !identity_current {
        return Err(OutputDependencySelection::FreshRequired);
    }
    facts
        .filter(|facts| !facts.is_empty())
        .ok_or(OutputDependencySelection::FreshRequired)
}

#[cfg(test)]
mod tests {
    use super::{reusable_facts, OutputDependencySelection};

    #[test]
    fn missing_dependency_facts_never_authorize_retained_output() {
        assert!(matches!(
            reusable_facts(true, None),
            Err(OutputDependencySelection::FreshRequired)
        ));
        assert!(matches!(
            reusable_facts(true, Some(&[])),
            Err(OutputDependencySelection::FreshRequired)
        ));
        assert!(matches!(
            reusable_facts(false, None),
            Err(OutputDependencySelection::FreshRequired)
        ));
    }
}
