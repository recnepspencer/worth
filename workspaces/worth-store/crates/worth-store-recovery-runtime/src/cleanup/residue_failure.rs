//! Outcomes of residue cleanup that did not settle, after recovery's own
//! publication is namespace durable.

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlock, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate,
};
use crate::progression::ReopenedPhysicalRecovery;

/// Residue evidence refused before any removal: damaged staging or obligation
/// listings, or a removal refused before its effect. The store is blocked for
/// inspection with the evidence named, not reported as a stuck publication.
pub(super) fn damaged(reopened: ReopenedPhysicalRecovery) -> PhysicalRecoveryOutcome {
    let ReopenedPhysicalRecovery {
        state,
        publication_counters,
        publication_settlement,
        ..
    } = reopened;
    let store = state.authority.media.store_identity();
    let session_identity = state.authority.session.identity();
    let recovery_effects = state.authority.media.recovery_effect_count();
    let _quiescent = state.coordination.into_owner().shutdown_is_quiescent();
    let AdmittedPlatformAuthority { media, session, .. } = state.authority;
    drop(media);
    session.block();
    PhysicalRecoveryOutcome::Blocked(PhysicalRecoveryBlock::new(
        crate::entry::PhysicalRecoveryBlockCause::Damage(
            PhysicalRecoveryBlockKind::MediaObservation,
        ),
        store,
        session_identity,
        PhysicalRecoveryBlockEvidence {
            counters: state.discovery_counters,
            planning_counters: Some(state.planning_counters),
            root_protocol_counters: Some(state.root_protocol_counters),
            source_denials: state.root_protocol_denials,
            integrity_observations: state.integrity.into_observations(),
            staging_counters: Some(state.staging_counters),
            staging_settlements: Some(state.staging_settlements),
            publication_counters: Some(publication_counters),
            publication_settlements: Some(publication_settlement),
            integrity_trace: state.integrity_trace,
            ..PhysicalRecoveryBlockEvidence::default()
        },
        recovery_effects,
    ))
}

/// A removal that started but did not settle leaves recovery's publication
/// durable and the residue namespace unproven.
pub(super) fn indeterminate(reopened: ReopenedPhysicalRecovery) -> PhysicalRecoveryOutcome {
    let ReopenedPhysicalRecovery {
        state,
        publication_counters,
        publication_settlement,
        ..
    } = reopened;
    let store = state.authority.media.store_identity();
    let session_identity = state.authority.session.identity();
    let recovery_effects = state.authority.media.recovery_effect_count();
    let _quiescent = state.coordination.into_owner().shutdown_is_quiescent();
    let AdmittedPlatformAuthority { media, session, .. } = state.authority;
    drop(media);
    session.publication_indeterminate();
    PhysicalRecoveryOutcome::PublicationIndeterminate(
        PhysicalRecoveryPublicationIndeterminate::new(
            store,
            session_identity,
            publication_counters,
            publication_settlement,
            state.root_protocol_denials,
            state.root_protocol_counters,
            recovery_effects,
        )
        .with_integrity_trace(state.integrity_trace)
        .with_integrity_observations(state.integrity.into_observations()),
    )
}
