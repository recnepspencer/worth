use super::{
    target::SelectedRecordScrubBasis, PhysicalIntegrityScrubDeferral, PhysicalIntegrityScrubTarget,
};
use crate::physical_runtime::record_serving::{
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadDenial, RecordReadLimits,
    RecordReadWorkDenial, RecordStreamFailureKind,
};
use worth_store_physical_format::{BTreeNodeKind, BlobRecordV1};
use worth_store_physical_integrity::{
    validate_blob_record, validate_btree_node, BTreeNodeIntegrityValidation,
    BlobRecordIntegrityValidation, IndeterminatePhysicalIntegrityCause,
    IndeterminatePhysicalIntegrityPosture, PhysicalArtifactScope, PhysicalBlastRadius,
    PhysicalDamageCause, PhysicalDamageLocalization, PhysicalFormatField,
    PhysicalIntegrityObservationCounters, PhysicalIntegrityObservationOutcome,
    PhysicalIntegrityRejection, UnknownPhysicalIntegrityCause, UnknownPhysicalIntegrityPosture,
    UntrustedPhysicalArtifact,
};

pub(super) fn inspect(
    reader: &PhysicalRecordReader,
    target: PhysicalIntegrityScrubTarget,
    destination: &mut [u8],
    late: bool,
) -> Result<
    (
        PhysicalArtifactScope,
        PhysicalIntegrityObservationOutcome,
        PhysicalIntegrityObservationCounters,
        u64,
    ),
    PhysicalIntegrityScrubDeferral,
> {
    let ceiling = target.scope();
    let Some(record) = (match target.source() {
        super::PhysicalIntegrityScrubSource::SelectedRecord(record) => Some(record),
        _ => None,
    }) else {
        unreachable!("selected inspector receives selected target");
    };
    if late || target.issued_under() != Some(reader.protected_root()) {
        return Ok(indeterminate(
            ceiling,
            IndeterminatePhysicalIntegrityCause::SourceChangedDuringInspection,
            0,
        ));
    }
    let limit = RecordByteLimit::new(target.declared_bytes()).expect("nonempty bounded target");
    let mut session = match reader.open(
        PhysicalRecordId::from_persisted(record),
        RecordReadLimits::new(limit),
    ) {
        Ok(session) => session,
        Err(error) => {
            return match error.denial() {
                RecordReadDenial::PhysicalWork(
                    RecordReadWorkDenial::SchedulerReservationRejected
                    | RecordReadWorkDenial::SchedulerRejected,
                )
                | RecordReadDenial::PhysicalPressure
                | RecordReadDenial::ResidencyUnavailable(_) => Err(
                    PhysicalIntegrityScrubDeferral::SelectedRecordRead(error.denial()),
                ),
                RecordReadDenial::RecordNotFound => Ok(unknown(
                    ceiling,
                    UnknownPhysicalIntegrityCause::ExpectedArtifactAbsent,
                    0,
                )),
                RecordReadDenial::ArtifactDamaged | RecordReadDenial::FormatMismatch => {
                    Ok(unknown(
                        ceiling,
                        UnknownPhysicalIntegrityCause::ExpectedScopeUnavailable,
                        0,
                    ))
                }
                _ => Ok(indeterminate(
                    ceiling,
                    IndeterminatePhysicalIntegrityCause::StableRangeNotProven,
                    0,
                )),
            }
        }
    };
    let expected = session.observation().bytes_requested();
    let Some(scope) = target.selected_scope(expected) else {
        return Ok(indeterminate(
            ceiling,
            IndeterminatePhysicalIntegrityCause::ObservationBoundExhausted,
            0,
        ));
    };
    let mut used = 0_usize;
    while used < expected as usize {
        match session.read_next(&mut destination[used..expected as usize]) {
            Ok(0) => {
                return Ok(indeterminate(
                    scope,
                    IndeterminatePhysicalIntegrityCause::StableRangeNotProven,
                    used as u64,
                ))
            }
            Ok(count) => used += count,
            Err(failure) => {
                return match failure.kind() {
                    RecordStreamFailureKind::PhysicalPressure
                    | RecordStreamFailureKind::SchedulerUnavailable
                    | RecordStreamFailureKind::ResidencyUnavailable(_)
                        if used == 0 =>
                    {
                        Err(PhysicalIntegrityScrubDeferral::SelectedRecordStream(
                            failure.kind(),
                        ))
                    }
                    RecordStreamFailureKind::ArtifactDamaged
                    | RecordStreamFailureKind::SelectedDataFrameChecksumDamaged
                    | RecordStreamFailureKind::FormatMismatch => Ok(unknown(
                        scope,
                        UnknownPhysicalIntegrityCause::ExpectedScopeUnavailable,
                        used as u64,
                    )),
                    _ => Ok(indeterminate(
                        scope,
                        IndeterminatePhysicalIntegrityCause::StableRangeNotProven,
                        used as u64,
                    )),
                }
            }
        }
    }
    let bytes = &destination[..used];
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
    let (outcome, counters) = match target.basis().expect("selected target has issuer basis") {
        SelectedRecordScrubBasis::BTreeNode {
            family_code: _,
            key_bytes,
            leaf_value_bytes,
        } => {
            let (validation, counters) = validate_btree_node(artifact, scope);
            match validation {
                BTreeNodeIntegrityValidation::Intact(validated)
                    if validated.node().cells().iter().all(|cell| {
                        cell.key().len() == key_bytes
                            && (validated.node().kind() != BTreeNodeKind::Leaf
                                || cell
                                    .leaf_value()
                                    .is_some_and(|value| value.len() == leaf_value_bytes))
                    }) =>
                {
                    (PhysicalIntegrityObservationOutcome::Intact(scope), counters)
                }
                BTreeNodeIntegrityValidation::Intact(_) => {
                    let rejection =
                        PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
                            scope,
                            PhysicalDamageCause::MalformedStructure,
                            scope.byte_range(),
                            Some(PhysicalFormatField::Payload),
                            PhysicalBlastRadius::CanonicalFrame,
                        ));
                    (
                        PhysicalIntegrityObservationOutcome::Rejected(rejection),
                        PhysicalIntegrityObservationCounters::for_rejection(rejection, used as u64),
                    )
                }
                BTreeNodeIntegrityValidation::Rejected(rejection) => (
                    PhysicalIntegrityObservationOutcome::Rejected(rejection),
                    counters,
                ),
            }
        }
        basis => {
            let (validation, counters) = validate_blob_record(artifact, scope);
            match validation {
                BlobRecordIntegrityValidation::Intact(intact)
                    if matches_basis(intact.record(), basis) =>
                {
                    (PhysicalIntegrityObservationOutcome::Intact(scope), counters)
                }
                BlobRecordIntegrityValidation::Intact(_) => {
                    let rejection =
                        PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
                            scope,
                            PhysicalDamageCause::ArtifactIdentityMismatch,
                            scope.byte_range(),
                            None,
                            PhysicalBlastRadius::CanonicalFrame,
                        ));
                    (
                        PhysicalIntegrityObservationOutcome::Rejected(rejection),
                        PhysicalIntegrityObservationCounters::for_rejection(rejection, used as u64),
                    )
                }
                BlobRecordIntegrityValidation::Rejected(rejection) => (
                    PhysicalIntegrityObservationOutcome::Rejected(rejection),
                    counters,
                ),
            }
        }
    };
    Ok((scope, outcome, counters, used as u64))
}

