use super::candidates::{EligibleStrategyOperation, PlanningAlternative, PlanningAlternativeSet};
use super::selected_plan::SelectedAccessPlanBasis;
use super::selected_plan::{admit_selected_plan_budget, PlanSelectionBasis};
use super::{
    AccessPlanSelectionDenied, DeterministicSelectionRule, SelectionCandidateAudit,
    SelectionCandidateEligibility, SelectionCandidateOutcome,
};
use crate::access::budget::PlannedCounterEnvelope;
use crate::access::execution::AccessPathCounterSnapshot;
use crate::access::shape::AccessAuthorityPosture;
use crate::access::AdmittedAccessIntent;
use crate::artifact_family::AdmittedPhysicalArtifactFamily;
use crate::catalog::AuthorityRole;
use crate::keyspace::{AdmittedPhysicalAccessIdentity, AdmittedPhysicalKeyDomain};
use crate::materialization::AdmittedLayoutMaterialization;
use crate::strategy::LayoutStrategyFamily;
use worth_store_budgets::PreExecutionBudgetEnvelope;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum PlanSelectionDecision {
    LsmLookup(SelectedAccessPlanBasis, LsmLookupSelectionGrant),
    LsmRunPublication(SelectedAccessPlanBasis, LsmPublicationSelectionGrant),
    LsmReplayRecovery(SelectedAccessPlanBasis, LsmReplaySelectionGrant),
    LsmCompaction(SelectedAccessPlanBasis, LsmCompactionSelectionGrant),
    Degraded(SelectedAccessPlanBasis, DegradedScanSelectionGrant),
    Denied(AccessPlanSelectionDenied),
}

macro_rules! define_selection_grant {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(super) struct $name {
            _issued_by_decision_owner: (),
        }

        impl $name {
            const fn issue() -> Self {
                Self {
                    _issued_by_decision_owner: (),
                }
            }
        }
    };
}

define_selection_grant!(LsmLookupSelectionGrant);
define_selection_grant!(LsmPublicationSelectionGrant);
define_selection_grant!(LsmReplaySelectionGrant);
define_selection_grant!(LsmCompactionSelectionGrant);
define_selection_grant!(DegradedScanSelectionGrant);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessPlanSelectionCaseId {
    LsmLookup,
    LsmRunPublication,
    LsmReplayRecovery,
    LsmCompaction,
    DegradedExactScan,
    NoEligibleAlternative,
    CostDenied,
    BudgetDenied,
}

impl AccessPlanSelectionCaseId {
    pub const ALL: [Self; 8] = [
        Self::LsmLookup,
        Self::LsmRunPublication,
        Self::LsmReplayRecovery,
        Self::LsmCompaction,
        Self::DegradedExactScan,
        Self::NoEligibleAlternative,
        Self::CostDenied,
        Self::BudgetDenied,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LsmLookup => "layout.selection.lsm.lookup",
            Self::LsmRunPublication => "layout.selection.lsm.publication",
            Self::LsmReplayRecovery => "layout.selection.lsm.replay",
            Self::LsmCompaction => "layout.selection.lsm.compaction",
            Self::DegradedExactScan => "layout.selection.degraded_exact_scan",
            Self::NoEligibleAlternative => "layout.selection.denied.no_eligible_alternative",
            Self::CostDenied => "layout.selection.denied.cost",
            Self::BudgetDenied => "layout.selection.denied.budget",
        }
    }
}

pub(super) fn decide_access_plan(
    lifecycle: AdmittedPhysicalArtifactFamily,
    key_domain: AdmittedPhysicalKeyDomain,
    request_identity: AdmittedPhysicalAccessIdentity,
    materialization: Option<AdmittedLayoutMaterialization>,
    intent: AdmittedAccessIntent,
    admitted_budget: PreExecutionBudgetEnvelope,
) -> PlanSelectionDecision {
    if is_degraded(intent) {
        match decide_degraded_plan(
            lifecycle,
            key_domain,
            request_identity,
            materialization,
            intent,
            admitted_budget,
        ) {
            Ok(plan) => PlanSelectionDecision::Degraded(plan, DegradedScanSelectionGrant::issue()),
            Err(denial) => PlanSelectionDecision::Denied(denial),
        }
    } else {
        match decide_indexed_plan(
            lifecycle,
            key_domain,
            request_identity,
            materialization,
            intent,
            admitted_budget,
        ) {
            Ok(plan) => classify_indexed_operation(plan),
            Err(denial) => PlanSelectionDecision::Denied(denial),
        }
    }
}

fn classify_indexed_operation(plan: SelectedAccessPlanBasis) -> PlanSelectionDecision {
    match plan
        .selected_operation()
        .expect("indexed candidate retains its owner-classified operation")
    {
        EligibleStrategyOperation::LsmLookup => {
            PlanSelectionDecision::LsmLookup(plan, LsmLookupSelectionGrant::issue())
        }
        EligibleStrategyOperation::LsmRunPublication => {
            PlanSelectionDecision::LsmRunPublication(plan, LsmPublicationSelectionGrant::issue())
        }
        EligibleStrategyOperation::LsmReplayRecovery => {
            PlanSelectionDecision::LsmReplayRecovery(plan, LsmReplaySelectionGrant::issue())
        }
        EligibleStrategyOperation::LsmCompaction => {
            PlanSelectionDecision::LsmCompaction(plan, LsmCompactionSelectionGrant::issue())
        }
    }
}

