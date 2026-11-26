//! `CHK24` record-identity pages of one complete capture.
//!
//! An allocator unchanged since it reopened carries its checkpoint pages:
//! as carried members when the destination already holds them, otherwise
//! by copying their exact bytes. A changed allocator encodes new pages.
use crate::durability::data::*;
use contracts::durability::data::native_checkpoint_capture_denial::{
    array_capture_stop, capture_stop, RelationalNativeCheckpointCaptureDenial as Denial,
};
use worth_execution::{
    ExecutionImmutableBytes, ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::paged_base::FrozenRecordIdentity;
use worth_relational_storage::internal_port::storage::data::record_identity_pages::{
    encode_record_identity, PagedRecordIdentity,
};
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers;

pub(super) fn capture(
    identity: &FrozenRecordIdentity,
    destination: &[RelationalDurableMemberLocator],
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<PagedRecordIdentity, Denial> {
    let (paged, pages) = match identity {
        FrozenRecordIdentity::Resident(state) => {
            let mut sink = CollectedMembers::default();
            let paged = encode_record_identity(state, &mut sink).map_err(|cause| {
                capture_stop(
                    DurabilityError::new(
                        RecoveryFailureClass::CorruptCheckpoint,
                        format!("record identity directory: {cause:?}"),
                    )
                    .into(),
                )
            })?;
            let mut pages = Vec::with_capacity(sink.members.len());
            for (locator, bytes) in sink.members {
                contracts::execution::request_custody::check_request(request)
                    .map_err(Denial::Request)?;
                pages.push(RelationalCheckpointMember::New {
                    locator,
                    bytes: request_bytes(&bytes, request)?,
                });
            }
            (paged, pages)
        }
        FrozenRecordIdentity::Carried(base) => {
            let mut held = ExecutionOrderedMap::default();
            for locator in destination {
                held.insert_cloned_native_for_request(locator, &(), *request)
                    .map_err(|cause| {
                        capture_stop(contracts::durability::data::recovery_stop::map_stop(cause))
                    })?;
            }
            let mut pages = Vec::with_capacity(base.paged().pages.len());
            for &locator in &base.paged().pages {
                contracts::execution::request_custody::check_request(request)
                    .map_err(Denial::Request)?;
                pages.push(if held.contains_key(&locator) {
                    RelationalCheckpointMember::Carried { locator }
                } else {
                    let bytes = base.page_bytes(locator).map_err(|cause| {
                        capture_stop(
                            DurabilityError::new(
                                RecoveryFailureClass::ColdStorageUnavailable,
                                format!("record identity page unavailable: {cause:?}"),
                            )
                            .into(),
                        )
                    })?;
                    RelationalCheckpointMember::New {
                        locator,
                        bytes: request_bytes(&bytes, request)?,
                    }
                });
            }
            (base.paged().clone(), pages)
        }
    };
    for member in pages {
        let locator = member.locator();
        if members.get(&locator).is_none() {
            members
                .insert_cloned_native_for_request(&locator, &member, *request)
                .map_err(|cause| {
                    capture_stop(contracts::durability::data::recovery_stop::map_stop(cause))
                })?;
        }
    }
    Ok(paged)
}

fn request_bytes(
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Denial> {
    let mut buffer = ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
        .map_err(array_capture_stop)?;
    buffer
        .extend_from_slice(bytes)
        .map_err(array_capture_stop)?;
    buffer.seal_immutable_bytes().map_err(array_capture_stop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, sync::Arc};
    use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::paged_base::PagedIdentityBase;
    use worth_relational_storage::internal_port::storage::data::record_identity_image::{
        DurableRecordGenerationClass::Entity, DurableRecordGenerationHighWater,
        DurableRecordSlotFrontier,
    };
    use worth_relational_storage::internal_port::storage::data::RelationalStorageUnavailableCause as Cause;
    use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSource;
    use worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant;

    struct MapSource(BTreeMap<RelationalDurableMemberLocator, Vec<u8>>);
    impl ChunkMemberSource for MapSource {
        fn read_range(
            &self,
            m: RelationalDurableMemberLocator,
            off: u64,
            dst: &mut [u8],
        ) -> Result<usize, Cause> {
            let tail = self
                .0
                .get(&m)
                .ok_or(Cause::MissingMember)?
                .get(off as usize..)
                .unwrap_or(&[]);
            let n = tail.len().min(dst.len());
            dst[..n].copy_from_slice(&tail[..n]);
            Ok(n)
        }
    }

    #[test]
    fn carried_capture_carries_present_pages_and_copies_missing_pages_into_request_memory() {
        let host = worth_harness::facade::execution_fixture::serial_request();
        let request = ExecutionRequest::serial(&host);
        let p0 = contracts::identity::data::primitives::PartitionId(0);
        let state = DurableRecordIdentityState::current_with_memory(
            vec![DurableRecordGenerationHighWater {
                class: Entity,
                partition_id: p0,
                slot: 0,
                generation: 1,
            }],
            Vec::new(),
            vec![DurableRecordSlotFrontier {
                class: Entity,
                partition_id: p0,
                next_slot: 1,
            }],
            Vec::new(),
            None,
        );
        let mut sink = CollectedMembers::default();
        let paged = encode_record_identity(&state, &mut sink).unwrap();
        let source = Arc::new(MapSource(sink.members.into_iter().collect()));
        let accountant =
            WorkingSetAccountant::new(contracts::config::data::residency_budget::ResidencyBudget {
                head_working_set_bytes: 1024 * 1024,
                historical_working_set_bytes: 1024 * 1024,
                max_in_flight_fill_bytes: 256 * 1024,
                streaming_window_bytes: 64 * 1024,
            });
        let base = Arc::new(PagedIdentityBase::open(paged.clone(), accountant, source));
        let (carried, copied) = (paged.pages[0], paged.pages[1]);
        let mut members = ExecutionOrderedMap::default();
        assert_eq!(
            capture(
                &FrozenRecordIdentity::Carried(base),
                &[carried],
                &mut members,
                &request
            )
            .unwrap(),
            paged
        );
        assert!(
            matches!(members.get(&carried), Some(RelationalCheckpointMember::Carried { locator }) if *locator == carried)
        );
        let Some(RelationalCheckpointMember::New { bytes, .. }) = members.get(&copied) else {
            panic!("expected copied New");
        };
        assert!(!bytes.is_external_backing() && bytes.charged_payload_bytes().is_some());
    }
}
