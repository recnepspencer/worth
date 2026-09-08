use super::projection_admission::{admit_projection, IntegrityAdmittedRecoveryProjection};
use super::projection_materialization::{projected_record_bytes, validate_extent_closure};
use super::*;
use worth_store_physical_format::RecordArtifactFile;

pub(super) fn validate_projection_semantics(
    records: &[PhysicalRedoRecord],
    projection: &PersistedPhysicalRecoveryProjection,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<Box<[super::projection_admission::AdmittedInlineFrame]>, PhysicalRedoPlanningDenial> {
    let admitted = admit_projection(projection, store, format)?;
    validate_root_state(projection)?;
    if records.len() != projection.record_identities().len() {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    for (record, identity) in records.iter().zip(projection.record_identities()) {
        let placements = projection
            .placements()
            .iter()
            .filter(|placement| placement.record() == *identity)
            .collect::<Vec<_>>();
        if placements.len() != 1
            || placements[0].payload_bytes() != record.bytes().len() as u64
            || projected_record_bytes(*identity, *placements[0], &admitted)? != record.bytes()
        {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    }
    for (frame_index, frame) in projection.frames().iter().enumerate() {
        match frame.subject() {
            PersistedPhysicalDataFrameSubject::InlinePage(page) => {
                let descriptors = admitted
                    .inline_frames()
                    .iter()
                    .find(|value| value.frame_index == frame_index)
                    .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
                let RecordArtifactFile::Segment {
                    segment: artifact_segment,
                    generation: artifact_generation,
                } = frame.coordinate().artifact()
                else {
                    return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
                };
                for (descriptor, _) in &descriptors.records {
                    let exact = projection
                        .placements()
                        .iter()
                        .any(|placement| match placement {
                            CurrentPhysicalRecordPlacement::Inline(value) => {
                                value.record() == descriptor.record()
                                    && value.segment() == page.segment_id()
                                    && value.segment().get() == artifact_segment
                                    && value.segment_generation() == artifact_generation
                                    && value.page() == page.page_id()
                                    && value.page_generation() == page.generation().get()
                                    && value.slot() == descriptor.slot()
                                    && value.slot_generation() == descriptor.slot_generation()
                                    && value.payload_bytes() == descriptor.payload_bytes()
                            }
                            CurrentPhysicalRecordPlacement::Extent(_) => false,
                        });
                    if !exact {
                        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
                    }
                }
                let coordinate = frame.coordinate();
                let exact_update = projection.segment_updates().iter().any(|update| {
                    update.page_cell() == page
                        && update.data_generation() == artifact_generation
                        && u64::from(update.frame_index()) * u64::from(coordinate.length())
                            == coordinate.offset()
                });
                if !exact_update {
                    return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
                }
            }
            PersistedPhysicalDataFrameSubject::ExtentChunk(_) => {}
        }
    }
    validate_bidirectional_closure(projection, &admitted)?;
    validate_resulting_lsns(records, projection, &admitted)?;
    Ok(admitted.into_inline_frames())
}

fn validate_bidirectional_closure(
    projection: &PersistedPhysicalRecoveryProjection,
    admitted: &IntegrityAdmittedRecoveryProjection<'_>,
) -> Result<(), PhysicalRedoPlanningDenial> {
    validate_segment_frame_cardinality(projection)?;
    for placement in projection.placements() {
        match placement {
            CurrentPhysicalRecordPlacement::Inline(value) => {
                let matching = projection
                    .frames()
                    .iter()
                    .filter(|frame| {
                        frame.subject()
                            == PersistedPhysicalDataFrameSubject::InlinePage(value.page_cell())
                    })
                    .filter_map(|frame| {
                        projection
                            .frames()
                            .iter()
                            .position(|candidate| core::ptr::eq(candidate, frame))
                    })
                    .filter_map(|index| {
                        admitted
                            .inline_frames()
                            .iter()
                            .find(|value| value.frame_index == index)
                    })
                    .flat_map(|frame| frame.records.iter())
                    .filter(|(descriptor, _)| *descriptor == *value)
                    .count();
                if matching != 1 {
                    return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
                }
            }
            CurrentPhysicalRecordPlacement::Extent(value) => {
                validate_extent_closure(*value, admitted)?;
            }
        }
    }
    for update in projection.segment_updates() {
        let matching = projection
            .frames()
            .iter()
            .filter(|frame| {
                frame.subject() == PersistedPhysicalDataFrameSubject::InlinePage(update.page_cell())
                    && frame.coordinate().offset()
                        == u64::from(update.frame_index()) * u64::from(frame.coordinate().length())
            })
            .count();
        if matching != 1 {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    }
    for allocation in projection.root_state().inline_allocations() {
        if !projection.placements().iter().any(|placement| {
            matches!(placement, CurrentPhysicalRecordPlacement::Inline(value)
                if value.segment_cell() == allocation.segment()
                    && value.segment_page_capacity() == allocation.page_capacity())
        }) {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    }
    Ok(())
}

fn validate_segment_frame_cardinality(
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let mut exact_counts = BTreeMap::<(u64, u64), u32>::new();
    for frame in projection.frames() {
        let PersistedPhysicalDataFrameSubject::InlinePage(_) = frame.subject() else {
            continue;
        };
        let count = exact_counts
            .entry(match frame.coordinate().artifact() {
                RecordArtifactFile::Segment {
                    segment,
                    generation,
                } => (segment, generation),
                _ => return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
            })
            .or_default();
        *count = count
            .checked_add(1)
            .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    }
    for update in projection.segment_updates() {
        let key = (
            update.page_cell().segment_id().get(),
            update.data_generation(),
        );
        if exact_counts.get(&key).copied() != Some(update.data_page_count()) {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    }
    Ok(())
}

fn validate_root_state(
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let root = projection.root_state();
    for placement in projection.placements() {
        if let CurrentPhysicalRecordPlacement::Inline(value) = placement {
            let exact = root.inline_allocations().iter().any(|allocation| {
                allocation.segment() == value.segment_cell()
                    && allocation.page_capacity() == value.segment_page_capacity()
                    && allocation.used_pages() != 0
            });
            if !exact {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
        }
    }
    let tail = projection
        .placements()
        .iter()
        .filter_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Inline(value) => Some(*value),
            CurrentPhysicalRecordPlacement::Extent(_) => None,
        })
        .max_by_key(|value| {
            (
                value.segment().get(),
                value.page().get(),
                value.slot().get(),
            )
        });
    if root.last_inline_record() != tail.map(|value| value.record())
        || root.last_inline_segment() != tail.map(|value| value.segment_cell())
    {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    Ok(())
}

fn validate_resulting_lsns(
    records: &[PhysicalRedoRecord],
    projection: &PersistedPhysicalRecoveryProjection,
    admitted: &IntegrityAdmittedRecoveryProjection<'_>,
) -> Result<(), PhysicalRedoPlanningDenial> {
    for (index, frame) in projection.frames().iter().enumerate() {
        let matching = records
            .iter()
            .filter(|record| {
                record.targets().iter().any(|target| {
                    target_identity_matches_subject(target.identity(), frame.subject())
                })
            })
            .map(|record| record.lsn().get())
            .max()
            .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
        let found = match frame.subject() {
            PersistedPhysicalDataFrameSubject::InlinePage(_) => admitted
                .inline_frames()
                .iter()
                .find(|value| value.frame_index == index)
                .map(|value| value.page_lsn),
            PersistedPhysicalDataFrameSubject::ExtentChunk(_) => admitted
                .extent_chunks()
                .iter()
                .find(|value| value.frame_index == index)
                .map(|value| value.page_lsn),
        };
        if found
            .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?
            .get()
            != matching
        {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    }
    Ok(())
}

fn target_identity_matches_subject(
    identity: PhysicalRedoTargetIdentity,
    subject: PersistedPhysicalDataFrameSubject,
) -> bool {
    match (identity, subject) {
        (
            PhysicalRedoTargetIdentity::InlinePage {
                segment,
                page,
                generation,
            },
            PersistedPhysicalDataFrameSubject::InlinePage(value),
        ) => {
            (segment, page, generation)
                == (
                    value.segment_id().get(),
                    value.page_id().get(),
                    value.generation().get(),
                )
        }
        (
            PhysicalRedoTargetIdentity::ExtentChunk {
                extent,
                generation,
                chunk,
            },
            PersistedPhysicalDataFrameSubject::ExtentChunk(value),
        ) => {
            (extent, generation, chunk)
                == (
                    value.extent_cell().extent_id().get(),
                    value.extent_cell().generation().get(),
                    value.ordinal(),
                )
        }
        _ => false,
    }
}