fn decide_indexed_plan(
    lifecycle: AdmittedPhysicalArtifactFamily,
    key_domain: AdmittedPhysicalKeyDomain,
    request_identity: AdmittedPhysicalAccessIdentity,
    materialization: Option<AdmittedLayoutMaterialization>,
    intent: AdmittedAccessIntent,
    admitted_budget: PreExecutionBudgetEnvelope,
) -> Result<SelectedAccessPlanBasis, AccessPlanSelectionDenied> {
    let alternatives =
        PlanningAlternativeSet::derive(lifecycle, key_domain, materialization.clone(), intent);
    let (selected, selection_rule) = select_alternative(&alternatives)?;
    let strategy_admission = selected.snapshot().clone();
    let admitted_strategy = strategy_admission.admitted_strategy();
    let selected_family = admitted_strategy.family();
    let selected_operation = selected.operation();
    let planned_counter_envelope =
        crate::strategy::planned_counter_envelope_for(selected_family, intent.detail())
            .expect("eligible alternatives declare planned envelopes");
    admit_budget_for_plan(
        PlanSelectionBasis {
            family: lifecycle,
            key_domain,
            request_identity,
            materialization,
            strategy_admission: Some(strategy_admission),
            selected_family,
            selected_operation: Some(selected_operation),
            intent,
            planned_counter_envelope,
            selection_rule,
            primary_candidate: alternatives.primary_audit().clone(),
            secondary_candidate: alternatives.secondary_audit().clone(),
        },
        admitted_budget,
    )
}

fn decide_degraded_plan(
    lifecycle: AdmittedPhysicalArtifactFamily,
    key_domain: AdmittedPhysicalKeyDomain,
    request_identity: AdmittedPhysicalAccessIdentity,
    materialization: Option<AdmittedLayoutMaterialization>,
    intent: AdmittedAccessIntent,
    admitted_budget: PreExecutionBudgetEnvelope,
) -> Result<SelectedAccessPlanBasis, AccessPlanSelectionDenied> {
    let requested_rows = intent.budget_rows().unwrap_or(0);
    let planned_rows = u16::try_from(requested_rows).map_err(|_| {
        AccessPlanSelectionDenied::CostDenied(
            super::AccessPlanCostDenial::DegradedRowDemandNotRepresentable {
                requested_rows,
                maximum: u16::MAX as u64,
            },
        )
    })?;
    let planned_counter_envelope = PlannedCounterEnvelope::new(
        AccessPathCounterSnapshot::exact(
            0,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            planned_rows,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        )
        .with_allocation_events(2)
        .with_selected_plan_authority_allocation(),
        zero_counters(),
        zero_counters(),
    );
    admit_budget_for_plan(
        PlanSelectionBasis {
            family: lifecycle,
            key_domain,
            request_identity,
            materialization,
            strategy_admission: None,
            selected_family: LayoutStrategyFamily::ExactScan,
            selected_operation: None,
            intent,
            planned_counter_envelope,
            selection_rule: DeterministicSelectionRule::ExplicitDegradedExactScan,
            primary_candidate: SelectionCandidateAudit::new(
                LayoutStrategyFamily::ExactScan,
                AuthorityRole::SemanticAuthorityConsumer,
                SelectionCandidateOutcome::Eligible(
                    SelectionCandidateEligibility::ExplicitDegradedExactScan {
                        planned_counter_envelope,
                        budget_rows: requested_rows,
                    },
                ),
            ),
            secondary_candidate: SelectionCandidateAudit::new(
                LayoutStrategyFamily::SparseIndex,
                AuthorityRole::SemanticAuthorityConsumer,
                SelectionCandidateOutcome::Rejected(
                    super::SelectionCandidateRejection::NotApplicableToExplicitDegradedScan,
                ),
            ),
        },
        admitted_budget,
    )
}

fn select_alternative(
    alternatives: &PlanningAlternativeSet,
) -> Result<(PlanningAlternative, DeterministicSelectionRule), AccessPlanSelectionDenied> {
    alternatives
        .selected()
        .cloned()
        .map(|selected| (selected, DeterministicSelectionRule::SoleEligibleCandidate))
        .ok_or(AccessPlanSelectionDenied::NoEligibleAlternative)
}

fn admit_budget_for_plan(
    basis: PlanSelectionBasis,
    admitted_budget: PreExecutionBudgetEnvelope,
) -> Result<SelectedAccessPlanBasis, AccessPlanSelectionDenied> {
    admit_selected_plan_budget(basis, admitted_budget)
}

const fn is_degraded(intent: AdmittedAccessIntent) -> bool {
    matches!(
        intent.authority_posture(),
        AccessAuthorityPosture::ExplicitDegradedExactScan
    )
}

const fn zero_counters() -> AccessPathCounterSnapshot {
    AccessPathCounterSnapshot::exact(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
}
