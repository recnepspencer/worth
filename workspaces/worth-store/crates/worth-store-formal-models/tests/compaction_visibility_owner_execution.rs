use std::collections::BTreeSet;

use worth_store_formal_models::{
    map_compaction_observation, map_lsm_maintenance_observation, map_lsm_membership_observation,
    CompactionVisibilityAction, LsmMaintenanceAction, LsmMaintenanceDenial, LsmMembershipAction,
    LsmMembershipDenial, ModeledOutcome,
};
use worth_store_physical_isolation::{
    CompactionCutoverDelta, CompactionOwnerCaseObservation, CompactionReadInterlockPlan,
};
use worth_store_test_support::harness::{
    observe_lsm_maintenance_owner_cases, observe_lsm_owner_cases,
    physical_isolation::compaction::admitted_compaction_plan,
};

#[derive(Debug, Clone, Copy)]
enum ExecutedOwnerObservation {
    LsmMembership(worth_store_lsm_authority::LsmMembershipOwnerCaseObservation),
    LsmMaintenance(worth_store_layout_indexes::LsmMaintenanceOwnerCaseObservation),
    PhysicalCompaction(CompactionOwnerCaseObservation),
}

#[test]
fn ordinary_owner_execution_maps_exact_actual_outcomes() {
    let observations = execute_owner_observations();
    let actions = observations
        .iter()
        .copied()
        .map(ExecutedOwnerObservation::map)
        .collect::<BTreeSet<_>>();

    assert_eq!(actions, expected_actions());
}

fn expected_actions() -> BTreeSet<CompactionVisibilityAction> {
    use LsmMaintenanceAction::{AdmitCompaction, AdmitRunPublication};
    use LsmMembershipAction::{Open, PersistRecord, SelectCompaction};
    BTreeSet::from([
        CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: ModeledOutcome::Admitted,
        },
        CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::CanonicalKeyRequired),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::DurableRecordBindingMismatch),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::UnsupportedRecordKind),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::MembershipAmbiguous),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: ModeledOutcome::Denied(
                LsmMembershipDenial::PersistedMembershipArtifactInvalid,
            ),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::Io),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: ModeledOutcome::Admitted,
        },
        CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::UnsupportedRecordKind),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::StoreBindingMismatch),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::DurableRecordBindingMismatch),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::MembershipAmbiguous),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: ModeledOutcome::Admitted,
        },
        CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::ValueRecordRequired),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::GenerationRecordRequired),
        },
        CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: ModeledOutcome::Denied(LsmMembershipDenial::TombstoneRecordRequired),
        },
        CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitRunPublication,
            outcome: ModeledOutcome::Admitted,
        },
        CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitRunPublication,
            outcome: ModeledOutcome::Denied(LsmMaintenanceDenial::Budget),
        },
        CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitRunPublication,
            outcome: ModeledOutcome::Denied(LsmMaintenanceDenial::SecurityScope),
        },
        CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitCompaction,
            outcome: ModeledOutcome::Admitted,
        },
        CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitCompaction,
            outcome: ModeledOutcome::Denied(LsmMaintenanceDenial::Budget),
        },
        CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitCompaction,
            outcome: ModeledOutcome::Denied(LsmMaintenanceDenial::SecurityScope),
        },
        CompactionVisibilityAction::LowerRewrite,
    ])
}

fn execute_owner_observations() -> Vec<ExecutedOwnerObservation> {
    let mut observations = observe_lsm_owner_cases()
        .membership()
        .map(ExecutedOwnerObservation::LsmMembership)
        .collect::<Vec<_>>();
    observations.extend(
        observe_lsm_maintenance_owner_cases()
            .into_iter()
            .map(ExecutedOwnerObservation::LsmMaintenance),
    );
    observations.extend(
        lower_compaction_observations(admitted_compaction_plan())
            .into_iter()
            .map(ExecutedOwnerObservation::PhysicalCompaction),
    );
    observations
}

fn lower_compaction_observations(
    plan: CompactionReadInterlockPlan,
) -> Vec<CompactionOwnerCaseObservation> {
    let manifest_epoch = plan.protected().root().manifest_epoch().get() + 1;
    let delta = CompactionCutoverDelta::lower_to_manifest(plan, manifest_epoch)
        .expect("admitted compaction plan lowers a rewrite candidate");
    vec![delta.owner_case_observation()]
}

impl ExecutedOwnerObservation {
    fn map(self) -> CompactionVisibilityAction {
        match self {
            Self::LsmMembership(observation) => map_lsm_membership_observation(observation),
            Self::LsmMaintenance(observation) => map_lsm_maintenance_observation(observation),
            Self::PhysicalCompaction(observation) => map_compaction_observation(observation),
        }
    }
}
