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
        let (current, work) = fact
            .source_currentness_in(runtime, snapshot, *remaining_work)
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
        *remaining_work -= work;
        if !current {
            return Ok(OutputDependencySelection::FreshRequired);
        }
    }
    Ok(OutputDependencySelection::Reuse)
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
