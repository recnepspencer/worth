use worth_store::physical_runtime::{
    PhysicalRecoveryPublicationCommand, PhysicalRecoveryPublicationCommandOutcome,
};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlock, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoveryPublicationCounters,
    PhysicalRecoveryPublicationDenial, PhysicalRecoveryPublicationIndeterminate,
    PhysicalRecoveryPublicationSettlement, PhysicalRecoveryPublicationSettlementLedger,
};
use crate::progression::{
    NamespaceDurablePhysicalRecovery, NamespaceDurableState, RecoveryPublicationExpectation,
    RecoveryPublicationPlan, StagedPhysicalRecovery,
};

pub(super) struct PublicationState {
    authority: AdmittedPlatformAuthority,
    coordination: super::super::RecoveryCoordination,
    selection: worth_store_recovery_physics::PhysicalSourceSelection,
    discovery: crate::progression::PhysicalRecoveryDiscoveryCounters,
    root_protocol_denials: Vec<crate::entry::PhysicalRecoverySourceDenial>,
    integrity: crate::progression::RecoveryIntegrityEvidence,
    integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    freshness: worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample,
    fates: crate::handoff::RecoveryOperationFateSet,
    planning: worth_store_recovery_physics::RecoveryPlanningCounters,
    root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    base: crate::progression::RecoveryBaseImagePlan,
    quiescence: crate::progression::RecoveryQuiescencePlan,
    closed: crate::progression::ClosedRecoveryStagingGeneration,
    staging: crate::entry::PhysicalRecoveryStagingCounters,
    staging_settlements: crate::entry::PhysicalRecoveryStagingSettlementLedger,
}

impl PublicationState {
    pub(super) fn from_staged(staged: StagedPhysicalRecovery) -> (Self, RecoveryPublicationPlan) {
        let StagedPhysicalRecovery {
            authority,
            coordination,
            selection,
            discovery_counters,
            root_protocol_denials,
            integrity,
            integrity_trace,
            freshness,
            fates,
            planning_counters,
            root_protocol_counters,
            base,
            publication,
            quiescence,
            closed,
            staging_counters,
            staging_settlements,
        } = staged;
        (
            Self {
                authority,
                coordination,
                selection,
                discovery: discovery_counters,
                root_protocol_denials,
                integrity,
                integrity_trace,
                freshness,
                fates,
                planning: planning_counters,
                root_protocol_counters,
                base,
                quiescence,
                closed,
                staging: staging_counters,
                staging_settlements,
            },
            publication,
        )
    }

    pub(super) const fn planned_effects(&self) -> u64 {
        self.quiescence.publication_commands()
    }

    pub(super) fn execute(
        &self,
        command: PhysicalRecoveryPublicationCommand,
    ) -> PhysicalRecoveryPublicationCommandOutcome {
        self.coordination
            .owner()
            .execute_publication_command(&self.authority.media, command)
    }

    pub(super) fn is_ready(&self) -> bool {
        self.coordination.is_ready()
    }

    pub(super) fn publish_preexisting(
        self,
        expectation: RecoveryPublicationExpectation,
    ) -> Result<NamespaceDurablePhysicalRecovery, PhysicalRecoveryOutcome> {
        let selected = self.selection.root().selected();
        if self.planned_effects() != 0
            || self.closed.artifact_count() != 0
            || self.closed.byte_count() != 0
            || expectation.current_selector() != selected.selector()
            || expectation.recovered_root() != selected.manifest()
            || !self.is_ready()
        {
            return Err(self.block_invalid());
        }
        Ok(self.into_namespace(
            expectation,
            PhysicalRecoveryPublicationCounters::default(),
            PhysicalRecoveryPublicationSettlement::PreexistingNamespaceDurable,
        ))
    }

