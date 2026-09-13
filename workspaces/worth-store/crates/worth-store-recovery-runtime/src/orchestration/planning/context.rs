use worth_store_recovery_physics::{
    PhysicalRedoPlanningDenial, PhysicalSourceSelection, RecoveryPlanCostDenial,
    RecoveryPlanningCounters,
};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryLimitFailure, PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoveryRootProtocolDenial,
    PhysicalRecoverySourceDenial,
};
use crate::progression::{
    PhysicalRecoveryDiscoveryCounters, RecoveryIntegrityEvidence, SelectedPhysicalRecovery,
};

use super::super::RecoveryCoordination;
use super::denial::{
    block, block_with_planning_attempt_denial, cost_denial_block, redo_block, redo_denial_block,
};

pub(super) struct PlanningContext {
    pub(super) authority: AdmittedPlatformAuthority,
    pub(super) coordination: RecoveryCoordination,
    pub(super) selection: PhysicalSourceSelection,
    pub(super) integrity: RecoveryIntegrityEvidence,
    pub(super) counters: PhysicalRecoveryDiscoveryCounters,
    pub(super) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(super) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(super) limits: PhysicalRecoveryLimitDeclaration,
    pub(super) effects_before: u64,
    pub(super) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

impl PlanningContext {
    pub(super) fn from_selected(selected: SelectedPhysicalRecovery) -> Self {
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
        Self {
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
        }
    }

    pub(super) fn block(
        self,
        kind: PhysicalRecoveryBlockKind,
        artifact: &str,
        limit: Option<PhysicalRecoveryLimitFailure>,
    ) -> PhysicalRecoveryOutcome {
        let integrity_trace = self.integrity_trace.clone();
        block(
            self.authority,
            self.coordination,
            kind,
            self.counters,
            self.root_protocol_counters,
            artifact,
            limit,
            self.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(self.integrity.into_observations())
    }

    pub(super) fn block_with_planning_attempt_denial(
        self,
        kind: PhysicalRecoveryBlockKind,
        planning_counters: RecoveryPlanningCounters,
        artifact: &str,
        limit: Option<PhysicalRecoveryLimitFailure>,
        denial: PhysicalRecoveryPlanningDenial,
    ) -> PhysicalRecoveryOutcome {
        let integrity_trace = self.integrity_trace.clone();
        block_with_planning_attempt_denial(
            self.authority,
            self.coordination,
            kind,
            self.counters,
            planning_counters,
            self.root_protocol_counters,
            artifact,
            limit,
            denial,
            self.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(self.integrity.into_observations())
    }

    pub(super) fn root_protocol_block(
        mut self,
        planning_counters: RecoveryPlanningCounters,
        artifact: PhysicalRecoveryRootProtocolArtifact,
        denial: PhysicalRecoveryRootProtocolDenial,
    ) -> PhysicalRecoveryOutcome {
        let integrity_trace = self.integrity_trace.clone();
        self.root_protocol_denials
            .push(PhysicalRecoverySourceDenial::RootProtocol { artifact, denial });
        super::denial::block_with_root_protocol_counters(
            self.authority,
            self.coordination,
            PhysicalRecoveryBlockKind::RootProtocol,
            self.counters,
            planning_counters,
            self.root_protocol_counters,
            "staged current-root selector closeout",
            None,
            None,
            self.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(self.integrity.into_observations())
    }

    pub(super) fn successor_candidate_block(
        self,
        planning_counters: RecoveryPlanningCounters,
        artifact: &str,
        limit: Option<PhysicalRecoveryLimitFailure>,
        denial: crate::entry::PhysicalRecoverySuccessorCandidateDenial,
    ) -> PhysicalRecoveryOutcome {
        let integrity_trace = self.integrity_trace.clone();
        super::denial::block_with_root_protocol_counters(
            self.authority,
            self.coordination,
            PhysicalRecoveryBlockKind::PageAdmission,
            self.counters,
            planning_counters,
            self.root_protocol_counters,
            artifact,
            limit,
            Some(PhysicalRecoveryPlanningDenial::SuccessorCandidate(denial)),
            self.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(self.integrity.into_observations())
    }

    pub(super) fn redo_block(
        self,
        planning_counters: RecoveryPlanningCounters,
        limit: Option<PhysicalRecoveryLimitFailure>,
    ) -> PhysicalRecoveryOutcome {
        let integrity_trace = self.integrity_trace.clone();
        redo_block(
            self.authority,
            self.coordination,
            self.counters,
            planning_counters,
            self.root_protocol_counters,
            limit,
            self.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(self.integrity.into_observations())
    }

    pub(super) fn redo_denial_block(
        self,
        planning_counters: RecoveryPlanningCounters,
        limit: Option<PhysicalRecoveryLimitFailure>,
        denial: PhysicalRedoPlanningDenial,
    ) -> PhysicalRecoveryOutcome {
        let integrity_trace = self.integrity_trace.clone();
        redo_denial_block(
            self.authority,
            self.coordination,
            self.counters,
            planning_counters,
            self.root_protocol_counters,
            limit,
            denial,
            self.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(self.integrity.into_observations())
    }

    pub(super) fn cost_denial_block(
        self,
        planning_counters: RecoveryPlanningCounters,
        denial: RecoveryPlanCostDenial,
        limit: PhysicalRecoveryLimitFailure,
    ) -> PhysicalRecoveryOutcome {
        let integrity_trace = self.integrity_trace.clone();
        cost_denial_block(
            self.authority,
            self.coordination,
            self.counters,
            planning_counters,
            self.root_protocol_counters,
            denial,
            limit,
            self.root_protocol_denials,
        )
        .with_integrity_trace(integrity_trace)
        .with_block_integrity_observations(self.integrity.into_observations())
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
