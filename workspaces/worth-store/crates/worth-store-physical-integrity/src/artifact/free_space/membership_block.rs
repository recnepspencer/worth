use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, maximum_current_root_entries, FreeSpaceBlockReference,
    FreeSpaceMembershipBlockDecodeLimits, FreeSpaceMembershipBlockScopeIdentity,
    PhysicalFreeSpaceMembershipBlock, PhysicalRecordFormatDeclaration,
};

use crate::artifact::durable_frame_rejection::{damaged, field_damage, input_length, wrong_scope};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedFreeSpaceMembershipBlock, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::membership_rejection::{
    free_space_membership_denial, membership_body_range, MembershipField,
};

#[derive(Debug)]
pub enum FreeSpaceMembershipBlockIntegrityValidation<'media> {
    Intact(IntegrityValidatedFreeSpaceMembershipBlock<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_free_space_membership_block<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    FreeSpaceMembershipBlockIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let (block, record_format) = match decode_bounded_membership(artifact, scope) {
        Ok(decoded) => decoded,
        Err(rejection) => return rejected(rejection, byte_count),
    };
    let expected = scope
        .free_space_membership_block_identity()
        .expect("free-space membership family scope carries its identity");
    if let Err(rejection) = validate_membership_format(scope, record_format) {
        return rejected(rejection, byte_count);
    }
    if let Err(rejection) = validate_membership_identity(scope, expected, &block) {
        return rejected(rejection, byte_count);
    }
    let complete_checksum = durable_artifact_checksum(artifact.bytes());
    if let Err(rejection) = validate_membership_reference(
        scope,
        expected.reference(),
        &block,
        artifact.bytes(),
        complete_checksum,
    ) {
        return rejected(rejection, byte_count);
    }
    let validated = IntegrityValidatedFreeSpaceMembershipBlock::new(
        scope,
        block,
        record_format,
        complete_checksum,
        artifact,
    )
    .expect("validated free-space membership block satisfies the sealed-view contract");
    (
        FreeSpaceMembershipBlockIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
            byte_count,
        ),
    )
}

fn decode_bounded_membership<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> Result<
    (
        PhysicalFreeSpaceMembershipBlock,
        PhysicalRecordFormatDeclaration,
    ),
    PhysicalIntegrityRejection,
> {
    let capacity = maximum_current_root_entries(scope.record_format());
    let limits = FreeSpaceMembershipBlockDecodeLimits {
        leaf_entries: u64::from(capacity),
        branch_children: u64::from(capacity),
    };
    PhysicalFreeSpaceMembershipBlock::decode_bounded(artifact.bytes(), capacity, limits)
        .map_err(|denial| free_space_membership_denial(scope, artifact.bytes(), denial))
}

fn validate_membership_format(
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
) -> Result<(), PhysicalIntegrityRejection> {
    if record_format == scope.record_format() {
        return Ok(());
    }
    Err(field_damage(
        scope,
        PhysicalDamageCause::FormatMismatch,
        MembershipField::FORMAT,
        PhysicalFormatField::FormatDeclaration,
        PhysicalBlastRadius::CompleteArtifact,
    ))
}

fn validate_membership_identity(
    scope: PhysicalArtifactScope,
    expected: FreeSpaceMembershipBlockScopeIdentity,
    block: &PhysicalFreeSpaceMembershipBlock,
) -> Result<(), PhysicalIntegrityRejection> {
    if block.tree_identity() != expected.tree().get() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            MembershipField::TREE_IDENTITY,
            PhysicalFormatField::TreeIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    let reference = expected.reference();
    if block.generation() != reference.generation() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            MembershipField::GENERATION,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if block.block() != reference.block() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            MembershipField::ENVELOPE_IDENTITY,
            PhysicalFormatField::BlockIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if block.level() != reference.level() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            MembershipField::LEVEL,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    Ok(())
}

fn validate_membership_reference(
    scope: PhysicalArtifactScope,
    expected: FreeSpaceBlockReference,
    block: &PhysicalFreeSpaceMembershipBlock,
    bytes: &[u8],
    complete_checksum: u32,
) -> Result<(), PhysicalIntegrityRejection> {
    let actual = block.reference(complete_checksum);
    if actual.first() != expected.first() || actual.last() != expected.last() {
        return Err(damaged(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            membership_body_range(scope, bytes),
            Some(PhysicalFormatField::MembershipRange),
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if complete_checksum != expected.checksum() {
        return Err(damaged(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::CompleteChildChecksum),
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    Ok(())
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    FreeSpaceMembershipBlockIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        FreeSpaceMembershipBlockIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
            byte_count,
            rejection,
        ),
    )
}
