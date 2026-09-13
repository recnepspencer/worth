use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum CiTestLane {
    OwnerUnit,
    Scenario,
    ProcessScenario,
    Ui,
    Formal,
    Structural,
}

impl CiTestLane {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::OwnerUnit => "owner-unit",
            Self::Scenario => "scenario",
            Self::ProcessScenario => "process-scenario",
            Self::Ui => "ui",
            Self::Formal => "formal",
            Self::Structural => "structural",
        }
    }
}

impl fmt::Display for CiTestLane {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for CiTestLane {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "owner-unit" => Ok(Self::OwnerUnit),
            "scenario" => Ok(Self::Scenario),
            "process-scenario" => Ok(Self::ProcessScenario),
            "ui" => Ok(Self::Ui),
            "formal" => Ok(Self::Formal),
            "structural" => Ok(Self::Structural),
            _ => Err(format!(
                "unknown CI lane `{value}`; expected owner-unit, scenario, process-scenario, ui, formal, or structural"
            )),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum TestProduct {
    Owner {
        package: String,
    },
    Smoke,
    Ui,
    Ci {
        lane: CiTestLane,
        shard: Option<(usize, usize)>,
    },
}

impl TestProduct {
    pub(crate) fn name(&self) -> String {
        match self {
            Self::Owner { package } => format!("owner:{package}"),
            Self::Smoke => "smoke".into(),
            Self::Ui => "ui".into(),
            Self::Ci { lane, shard } => match shard {
                Some((index, count)) => format!("ci:{lane}:{index}/{count}"),
                None => format!("ci:{lane}"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct SmokeCase {
    pub(crate) package: &'static str,
    pub(crate) target: &'static str,
    pub(crate) filter: &'static str,
    pub(crate) feature: Option<&'static str>,
}

pub(crate) fn smoke_cases() -> &'static [SmokeCase] {
    SMOKE_CASES
}

const SMOKE_CASES: &[SmokeCase] = &[
        smoke(
            "worth-store-certification",
            "io_scheduling",
            "access_policy::certification_materializes_successful_execution_receipts",
        ),
        smoke(
            "worth-store-certification",
            "layout_access",
            "btree_lookup_authority::ordinary_runtime_selects_and_executes_separator_directed_page_lookup",
        ),
        smoke(
            "worth-store-certification",
            "operational_security",
            "security_scope_propagation::stable_read_scope_survives_protection_observation_and_decode_entry",
        ),
        smoke(
            "worth-store-certification",
            "physical_isolation",
            "stable_read_plan_admission::proof_bearing_read_plan_admits_before_execution_handle",
        ),
        store_smoke("baseline_admission::empty_bootstrap_create_and_reopen_converge"),
        store_smoke("scan_journeys::scan_batch_widths_converge_to_one_physical_sequence"),
        store_smoke("publication_faults::possible_catalog_cutover_is_typed_indeterminate_and_close_adds_no_publication_effect"),
        store_smoke("physical_work::serving_frame_residency::pins_distinguish_faults_hits_overpin_and_refault_without_another_runtime"),
        store_smoke("record_chunk_views::borrowed_access::inline_view_exposes_only_the_record_payload_and_observational_basis"),
        store_smoke("record_chunk_views::bounded_copy::bounded_copies_and_views_share_one_cursor_with_exact_copy_evidence"),
        store_smoke("ordinary_writeback_failures::ordinary_candidate_tail_no_effect_is_typed_and_discards_dirty_residency"),
        store_smoke("physical_work::speculative_residency::outcomes::cold_hot_and_mixed_speculation_reconcile_work_media_and_residency_truth"),
];

const fn smoke(package: &'static str, target: &'static str, filter: &'static str) -> SmokeCase {
    SmokeCase {
        package,
        target,
        filter,
        feature: None,
    }
}

const fn store_smoke(filter: &'static str) -> SmokeCase {
    SmokeCase {
        package: "worth-store",
        target: "physical_record_journeys",
        filter,
        feature: Some("worth-store/certification-test-authority"),
    }
}
