use worth_store_recovery_physics::{
    PhysicalRedoPlanningDenial, RecoveryPlanCostDenial, RecoveryPlanningCounters,
};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlanningDenial, PhysicalRecoveryRootProtocolArtifact,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial,
};
use crate::orchestration::recovery_budget::RecoveryAllowance;
use crate::progression::{
    PhysicalRecoveryDiscoveryCounters, PlanningMemoryDenial, PlanningResidentAllowance,
    RecoveryIntegrityEvidence, SelectedPhysicalRecovery,
};

use super::super::{RecoveryCoordination, ResidentSourceSelection};
use super::denial::{
    block, block_with_planning_attempt_denial, cost_denial_block, redo_block, redo_denial_block,
};

pub(super) type PlanningContext = Box<PlanningContextBody>;

pub(super) struct PlanningContextBody {
    pub(super) authority: AdmittedPlatformAuthority,
    pub(super) coordination: RecoveryCoordination,
    pub(super) selection: ResidentSourceSelection,
    pub(super) integrity: RecoveryIntegrityEvidence,
    pub(super) counters: PhysicalRecoveryDiscoveryCounters,
    pub(super) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(super) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(super) limits: PhysicalRecoveryLimitDeclaration,
    pub(super) effects_before: u64,
    pub(super) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

impl PlanningContextBody {
    pub(super) fn from_selected(
        selected: SelectedPhysicalRecovery,
    ) -> Result<PlanningContext, PhysicalRecoveryOutcome> {
        let (
            authority,
            coordination,
            selection,
            integrity,
            counters,
            root_protocol_denials,
            integrity_trace,
        ) = selected.into_parts();
        let limits = authority.limits.declaration();
        let effects_before = authority.media.recovery_effect_count();
        let live_before = super::resident_memory::selected_live_bytes(
            &coordination,
            &selection,
            &integrity,
            &root_protocol_denials,
            &integrity_trace,
        )
        .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX });
        let slot_bytes = PlanningResidentAllowance::slot_bytes::<Self>(1);
        let admitted = live_before.and_then(|live| {
            let mut allowance =
                PlanningResidentAllowance::new(live, limits.recovery_memory_bytes)?;
            allowance.retain(slot_bytes?)
        });
        if let Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed }) = admitted {
            let limit = RecoveryAllowance::declared(
                &limits,
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            )
            .past(observed);
            return Err(cost_denial_block(
                authority,
                coordination,
                counters,
                RecoveryPlanningCounters::default().with_peak_recovery_bytes(observed),
                Default::default(),
                RecoveryPlanCostDenial::RecoveryMemoryBytes,
                limit,
                root_protocol_denials,
            )
            .with_integrity_trace(integrity_trace)
            .with_block_integrity_observations(integrity.into_observations()));
        } else if let Err(PlanningMemoryDenial::Allocation {
            requested_bytes,
            cause,
        }) = admitted
        {
            return Err(block_with_planning_attempt_denial(
                authority,
                coordination,
                PhysicalRecoveryBlockKind::RedoPlanning,
                counters,
                RecoveryPlanningCounters::default(),
                Default::default(),
                "planning-context-allocation",
                None,
                PhysicalRecoveryPlanningDenial::ResolvedStateAllocation {
                    requested_bytes,
                    cause,
                },
                root_protocol_denials,
            )
            .with_integrity_trace(integrity_trace)
            .with_block_integrity_observations(integrity.into_observations()));
        }
        Ok(Box::new(Self {
            authority,
            coordination,
            selection,
            integrity,
            counters,
            root_protocol_denials,
            root_protocol_counters: Default::default(),
            limits,
            effects_before,
            integrity_trace,
        }))
    }

    #[inline(never)]
    pub(super) fn block(
        self: Box<Self>,
        kind: PhysicalRecoveryBlockKind,
        artifact: &str,
        limit: Option<PhysicalRecoveryLimitFailure>,
    ) -> PhysicalRecoveryOutcome {
        let body = *self;
        let integrity_trace = body.integrity_trace;
        block(
            body.authority,
            body.coordination,
            kind,
            body.counters,
            body.root_protocol_counters,
            artifact,
            limit,
            body.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(body.integrity.into_observations())
    }

    #[inline(never)]
    pub(super) fn block_with_planning_attempt_denial(
        self: Box<Self>,
        kind: PhysicalRecoveryBlockKind,
        planning_counters: RecoveryPlanningCounters,
        artifact: &str,
        limit: Option<PhysicalRecoveryLimitFailure>,
        denial: PhysicalRecoveryPlanningDenial,
    ) -> PhysicalRecoveryOutcome {
        let body = *self;
        let integrity_trace = body.integrity_trace;
        block_with_planning_attempt_denial(
            body.authority,
            body.coordination,
            kind,
            body.counters,
            planning_counters,
            body.root_protocol_counters,
            artifact,
            limit,
            denial,
            body.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(body.integrity.into_observations())
    }

    #[inline(never)]
    pub(super) fn root_protocol_block(
        self: Box<Self>,
        planning_counters: RecoveryPlanningCounters,
        artifact: PhysicalRecoveryRootProtocolArtifact,
        denial: PhysicalRecoveryRootProtocolDenial,
    ) -> PhysicalRecoveryOutcome {
        let mut body = *self;
        let integrity_trace = body.integrity_trace;
        body.root_protocol_denials
            .push(PhysicalRecoverySourceDenial::RootProtocol { artifact, denial });
        super::denial::block_with_root_protocol_counters(
            body.authority,
            body.coordination,
            PhysicalRecoveryBlockKind::RootProtocol,
            body.counters,
            planning_counters,
            body.root_protocol_counters,
            "staged current-root selector closeout",
            None,
            None,
            body.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(body.integrity.into_observations())
    }

    /// Page observation stopped: at a limit, which carries its own counts and
    /// names no page, or with what observation says of the pages.
    #[inline(never)]
    pub(super) fn page_block(
        self: Box<Self>,
        planning_counters: RecoveryPlanningCounters,
        artifact: &str,
        limit: Option<PhysicalRecoveryLimitFailure>,
        denial: Option<crate::entry::PhysicalRecoveryPageAdmissionDenial>,
    ) -> PhysicalRecoveryOutcome {
        let body = *self;
        let integrity_trace = body.integrity_trace;
        super::denial::block_with_root_protocol_counters(
            body.authority,
            body.coordination,
            PhysicalRecoveryBlockKind::PageAdmission,
            body.counters,
            planning_counters,
            body.root_protocol_counters,
            artifact,
            limit,
            denial.map(PhysicalRecoveryPlanningDenial::Page),
            body.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(body.integrity.into_observations())
    }

    #[inline(never)]
    pub(super) fn successor_candidate_block(
        self: Box<Self>,
        planning_counters: RecoveryPlanningCounters,
        artifact: &str,
        limit: Option<PhysicalRecoveryLimitFailure>,
        denial: crate::entry::PhysicalRecoverySuccessorCandidateDenial,
    ) -> PhysicalRecoveryOutcome {
        let body = *self;
        let integrity_trace = body.integrity_trace;
        super::denial::block_with_root_protocol_counters(
            body.authority,
            body.coordination,
            PhysicalRecoveryBlockKind::PageAdmission,
            body.counters,
            planning_counters,
            body.root_protocol_counters,
            artifact,
            limit,
            Some(PhysicalRecoveryPlanningDenial::SuccessorCandidate(denial)),
            body.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(body.integrity.into_observations())
    }

    #[inline(never)]
    pub(super) fn redo_block(
        self: Box<Self>,
        planning_counters: RecoveryPlanningCounters,
        limit: Option<PhysicalRecoveryLimitFailure>,
    ) -> PhysicalRecoveryOutcome {
        let body = *self;
        let integrity_trace = body.integrity_trace;
        redo_block(
            body.authority,
            body.coordination,
            body.counters,
            planning_counters,
            body.root_protocol_counters,
            limit,
            body.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(body.integrity.into_observations())
    }

    #[inline(never)]
    pub(super) fn redo_denial_block(
        self: Box<Self>,
        planning_counters: RecoveryPlanningCounters,
        denial: PhysicalRedoPlanningDenial,
    ) -> PhysicalRecoveryOutcome {
        let body = *self;
        let integrity_trace = body.integrity_trace;
        let limit =
            super::redo_limit::redo_limit(&body.limits, body.counters.manifest_entries, denial);
        redo_denial_block(
            body.authority,
            body.coordination,
            body.counters,
            planning_counters,
            body.root_protocol_counters,
            limit,
            denial,
            body.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(body.integrity.into_observations())
    }

    #[inline(never)]
    pub(super) fn cost_denial_block(
        self: Box<Self>,
        planning_counters: RecoveryPlanningCounters,
        denial: RecoveryPlanCostDenial,
        limit: Option<PhysicalRecoveryLimitFailure>,
    ) -> PhysicalRecoveryOutcome {
        let body = *self;
        let integrity_trace = body.integrity_trace;
        cost_denial_block(
            body.authority,
            body.coordination,
            body.counters,
            planning_counters,
            body.root_protocol_counters,
            denial,
            limit,
            body.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(body.integrity.into_observations())
    }

    pub(super) fn record_successor_root_route(
        &mut self,
        route: crate::entry::PhysicalRecoveryRootProtocolCounters,
    ) {
        self.root_protocol_counters = self.root_protocol_counters.with_successor_root_route(route);
    }

    pub(super) fn record_staged_selector_route(
        &mut self,
        route: crate::entry::PhysicalRecoveryRootProtocolCounters,
    ) {
        self.root_protocol_counters = self
            .root_protocol_counters
            .with_staged_selector_route(route);
    }
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