fn matches_basis(record: &BlobRecordV1<'_>, basis: SelectedRecordScrubBasis) -> bool {
    match (record, basis) {
        (
            BlobRecordV1::Chunk(chunk),
            SelectedRecordScrubBasis::BlobChunk {
                session,
                ordinal,
                chunk_size,
                digest,
                covered_bytes,
            },
        ) => {
            chunk.occurrence().session() == session
                && chunk.occurrence().ordinal() == ordinal
                && chunk.chunk_size() == chunk_size
                && chunk.stored_digest() == digest
                && chunk.bytes().len() as u64 == covered_bytes
        }
        (
            BlobRecordV1::TreeNode(node),
            SelectedRecordScrubBasis::BlobTreeNode {
                session,
                level,
                index,
                digest,
                covered_bytes,
                root_digest,
            },
        ) => {
            node.occurrence().session() == session
                && node.occurrence().level() == level
                && node.occurrence().index() == index
                && node.covered_bytes() == covered_bytes
                && if root_digest {
                    node.frame_digest() == digest
                } else {
                    node.canonical_digest() == digest
                }
        }
        (
            BlobRecordV1::GenerationPublished(publication),
            SelectedRecordScrubBasis::BlobGenerationPublication { object, generation },
        ) => publication.object() == object && publication.generation() == generation,
        _ => false,
    }
}

fn unknown(
    scope: PhysicalArtifactScope,
    cause: UnknownPhysicalIntegrityCause,
    acquired: u64,
) -> (
    PhysicalArtifactScope,
    PhysicalIntegrityObservationOutcome,
    PhysicalIntegrityObservationCounters,
    u64,
) {
    (
        scope,
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(
            UnknownPhysicalIntegrityPosture::new(scope, cause),
        )),
        PhysicalIntegrityObservationCounters::empty(scope.artifact_family()),
        acquired,
    )
}

fn indeterminate(
    scope: PhysicalArtifactScope,
    cause: IndeterminatePhysicalIntegrityCause,
    acquired: u64,
) -> (
    PhysicalArtifactScope,
    PhysicalIntegrityObservationOutcome,
    PhysicalIntegrityObservationCounters,
    u64,
) {
    (
        scope,
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Indeterminate(
            IndeterminatePhysicalIntegrityPosture::new(scope, cause, None),
        )),
        PhysicalIntegrityObservationCounters::empty(scope.artifact_family()),
        acquired,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{
        decode_blob_record, BlobChunkFrameV1, BlobChunkOccurrenceV1,
    };

    #[test]
    fn chunk_parent_edge_must_match_valid_inner_frame() {
        let bytes = BlobChunkFrameV1::encode(
            BlobChunkOccurrenceV1::new([1; 16], [2; 16], 7).unwrap(),
            64 << 10,
            &[3; 128],
        )
        .unwrap();
        let record = decode_blob_record(&bytes).unwrap();
        let BlobRecordV1::Chunk(chunk) = &record else {
            panic!("encoded chunk");
        };
        let basis = |session, ordinal, digest, covered_bytes| SelectedRecordScrubBasis::BlobChunk {
            session,
            ordinal,
            chunk_size: 64 << 10,
            digest,
            covered_bytes,
        };
        let expected = basis([2; 16], 7, chunk.stored_digest(), 128);
        assert!(matches_basis(&record, expected));
        for wrong in [
            basis([2; 16], 8, chunk.stored_digest(), 128),
            basis([2; 16], 7, chunk.stored_digest(), 127),
            basis([2; 16], 7, [9; 32], 128),
            basis([4; 16], 7, chunk.stored_digest(), 128),
        ] {
            assert!(!matches_basis(&record, wrong));
        }
    }
}
