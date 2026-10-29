pub(crate) use {std::collections::BTreeSet};

pub(crate) use {worth_store_formal_models::map_compaction_observation, worth_store_formal_models::map_lsm_maintenance_observation, worth_store_formal_models::map_lsm_membership_observation, worth_store_formal_models::CompactionVisibilityAction, worth_store_formal_models::LsmMaintenanceAction, worth_store_formal_models::LsmMaintenanceDenial, worth_store_formal_models::LsmMembershipAction, worth_store_formal_models::LsmMembershipDenial, worth_store_formal_models::ModeledOutcome};
pub(crate) use {worth_store_physical_isolation::CompactionCutoverDelta, worth_store_physical_isolation::CompactionOwnerCaseObservation, worth_store_physical_isolation::CompactionReadInterlockPlan};
pub(crate) use {worth_store_test_support::harness::observe_lsm_maintenance_owner_cases, worth_store_test_support::harness::observe_lsm_owner_cases, worth_store_test_support::harness::physical_isolation::compaction::admitted_compaction_plan};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ExecutedOwnerObservation {
    LsmMembership(worth_store_lsm_authority::LsmMembershipOwnerCaseObservation),
    LsmMaintenance(worth_store_layout_indexes::LsmMaintenanceOwnerCaseObservation),
    PhysicalCompaction(CompactionOwnerCaseObservation),
}

pub(crate) fn expected_actions() -> BTreeSet<CompactionVisibilityAction> {
    use {worth_store_formal_models::LsmMaintenanceAction::AdmitCompaction, worth_store_formal_models::LsmMaintenanceAction::AdmitRunPublication};
    use {worth_store_formal_models::LsmMembershipAction::Open, worth_store_formal_models::LsmMembershipAction::PersistRecord, worth_store_formal_models::LsmMembershipAction::SelectCompaction};
    std::collections::BTreeSet::from([
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Admitted,
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::CanonicalKeyRequired),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::DurableRecordBindingMismatch),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::UnsupportedRecordKind),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::MembershipAmbiguous),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::MembershipStale),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::ManifestMembershipMismatch),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::ReplacementOutputMismatch),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(
                worth_store_formal_models::LsmMembershipDenial::PersistedMembershipArtifactInvalid,
            ),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: Open,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::Io),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: worth_store_formal_models::ModeledOutcome::Admitted,
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::UnsupportedRecordKind),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::StoreBindingMismatch),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::DurableRecordBindingMismatch),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: PersistRecord,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::MembershipAmbiguous),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: worth_store_formal_models::ModeledOutcome::Admitted,
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::ValueRecordRequired),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::GenerationRecordRequired),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::TombstoneRecordRequired),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMembership {
            operation: SelectCompaction,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMembershipDenial::ReplacementOutputMismatch),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitRunPublication,
            outcome: worth_store_formal_models::ModeledOutcome::Admitted,
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitRunPublication,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMaintenanceDenial::Budget),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitRunPublication,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMaintenanceDenial::SecurityScope),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitCompaction,
            outcome: worth_store_formal_models::ModeledOutcome::Admitted,
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitCompaction,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMaintenanceDenial::Budget),
        },
        worth_store_formal_models::CompactionVisibilityAction::LsmMaintenance {
            operation: AdmitCompaction,
            outcome: worth_store_formal_models::ModeledOutcome::Denied(worth_store_formal_models::LsmMaintenanceDenial::SecurityScope),
        },
        worth_store_formal_models::CompactionVisibilityAction::LowerRewrite,
    ])
}

pub(crate) fn execute_owner_observations() -> Vec<ExecutedOwnerObservation> {
    let mut observations = observe_lsm_owner_cases()
        .membership()
        .map(crate::compaction_visibility_owner_execution::ExecutedOwnerObservation::LsmMembership)
        .collect::<Vec<_>>();
    observations.extend(
        observe_lsm_maintenance_owner_cases()
            .into_iter()
            .map(crate::compaction_visibility_owner_execution::ExecutedOwnerObservation::LsmMaintenance),
    );
    observations.extend(
        lower_compaction_observations(admitted_compaction_plan())
            .into_iter()
            .map(crate::compaction_visibility_owner_execution::ExecutedOwnerObservation::PhysicalCompaction),
    );
    observations
}

pub(crate) fn lower_compaction_observations(
    plan: CompactionReadInterlockPlan,
) -> Vec<CompactionOwnerCaseObservation> {
    let manifest_epoch = plan.protected().root().manifest_epoch().get() + 1;
    let delta = worth_store_physical_isolation::CompactionCutoverDelta::lower_to_manifest(plan, manifest_epoch)
        .expect("admitted compaction plan lowers a rewrite candidate");
    vec![delta.owner_case_observation()]
}

impl ExecutedOwnerObservation {
    pub(crate) fn map(self) -> CompactionVisibilityAction {
        match self {
            Self::LsmMembership(observation) => map_lsm_membership_observation(observation),
            Self::LsmMaintenance(observation) => map_lsm_maintenance_observation(observation),
            Self::PhysicalCompaction(observation) => map_compaction_observation(observation),
        }
    }
}