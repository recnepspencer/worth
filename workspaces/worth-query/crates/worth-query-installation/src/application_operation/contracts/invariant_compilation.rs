//! Derives the shared state-load and execution budgets for installed invariants.

use std::num::NonZeroU32;

use super::compilation::APPLICATION_INVARIANT_SLOT;
use crate::application_operation::WorthQueryApplicationCandidateDemand;
use crate::domain_operation::{
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
    WorthQueryInvariantExecutionContract, WorthQueryOperationEffectContract,
    WorthQueryOperationEffectFamily, WorthQueryOperationInvariantContract,
};

pub(super) fn mutation_contracts(
    decision_fact_budget: usize,
    graph_mutation_count: usize,
    candidate_demand: WorthQueryApplicationCandidateDemand,
    invariant_invocations: &[crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor],
) -> Result<
    (
        WorthQueryOperationEffectContract,
        WorthQueryOperationInvariantContract,
        WorthQueryInvariantExecutionContract,
    ),
    (),
> {
    if graph_mutation_count == 0 {
        return Ok((
            WorthQueryOperationEffectContract::NotRequired,
            WorthQueryOperationInvariantContract::NotRequired,
            WorthQueryInvariantExecutionContract::NotRequired,
        ));
    }
    let invariant_execution = application_invariant_execution_contract(
        decision_fact_budget,
        graph_mutation_count,
        candidate_demand.candidate_items(),
        invariant_invocations,
    )?;
    let invariant_slots = invariant_execution
        .requirements()
        .iter()
        .map(|requirement| requirement.slot().to_owned())
        .collect();
    Ok((
        WorthQueryOperationEffectContract::Declared {
            effect_families: vec![WorthQueryOperationEffectFamily::Mutation],
        },
        WorthQueryOperationInvariantContract::Declared { invariant_slots },
        invariant_execution,
    ))
}

fn application_invariant_execution_contract(
    decision_fact_budget: usize,
    graph_mutation_count: usize,
    candidate_items: u64,
    invariant_invocations: &[crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor],
) -> Result<WorthQueryInvariantExecutionContract, ()> {
    let maximum_state_facts =
        maximum_state_facts(decision_fact_budget, graph_mutation_count, candidate_items)?;
    let native_execution_work = u64::try_from(maximum_state_facts).map_err(|_| ())?;
    let native = WorthQueryInstalledInvariantExecutionRequirement::new(
        APPLICATION_INVARIANT_SLOT,
        "application-installed-invariants",
        NonZeroU32::new(1).expect("one is nonzero"),
        WorthQueryInvariantEnforcement::Blocking,
        "primary",
        ["application-proposed-state"],
        maximum_state_facts,
        load_and_execution_work(maximum_state_facts, native_execution_work)?,
    )
    .map_err(|_| ())?;
    let custom = invariant_invocations.iter().map(|invariant| {
        let point = match invariant.execution_point() {
            worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint::CommitBoundary => "commit-boundary",
            worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint::MutationSensitive => "mutation-sensitive",
            worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint::SnapshotPublication => "snapshot-publication",
        };
        let version = (u32::from(invariant.major()) << 16) | u32::from(invariant.minor());
        WorthQueryInstalledInvariantExecutionRequirement::new(
            format!("relational-custom:{}@{point}", invariant.identifier()),
            "relational-custom-invariant",
            NonZeroU32::new(version).ok_or(())?,
            WorthQueryInvariantEnforcement::Blocking,
            "primary",
            ["application-proposed-state"],
            maximum_state_facts,
            load_and_execution_work(maximum_state_facts, invariant.maximum_work_units().get())?,
        ).map_err(|_| ())
            .map(|requirement| requirement.with_application_invariant(invariant.clone()))
    }).collect::<Result<Vec<_>, ()>>()?;
    WorthQueryInvariantExecutionContract::declared(std::iter::once(native).chain(custom))
        .map_err(|_| ())
}

fn maximum_state_facts(
    decision_fact_budget: usize,
    graph_mutation_count: usize,
    candidate_items: u64,
) -> Result<usize, ()> {
    let declared_state_facts = decision_fact_budget
        .checked_add(graph_mutation_count)
        .ok_or(())?;
    let candidate_state_facts = usize::try_from(candidate_items).map_err(|_| ())?;
    Ok(declared_state_facts.max(candidate_state_facts).max(1))
}

fn load_and_execution_work(state_facts: usize, execution_work: u64) -> Result<u64, ()> {
    u64::try_from(state_facts)
        .map_err(|_| ())?
        .checked_add(execution_work)
        .ok_or(())
}

#[cfg(test)]
mod tests {
    use super::application_invariant_execution_contract;
    use crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor;
    use std::num::NonZeroU64;
    use worth_query_declaration::facade::application_schema::{
        ApplicationInvariantCostPosture, ApplicationInvariantEnforcement,
        ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
        ApplicationInvariantScopeTarget,
    };

    #[test]
    fn invariant_budgets_cover_candidate_cardinality_and_both_work_phases() {
        let custom = custom_invariant(7);
        for (candidates, expected_state, expected_native, expected_custom) in
            [(1, 4, 8, 11), (32, 32, 64, 39), (128, 128, 256, 135)]
        {
            let contract =
                application_invariant_execution_contract(3, 1, candidates, &[custom.clone()])
                    .unwrap();
            let native = contract
                .requirements()
                .iter()
                .find(|r| r.application_invariant().is_none())
                .unwrap();
            let custom = contract
                .requirements()
                .iter()
                .find(|r| r.application_invariant().is_some())
                .unwrap();
            assert_eq!(native.max_state_facts(), expected_state);
            assert_eq!(native.max_work_units(), expected_native);
            assert_eq!(custom.max_state_facts(), expected_state);
            assert_eq!(custom.max_work_units(), expected_custom);
        }
    }

    #[test]
    fn unrepresentable_invariant_budgets_are_denied() {
        assert!(application_invariant_execution_contract(usize::MAX, 1, 0, &[]).is_err());
        assert!(application_invariant_execution_contract(1, 1, u64::MAX, &[]).is_err());
        assert!(
            application_invariant_execution_contract(1, 1, 0, &[custom_invariant(u64::MAX)])
                .is_err()
        );
    }

    fn custom_invariant(work: u64) -> WorthQueryInstalledApplicationInvariantDescriptor {
        WorthQueryInstalledApplicationInvariantDescriptor::from_installed_parts(
            "budget-regression".to_owned(),
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
            NonZeroU64::new(work).unwrap(),
            ApplicationInvariantEnforcement::BlockCommit,
            vec![ApplicationInvariantGroup::SchemaCompliance],
            vec![ApplicationInvariantScopeTarget::Entity("item".to_owned())],
            vec![ApplicationInvariantScopeTarget::Entity("item".to_owned())],
            "primary".to_owned(),
            ApplicationInvariantCostPosture::Touched,
        )
    }
}
