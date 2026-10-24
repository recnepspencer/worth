use worth_store::physical_runtime::RecoveryStagingResidueOutcome;

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate,
};
use crate::progression::ReopenedPhysicalRecovery;

/// The bound on stale candidates one recovery removes. A killed runtime leaves
/// at most one publication attempt's three candidates per crash; a larger
/// population is evidence for inspection, not residue.
const MAXIMUM_STALE_CANDIDATES: usize = 48;

/// Removes root-protocol candidates a killed runtime left in record staging,
/// after recovery's own publication is namespace durable.
pub(super) fn prepare(reopened: &ReopenedPhysicalRecovery) -> RecoveryStagingResidueOutcome {
    reopened
        .state
        .coordination
        .owner()
        .remove_record_staging_residue(&reopened.state.authority.media, MAXIMUM_STALE_CANDIDATES)
}

/// A removal that did not settle leaves recovery's publication durable but the
/// staging namespace unproven, so the session reports an indeterminate
/// publication rather than a serving-ready store.
pub(super) fn failure(reopened: ReopenedPhysicalRecovery) -> PhysicalRecoveryOutcome {
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
