use std::sync::Arc;

use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::history::CompositeRuntimeWorldCommit;
use crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract;
use crate::publication::CompositeOwnerExecutionResults;

use super::super::RuntimeWorldHistoryCatalogContract;

fn history_budgets(maximum_commits: u64, maximum_metadata_bytes: u64) -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 1,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: maximum_commits,
            history_metadata_bytes: maximum_metadata_bytes,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 1,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 1,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 1,
            retained_partial_metadata_bytes: 1,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 8,
            in_flight_pin_acquisition_reservations: 8,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 1,
        },
    })
    .expect("test history budgets are valid")
}
pub(super) fn history_contract(
    maximum_commits: u64,
    maximum_metadata_bytes: u64,
) -> RuntimeWorldHistoryCatalogContract {
    let budgets = history_budgets(maximum_commits, maximum_metadata_bytes);
    RuntimeWorldHistoryCatalogContract::installed(
        budgets.retained_composite_commits(),
        budgets.history_metadata_bytes(),
    )
}
struct FixedClock;
impl crate::lifecycle::RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(0)
    }
}
pub(super) struct HistoryFixture {
    pub(super) authority: RuntimeWorldOwnerConstructionContract,
    pub(super) retention: crate::retention::RuntimeWorldRetentionOwner<(), (), ()>,
    _components: crate::branch::reference_test_fixture::RealReferenceFixture,
}
impl HistoryFixture {
    pub(super) fn history_pins(
        &self,
        commit: &CompositeRuntimeWorldCommit,
    ) -> crate::retention::HistoryRetentionObligation {
        self.retention
            .issue_publication(commit.basis())
            .unwrap()
            .fork_history(commit.basis())
            .unwrap()
    }
}

pub(super) fn linear_history(
    length: usize,
) -> (HistoryFixture, Vec<Arc<CompositeRuntimeWorldCommit>>) {
    assert!(length > 0);
    let mut components = crate::branch::reference_test_fixture::real_fixture(8, 8);
    let authority = RuntimeWorldOwnerConstructionContract::new().unwrap();
    let inputs = components.owner_inputs(
        history_budgets(length as u64, u64::MAX),
        crate::lifecycle::RuntimeWorldClock::from_source(FixedClock),
    );
    let (_, relational, signal, correspondence, _) = components.bootstrap_intent().into_parts();
    let basis = crate::basis::admit_current(
        authority.issuer(),
        &inputs.relational().basis_port(),
        &inputs.signal().basis_port(),
        inputs.bridge(),
        relational,
        signal,
        correspondence,
    )
    .unwrap();
    let retention = crate::retention::RuntimeWorldRetentionOwner::from_component_services(
        authority.owner_identity(),
        inputs.relational(),
        inputs.signal(),
        inputs.budgets().unique_exact_component_pins(),
        inputs.budgets().in_flight_pin_acquisition_reservations(),
        inputs.budgets().active_observations(),
    );
    let mut owner = HistoryFixture {
        authority,
        retention,
        _components: components,
    };
    let root = Arc::new(
        CompositeRuntimeWorldCommit::from_root_bootstrap(
            owner
                .authority
                .issuer_mut()
                .composite_commit()
                .expect("root identity"),
            basis.clone(),
            owner
                .authority
                .issuer_mut()
                .bootstrap_attempt()
                .expect("root attempt"),
            None,
        )
        .expect("root commit"),
    );
    let mut commits = vec![root];
    for _ in 1..length {
        let predecessor = commits.last().expect("linear predecessor");
        let commit = Arc::new(
            CompositeRuntimeWorldCommit::from_ordinary_publication(
                owner
                    .authority
                    .issuer_mut()
                    .composite_commit()
                    .expect("ordinary identity"),
                predecessor.as_ref(),
                basis.clone(),
                owner
                    .authority
                    .issuer_mut()
                    .publication_attempt()
                    .expect("ordinary attempt"),
                &CompositeOwnerExecutionResults::retained(),
                None,
            )
            .expect("ordinary commit"),
        );
        commits.push(commit);
    }
    (owner, commits)
}
