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
    NamespaceDurablePayload, NamespaceDurablePhysicalRecovery, NamespaceDurableState,
    RecoveryPublicationExpectation, RecoveryPublicationPlan, StagedPhysicalRecovery,
    StagedRecoveryState,
};

pub(crate) struct PublicationPayload {
    pub(crate) reopened_backing: crate::progression::ReopenedBacking,
    namespace_backing: crate::progression::NamespaceDurableBacking,
    authority: AdmittedPlatformAuthority,
    coordination: super::super::RecoveryCoordination,
    selection: crate::orchestration::ResidentSourceSelection,
    custody: crate::progression::PlanningCustody,
    verified_selected_tier_custody:
        Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>,
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

pub(super) struct PublicationState {
    backing: PublicationStateBacking,
}
mod storage;
pub(crate) use storage::PublicationStateBacking;

impl PublicationState {
    const fn data(&self) -> &PublicationPayload {
        self.backing.payload()
    }

    pub(super) fn from_staged(staged: StagedPhysicalRecovery) -> (Self, RecoveryPublicationPlan) {
        let StagedRecoveryState {
            reopened_backing,
            publication_backing,
            namespace_backing,
            authority,
            coordination,
            selection,
            custody,
            verified_selected_tier_custody,
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
        } = staged.into_state();
        (
            publication_backing.finish(PublicationPayload {
                reopened_backing,
                namespace_backing,
                authority,
                coordination,
                selection,
                custody,
                verified_selected_tier_custody,
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
            }),
            publication,
        )
    }

    pub(super) const fn media(
        &self,
    ) -> &worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia {
        &self.data().authority.media
    }

    /// Whether the dead runtime left obligations that only recovery's own
    /// root-protocol publication can settle. A damaged listing is reported by
    /// cleanup, which refuses it.
    pub(super) fn prior_effects_unproven(&self) -> bool {
        self.data()
            .coordination
            .owner()
            .prior_physical_work_obligations(
                &self.data().authority.media,
                crate::cleanup::MAXIMUM_PRIOR_OBLIGATIONS,
            )
            .is_ok_and(|obligations| crate::cleanup::requires_republication(&obligations))
    }

    pub(super) const fn planned_effects(&self) -> u64 {
        self.data().quiescence.publication_commands()
    }

    pub(super) fn execute(
        &self,
        command: PhysicalRecoveryPublicationCommand,
    ) -> PhysicalRecoveryPublicationCommandOutcome {
        self.data()
            .coordination
            .owner()
            .execute_publication_command(&self.data().authority.media, command)
    }

    pub(super) fn is_ready(&self) -> bool {
        self.data().coordination.is_ready()
    }

    pub(super) fn publish_preexisting(
        self,
        expectation: RecoveryPublicationExpectation,
    ) -> Result<NamespaceDurablePhysicalRecovery, PhysicalRecoveryOutcome> {
        let selected = self.data().selection.root().selected();
        if self.planned_effects() != 0
            || self.data().closed.artifact_count() != 0
            || self.data().closed.byte_count() != 0
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
        let data = self.backing.into_payload();
        data.namespace_backing.finish(NamespaceDurablePayload {
            reopened_backing: data.reopened_backing,
            state: NamespaceDurableState {
                authority: data.authority,
                coordination: data.coordination,
                selection: data.selection,
                custody: data.custody,
                verified_selected_tier_custody: data.verified_selected_tier_custody,
                discovery_counters: data.discovery,
                root_protocol_denials: data.root_protocol_denials,
                integrity: data.integrity,
                integrity_trace: data.integrity_trace,
                freshness: data.freshness,
                fates: data.fates,
                planning_counters: data.planning,
                root_protocol_counters: data.root_protocol_counters,
                base: data.base,
                quiescence: data.quiescence,
                closed: data.closed,
                staging_counters: data.staging,
                staging_settlements: data.staging_settlements,
            },
            expectation,
            publication_counters: counters,
            publication_settlement: PhysicalRecoveryPublicationSettlementLedger::new(settlement),
        })
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
        let data = self.backing.into_payload();
        assert!(data.coordination.shutdown_is_quiescent());
        let store = data.authority.media.store_identity();
        let session_identity = data.authority.session.identity();
        let recovery_effects = data.authority.media.recovery_effect_count();
        let AdmittedPlatformAuthority { media, session, .. } = data.authority;
        drop(media);
        session.block();
        PhysicalRecoveryOutcome::Blocked(Box::new(PhysicalRecoveryBlock::new(
            crate::entry::PhysicalRecoveryBlockCause::Damage(
                PhysicalRecoveryBlockKind::Publication,
            ),
            store,
            session_identity,
            PhysicalRecoveryBlockEvidence {
                counters: data.discovery,
                source_denials: data.root_protocol_denials,
                integrity_observations: data.integrity.into_observations(),
                integrity_trace: data.integrity_trace,
                planning_counters: Some(data.planning),
                root_protocol_counters: Some(data.root_protocol_counters),
                staging_counters: Some(data.staging),
                staging_settlements: Some(data.staging_settlements),
                publication_counters: Some(counters),
                publication_denial: denial,
                publication_settlements: settlement
                    .map(PhysicalRecoveryPublicationSettlementLedger::new),
                ..PhysicalRecoveryBlockEvidence::default()
            },
            recovery_effects,
        )))
    }

    pub(super) fn indeterminate(
        self,
        counters: PhysicalRecoveryPublicationCounters,
        settlement: PhysicalRecoveryPublicationSettlement,
    ) -> PhysicalRecoveryOutcome {
        let data = self.backing.into_payload();
        assert!(data.coordination.shutdown_is_quiescent());
        let store = data.authority.media.store_identity();
        let session_identity = data.authority.session.identity();
        let recovery_effects = data.authority.media.recovery_effect_count();
        let AdmittedPlatformAuthority { media, session, .. } = data.authority;
        drop(media);
        session.publication_indeterminate();
        PhysicalRecoveryOutcome::PublicationIndeterminate(Box::new(
            PhysicalRecoveryPublicationIndeterminate::new(
                store,
                session_identity,
                counters,
                PhysicalRecoveryPublicationSettlementLedger::new(settlement),
                data.root_protocol_denials,
                data.root_protocol_counters,
                recovery_effects,
            )
            .with_integrity_observations(data.integrity.into_observations())
            .with_integrity_trace(data.integrity_trace),
        ))
    }
}
#[cfg(test)]
#[path = "state_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
