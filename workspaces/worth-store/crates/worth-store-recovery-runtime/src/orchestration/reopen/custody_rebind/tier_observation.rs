//! Consuming media observation returns the same namespace backing before conversion.
use super::super::custody_indeterminate;
use super::observed_published_free_header;
use crate::entry::PhysicalRecoveryOutcome;
use crate::progression::{NamespaceDurablePayload, NamespaceDurablePhysicalRecovery};

#[inline(never)]
pub(in crate::orchestration::reopen) fn observe_selected_tier(
    durable: NamespaceDurablePhysicalRecovery,
    completed: &worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen,
) -> Result<NamespaceDurablePhysicalRecovery, PhysicalRecoveryOutcome> {
    if durable
        .payload()
        .state
        .verified_selected_tier_custody
        .is_none()
    {
        return Ok(durable);
    }
    let (
        NamespaceDurablePayload {
            mut state,
            expectation,
            publication_counters,
            publication_settlement,
            reopened_backing,
        },
        namespace_backing,
    ) = durable.into_parts();
    let published = completed.root();
    let selected_free = if let Some(claim) = &state.verified_selected_tier_custody {
        let byte_limit = claim.free_header().encode(completed.format()).len() as u64;
        let (media, observed) = observed_published_free_header(
            state.authority.media,
            &mut state.integrity_trace,
            published,
            completed.format(),
            byte_limit,
        );
        state.authority.media = media;
        match observed {
            Some(header) => Some(header),
            None => {
                return Err(custody_indeterminate(
                    state,
                    publication_counters,
                    publication_settlement,
                ))
            }
        }
    } else {
        None
    };
    if let Some(claim) = &mut state.verified_selected_tier_custody
        && claim
            .rebind_published_root_and_free_header(
                &state.selection,
                published,
                selected_free
                    .as_ref()
                    .expect("tier claim requires observed free header"),
                completed.format(),
            )
            .is_err()
    {
        return Err(custody_indeterminate(
            state,
            publication_counters,
            publication_settlement,
        ));
    }

    Ok(namespace_backing.finish(NamespaceDurablePayload {
        state,
        expectation,
        publication_counters,
        publication_settlement,
        reopened_backing,
    }))
}
