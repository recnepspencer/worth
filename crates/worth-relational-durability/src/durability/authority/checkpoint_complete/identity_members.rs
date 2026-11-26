//! `CHK24` record-identity pages of one complete capture.
//!
//! An allocator unchanged since it reopened carries its checkpoint pages:
//! as carried members when the destination already holds them, otherwise
//! by copying their exact bytes. A changed allocator encodes new pages.
use crate::durability::data::*;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionOrderedMap, ExecutionRequest};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::paged_base::FrozenRecordIdentity;
use worth_relational_storage::internal_port::storage::data::record_identity_pages::PagedRecordIdentity;

pub(super) fn capture(
    identity: &FrozenRecordIdentity,
    destination: &[RelationalDurableMemberLocator],
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<PagedRecordIdentity, Denial> {
    let (paged, pages) = match identity {
        FrozenRecordIdentity::Resident(state) => {
            let (paged, pages) =
                crate::durability::log::native_checkpoint_codec::identity_pages::encode_identity_pages(
                    state,
                )
                .map_err(|error| stop(error.into()))?;
            let pages = pages
                .into_iter()
                .map(|(locator, bytes)| RelationalCheckpointMember::New { locator, bytes })
                .collect::<Vec<_>>();
            (paged, pages)
        }
        FrozenRecordIdentity::Carried(base) => {
            let held = destination.iter().copied().collect::<BTreeSet<_>>();
            let mut pages = Vec::with_capacity(base.paged().pages.len());
            for &locator in &base.paged().pages {
                contracts::execution::request_custody::check_request(request)
                    .map_err(Denial::Request)?;
                pages.push(if held.contains(&locator) {
                    RelationalCheckpointMember::Carried { locator }
                } else {
                    let bytes = base.page_bytes(locator).map_err(|cause| {
                        stop(
                            DurabilityError::new(
                                RecoveryFailureClass::ColdStorageUnavailable,
                                format!("record identity page unavailable: {cause:?}"),
                            )
                            .into(),
                        )
                    })?;
                    RelationalCheckpointMember::New {
                        locator,
                        bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(
                            bytes.into_boxed_slice(),
                        )),
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
                    stop(contracts::durability::data::recovery_stop::map_stop(cause))
                })?;
        }
    }
    Ok(paged)
}

fn stop(cause: RelationalRecoveryStop) -> Denial {
    contracts::durability::data::native_checkpoint_capture_denial::capture_stop(cause)
}
