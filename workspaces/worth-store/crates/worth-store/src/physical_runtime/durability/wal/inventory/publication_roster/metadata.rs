//! Grant-bounded decoding of one authenticated WAL publication member.

use super::*;

pub(super) fn decode_metadata(
    redo: &[u8],
    range: WalLsnRange,
    format: PhysicalRecordFormatDeclaration,
    available_bytes: u64,
) -> Result<ReopenedPublicationMemberMetadata, PhysicalWalOpenFailure> {
    let mut body = redo;
    let domain = field(&mut body)?;
    // The format decoder holds at most the input-sized record, frame and
    // manifest payload copies plus bounded, fixed-size entry vectors. Divide
    // the remaining recovery grant across the nine simultaneously possible
    // entry families, with both Vec and boxed conversion space. Every decoder
    // entry count is constrained below.
    let encoded_bytes = redo.len() as u64;
    let payload_copies = encoded_bytes
        .checked_mul(4)
        .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
    let entry_size = [
        std::mem::size_of::<worth_store_physical_format::CanonicalRedoWireRecord>(),
        std::mem::size_of::<worth_store_physical_format::CanonicalRedoTarget>(),
        std::mem::size_of::<worth_store_physical_format::PersistedPhysicalRecoveryFrame>(),
        std::mem::size_of::<worth_store_physical_format::PersistedPhysicalRecoveryManifest>(),
        std::mem::size_of::<worth_store_physical_format::PersistedRecordIdentity>(),
        std::mem::size_of::<worth_store_physical_format::CurrentPhysicalRecordPlacement>(),
        std::mem::size_of::<worth_store_physical_format::RecordSegmentPageManifestEntry>(),
        std::mem::size_of::<worth_store_physical_format::PersistedInlineSegmentAllocation>(),
        std::mem::size_of::<usize>(),
    ]
    .into_iter()
    .max()
    .unwrap() as u64;
    let entry_envelope = entry_size
        .checked_mul(18)
        .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
    let available_entries =
        available_bytes.checked_sub(payload_copies).unwrap_or(0) / entry_envelope;
    let bounded = available_entries.min(encoded_bytes);
    // Rewrite v2 is a fixed descriptor with no decoded entry vectors or
    // candidate payload copy; only the retained roster member is charged.
    if bounded == 0 && domain != REWRITE_REDO_DOMAIN {
        return Err(PhysicalWalOpenFailure::ReopenAllocationLimitExceeded {
            admitted: available_bytes,
            required: payload_copies
                .checked_add(entry_envelope)
                .ok_or(PhysicalWalOpenFailure::CounterOverflow)?,
        });
    }
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: bounded,
        record_identities: bounded,
        placements: bounded,
        segment_updates: bounded,
        manifests: bounded,
        total_entries: bounded.saturating_mul(3),
        inline_allocations: bounded,
    };
    let (source_root_generation, successor_manifest_capacity, inserted_records) = if domain
        == CANONICAL_REDO_V3_DOMAIN
    {
        let (_, projection) = decode_canonical_redo_v3(
            redo,
            range.start().get(),
            range.end_exclusive().get(),
            bounded,
            None,
            limits,
            format,
        )
        .map_err(|denial| match denial {
            worth_store_physical_format::CanonicalRedoWireDenial::TargetLimit
            | worth_store_physical_format::CanonicalRedoWireDenial::ProjectionEntryLimit => {
                PhysicalWalOpenFailure::ReopenAllocationLimitExceeded {
                    admitted: available_bytes,
                    required: payload_copies
                        .saturating_add(entry_envelope.saturating_mul(bounded.saturating_add(1))),
                }
            }
            worth_store_physical_format::CanonicalRedoWireDenial::UnsupportedRecoveryProjectionVersion(version) => {
                PhysicalWalOpenFailure::UnsupportedRecoveryProjectionVersion(version)
            }
            _ => PhysicalWalOpenFailure::MemberPayloadRejected,
        })?;
        (
            projection.source_root_generation(),
            Some(projection.root_state().successor_manifest_capacity()),
            projection.record_identities().len() as u64,
        )
    } else if domain == REWRITE_REDO_DOMAIN {
        // This is a fixed-size descriptor; candidate_bytes is a physical
        // work bound, not a decoder allocation charged to this grant.
        let rewrite = PhysicalRewriteRedo::decode(redo, u64::MAX)
            .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
        if range.end_exclusive().get()
            != range
                .start()
                .get()
                .checked_add(1)
                .ok_or(PhysicalWalOpenFailure::MemberPayloadRejected)?
            || rewrite.page_lsn() != range.start().get()
            || rewrite.resulting_root_generation()
                != rewrite
                    .source_root_generation()
                    .checked_add(1)
                    .ok_or(PhysicalWalOpenFailure::MemberPayloadRejected)?
        {
            return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
        }
        (rewrite.source_root_generation(), None, 0)
    } else if domain == COPY_PUBLICATION_DOMAIN {
        let lsn = take_u64(&mut body)?;
        let encoded = field(&mut body)?;
        if lsn != range.start().get()
            || range.end_exclusive().get()
                != lsn
                    .checked_add(1)
                    .ok_or(PhysicalWalOpenFailure::MemberPayloadRejected)?
            || !body.is_empty()
        {
            return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
        }
        let projection = PersistedPhysicalRecoveryProjection::decode(encoded, limits, format)
            .map_err(|denial| match denial {
                worth_store_physical_format::PhysicalRecoveryProjectionDenial::EntryLimit => {
                    PhysicalWalOpenFailure::ReopenAllocationLimitExceeded {
                        admitted: available_bytes,
                        required: payload_copies.saturating_add(
                            entry_envelope.saturating_mul(bounded.saturating_add(1)),
                        ),
                    }
                }
                worth_store_physical_format::PhysicalRecoveryProjectionDenial::UnsupportedVersion(version) => {
                    PhysicalWalOpenFailure::UnsupportedRecoveryProjectionVersion(version)
                }
                _ => PhysicalWalOpenFailure::MemberPayloadRejected,
            })?;
        if !matches!(
            projection.payload(),
            worth_store_physical_format::PersistedPhysicalRecoveryPayload::SourceCopy(_)
        ) {
            return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
        }
        (
            projection.source_root_generation(),
            Some(projection.root_state().successor_manifest_capacity()),
            0,
        )
    } else {
        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
    };
    Ok(ReopenedPublicationMemberMetadata {
        source_root_generation,
        successor_manifest_capacity,
        inserted_records,
    })
}
