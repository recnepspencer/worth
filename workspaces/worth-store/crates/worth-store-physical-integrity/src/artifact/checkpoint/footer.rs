use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{durable_artifact_checksum, CheckpointStreamFooter};

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedCheckpointFooter, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::footer_basis::{CheckpointFooterExpectedBindings, CheckpointFooterValidationBasis};
use super::record_rejection::{checkpoint_record_denial, field_damage, CheckpointRecordFieldRange};

const STORE_IDENTITY_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(16, 16);
const SEQUENCE_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(32, 8);
const DIRTY_COUNT_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(40, 8);
const DIRTY_DIGEST_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(48, 32);
const COMPACTION_OFFSET_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(80, 8);
const COMPACTION_GENERATION_FIELD: CheckpointRecordFieldRange =
    CheckpointRecordFieldRange::new(88, 8);
const COMPACTION_WAL_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(96, 8);
const BINDING_COUNT_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(104, 8);
const BINDING_BYTES_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(112, 8);
const BINDING_DIGEST_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(120, 32);

#[derive(Debug)]
pub enum CheckpointFooterIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointFooter<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_checkpoint_footer<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
    basis: CheckpointFooterValidationBasis<'_, 'media>,
) -> (
    CheckpointFooterIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    validate_with_expected(artifact, scope, || basis.expected_bindings(scope))
}

pub(super) fn validate_with_expected<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
    expected: impl FnOnce() -> Result<CheckpointFooterExpectedBindings, PhysicalIntegrityRejection>,
) -> (
    CheckpointFooterIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    let family = PhysicalIntegrityArtifactFamily::CheckpointFooter;
    if scope.artifact_family() != family {
        return rejected_footer(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected_footer(rejection, byte_count);
    }
    let footer = match CheckpointStreamFooter::decode_record(artifact.bytes()) {
        Ok(footer) => footer,
        Err(denial) => {
            return rejected_footer(
                checkpoint_record_denial(scope, artifact.bytes(), denial),
                byte_count,
            )
        }
    };
    if let Some(rejection) = identity_mismatch(scope, footer) {
        return rejected_footer(rejection, byte_count);
    }
    let expected = match expected() {
        Ok(expected) => expected,
        Err(rejection) => return rejected_footer(rejection, byte_count),
    };
    if let Some(rejection) = binding_mismatch(scope, footer, expected) {
        return rejected_footer(rejection, byte_count);
    }
    let validated = IntegrityValidatedCheckpointFooter::new(
        scope,
        footer,
        durable_artifact_checksum(artifact.bytes()),
        artifact,
    )
    .expect("validated checkpoint footer satisfies its sealed-view contract");
    (
        CheckpointFooterIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(family, byte_count),
    )
}

pub(super) fn identity_mismatch(
    scope: PhysicalArtifactScope,
    footer: CheckpointStreamFooter,
) -> Option<PhysicalIntegrityRejection> {
    let expected = scope
        .checkpoint_identity()
        .expect("checkpoint-footer scope carries admitted identity");
    if footer.identity().store_identity() != expected.store_identity() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            STORE_IDENTITY_FIELD,
            PhysicalFormatField::StoreIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    (footer.identity() != expected).then(|| {
        field_damage(
            scope,
            PhysicalDamageCause::SequenceMismatch,
            SEQUENCE_FIELD,
            PhysicalFormatField::CheckpointIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        )
    })
}

fn binding_mismatch(
    scope: PhysicalArtifactScope,
    footer: CheckpointStreamFooter,
    expected: CheckpointFooterExpectedBindings,
) -> Option<PhysicalIntegrityRejection> {
    let mismatch = if footer.dirty_record_count() != expected.dirty.record_count() {
        Some(DIRTY_COUNT_FIELD)
    } else if footer.binding_record_count() != expected.bindings.record_count() {
        Some(BINDING_COUNT_FIELD)
    } else if footer.binding_record_bytes() != expected.bindings.encoded_bytes() {
        Some(BINDING_BYTES_FIELD)
    } else if footer.binding_compaction_header_offset() != expected.compaction_offset {
        Some(COMPACTION_OFFSET_FIELD)
    } else if footer.binding_compaction_generation() != expected.compaction_generation {
        Some(COMPACTION_GENERATION_FIELD)
    } else if footer.binding_wal_cutoff_lsn_exclusive() != expected.wal_cutoff_lsn_exclusive {
        Some(COMPACTION_WAL_FIELD)
    } else if footer.dirty_records_digest() != expected.dirty.digest() {
        Some(DIRTY_DIGEST_FIELD)
    } else if footer.binding_records_digest() != expected.bindings.digest() {
        Some(BINDING_DIGEST_FIELD)
    } else {
        None
    }?;
    let field = match mismatch {
        COMPACTION_GENERATION_FIELD => PhysicalFormatField::PhysicalGeneration,
        COMPACTION_WAL_FIELD => PhysicalFormatField::WalLsnRange,
        _ => PhysicalFormatField::CheckpointAggregate,
    };
    Some(field_damage(
        scope,
        PhysicalDamageCause::AggregateMismatch,
        mismatch,
        field,
        PhysicalBlastRadius::CompleteArtifact,
    ))
}

pub(super) fn rejected_footer<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    CheckpointFooterIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        CheckpointFooterIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::CheckpointFooter,
            byte_count,
            rejection,
        ),
    )
}