    pub(super) fn into_namespace(
        self,
        expectation: RecoveryPublicationExpectation,
        counters: PhysicalRecoveryPublicationCounters,
        settlement: PhysicalRecoveryPublicationSettlement,
    ) -> NamespaceDurablePhysicalRecovery {
        NamespaceDurablePhysicalRecovery::new(
            NamespaceDurableState {
                authority: self.authority,
                coordination: self.coordination,
                selection: self.selection,
                discovery_counters: self.discovery,
                root_protocol_denials: self.root_protocol_denials,
                integrity: self.integrity,
                integrity_trace: self.integrity_trace,
                freshness: self.freshness,
                fates: self.fates,
                planning_counters: self.planning,
                root_protocol_counters: self.root_protocol_counters,
                base: self.base,
                quiescence: self.quiescence,
                closed: self.closed,
                staging_counters: self.staging,
                staging_settlements: self.staging_settlements,
            },
            expectation,
            counters,
            PhysicalRecoveryPublicationSettlementLedger::new(settlement),
        )
    }

    pub(super) fn block_invalid(self) -> PhysicalRecoveryOutcome {
        let counters = PhysicalRecoveryPublicationCounters {
            planned_effects: self.planned_effects(),
            ..PhysicalRecoveryPublicationCounters::default()
        };
        self.block(
            counters,
            Some(PhysicalRecoveryPublicationDenial::InvalidPlan),
            None,
        )
    }

    pub(super) fn block_settlement(
        self,
        counters: PhysicalRecoveryPublicationCounters,
        settlement: PhysicalRecoveryPublicationSettlement,
    ) -> PhysicalRecoveryOutcome {
        self.block(counters, None, Some(settlement))
    }

    fn block(
        self,
        counters: PhysicalRecoveryPublicationCounters,
        denial: Option<PhysicalRecoveryPublicationDenial>,
        settlement: Option<PhysicalRecoveryPublicationSettlement>,
    ) -> PhysicalRecoveryOutcome {
        assert!(self.coordination.shutdown_is_quiescent());
        let store = self.authority.media.store_identity();
        let session_identity = self.authority.session.identity();
        let recovery_effects = self.authority.media.recovery_effect_count();
        let AdmittedPlatformAuthority { media, session, .. } = self.authority;
        drop(media);
        session.block();
        PhysicalRecoveryOutcome::Blocked(PhysicalRecoveryBlock::new(
            PhysicalRecoveryBlockKind::Publication,
            store,
            session_identity,
            PhysicalRecoveryBlockEvidence {
                counters: self.discovery,
                source_denials: self.root_protocol_denials,
                integrity_observations: self.integrity.into_observations(),
                integrity_trace: self.integrity_trace,
                planning_counters: Some(self.planning),
                root_protocol_counters: Some(self.root_protocol_counters),
                staging_counters: Some(self.staging),
                staging_settlements: Some(self.staging_settlements),
                publication_counters: Some(counters),
                publication_denial: denial,
                publication_settlements: settlement
                    .map(PhysicalRecoveryPublicationSettlementLedger::new),
                ..PhysicalRecoveryBlockEvidence::default()
            },
            recovery_effects,
        ))
    }

    pub(super) fn indeterminate(
        self,
        counters: PhysicalRecoveryPublicationCounters,
        settlement: PhysicalRecoveryPublicationSettlement,
    ) -> PhysicalRecoveryOutcome {
        assert!(self.coordination.shutdown_is_quiescent());
        let store = self.authority.media.store_identity();
        let session_identity = self.authority.session.identity();
        let recovery_effects = self.authority.media.recovery_effect_count();
        let AdmittedPlatformAuthority { media, session, .. } = self.authority;
        drop(media);
        session.publication_indeterminate();
        PhysicalRecoveryOutcome::PublicationIndeterminate(
            PhysicalRecoveryPublicationIndeterminate::new(
                store,
                session_identity,
                counters,
                PhysicalRecoveryPublicationSettlementLedger::new(settlement),
                self.root_protocol_denials,
                self.root_protocol_counters,
                recovery_effects,
            )
            .with_integrity_observations(self.integrity.into_observations())
            .with_integrity_trace(self.integrity_trace),
        )
    }
}
