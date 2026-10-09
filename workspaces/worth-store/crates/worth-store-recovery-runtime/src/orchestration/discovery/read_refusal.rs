//! What a refused or failed read says to discovery: a limit with recovery's
//! own counts, damage the caller words for the artifact it read, or a media
//! failure that blocks.

use worth_store::physical_runtime::{
    ArtifactDamage, ExceededFilesystemObservationBound, FilesystemObservationBound, GrantOverrun,
    RecoveryDiscoveryFailure,
};

use super::DiscoveryFailure;
use crate::entry::{
    PhysicalRecoveryBlockKind as PhysicalRecoveryBlock, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryLimitDimension, PhysicalRecoveryMediaObservationFailure,
    PhysicalRecoverySourceDenial,
};
use crate::orchestration::recovery_budget::{RecoveryAllowance, RecoveryReadBudget};

/// An artifact longer than the ceiling its fact declares. Each reader words
/// this damage for the artifact it read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::orchestration) struct OversizedArtifact;

/// What a whole read met that was no grant's. `Ok` is an artifact longer than
/// its own ceiling: damage, which the caller words for the artifact it read.
/// `Err` blocks discovery.
pub(in crate::orchestration) fn unread(
    failure: RecoveryDiscoveryFailure,
    limits: &PhysicalRecoveryLimitDeclaration,
) -> Result<OversizedArtifact, DiscoveryFailure> {
    match failure {
        RecoveryDiscoveryFailure::Limit(past) => Err(observation_refused(past, limits)),
        RecoveryDiscoveryFailure::Damage(damage) => damaged_read(damage),
    }
}

/// A read's grant passed, stated by the budget that granted it.
pub(in crate::orchestration) fn past_grant(
    budget: &RecoveryReadBudget,
    overrun: GrantOverrun<PhysicalRecoveryLimitDimension>,
) -> DiscoveryFailure {
    budget.refuse(overrun).map_or_else(
        || DiscoveryFailure::from(PhysicalRecoveryBlock::MediaObservation),
        |limit| DiscoveryFailure::limit(PhysicalRecoveryBlock::MediaObservation, limit),
    )
}

/// The observation's own bound stopped a read. Discovery's reader was handed
/// all of recovery's observation bytes, so its counts are recovery's.
pub(in crate::orchestration) fn observation_refused(
    past: ExceededFilesystemObservationBound,
    limits: &PhysicalRecoveryLimitDeclaration,
) -> DiscoveryFailure {
    match past.dimension() {
        FilesystemObservationBound::ObservationBytes => refused_beside(
            RecoveryAllowance::declared(limits, PhysicalRecoveryLimitDimension::ObservationBytes),
            past.observed(),
            past.admitted(),
            PhysicalRecoveryBlock::MediaObservation,
        ),
        // Discovery's main reader counts no reads or entries: each count
        // limit counts its own before the read, so a refused count is past
        // every count, and no limit can state it. A WAL listing counts its
        // entries, and its caller reads them as WAL segments first.
        FilesystemObservationBound::Reads | FilesystemObservationBound::Entries => {
            DiscoveryFailure::from(PhysicalRecoveryBlock::MediaObservation)
        }
    }
}

/// What more budget cannot fix. `Ok` is an artifact longer than its own
/// ceiling, which the caller words for the artifact it read.
fn damaged_read(damage: ArtifactDamage) -> Result<OversizedArtifact, DiscoveryFailure> {
    match damage {
        ArtifactDamage::PastCeiling { .. } => Ok(OversizedArtifact),
        ArtifactDamage::Media { artifact, failure } => Err(media_observation(
            artifact,
            PhysicalRecoveryMediaObservationFailure::Backend {
                kind: failure.kind(),
                io_kind: failure.io_kind(),
            },
        )),
        ArtifactDamage::InvalidAddress { artifact } => Err(media_observation(
            artifact,
            PhysicalRecoveryMediaObservationFailure::InvalidAddress,
        )),
        // A count past every count is no limit and names no artifact.
        ArtifactDamage::CountOverflow(_) => Err(DiscoveryFailure::from(
            PhysicalRecoveryBlock::MediaObservation,
        )),
    }
}

/// `observed` refused by the `remaining` a caller was handed of the `whole`
/// allowance: the rest was spent before it. `phase` ran out of the whole;
/// counts that do not cross it name no limit, and `phase` failed.
pub(in crate::orchestration) fn refused_beside(
    whole: RecoveryAllowance,
    observed: u64,
    remaining: u64,
    phase: PhysicalRecoveryBlock,
) -> DiscoveryFailure {
    whole.beside(observed, remaining).map_or_else(
        || DiscoveryFailure::from(phase),
        |limit| DiscoveryFailure::limit(phase, limit),
    )
}

/// A read that failed for the media, not for any count.
fn media_observation(
    artifact: worth_store::physical_runtime::RecoveryDiscoveryArtifact,
    failure: PhysicalRecoveryMediaObservationFailure,
) -> DiscoveryFailure {
    let mut blocked = DiscoveryFailure::from(PhysicalRecoveryBlock::MediaObservation);
    blocked
        .source_denials
        .push(PhysicalRecoverySourceDenial::MediaObservation { artifact, failure });
    blocked
}
