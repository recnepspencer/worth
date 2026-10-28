use crate::reachability::denial::BlobReachabilityDenial;
use crate::reachability::edges::BlobReachabilityAuthorityKey;
use crate::reachability::types::BlobChunkReachabilityRegistry;
use crate::{BlobReachabilityEdge, BlobReachabilityProtectedHold};

pub(crate) fn verify_edge_authority(
    registry: &mut BlobChunkReachabilityRegistry,
    edge: &BlobReachabilityEdge,
) -> Result<(), BlobReachabilityDenial> {
    if let Some(authority) = registry.authority()
        && !authority.matches(edge)
    {
        registry.set_stored_counters(registry.stored_counters().record_wrong_authority_denial());
        return Err(BlobReachabilityDenial::WrongBlobAuthority {
            counters: registry.stored_counters(),
        });
    }
    Ok(())
}

pub(crate) fn verify_hold_authority(
    registry: &mut BlobChunkReachabilityRegistry,
    hold: &BlobReachabilityProtectedHold,
) -> Result<Option<BlobReachabilityAuthorityKey>, BlobReachabilityDenial> {
    let hold_authority = hold.authority_key();
    if let Some(authority) = registry.authority() {
        if authority != hold_authority {
            registry
                .set_stored_counters(registry.stored_counters().record_wrong_authority_denial());
            return Err(BlobReachabilityDenial::WrongBlobAuthority {
                counters: registry.stored_counters(),
            });
        }
        return Ok(None);
    }
    if hold.can_seed_registry_authority() {
        return Ok(Some(hold_authority));
    }
    registry.set_stored_counters(registry.stored_counters().record_wrong_authority_denial());
    Err(BlobReachabilityDenial::InvalidProtectedHold {
        counters: registry.stored_counters(),
    })
}

pub(crate) fn require_registry_bound_hold_authority(
    registry: &mut BlobChunkReachabilityRegistry,
) -> Result<BlobReachabilityAuthorityKey, BlobReachabilityDenial> {
    let Some(authority) = registry.authority().clone() else {
        registry.set_stored_counters(registry.stored_counters().record_wrong_authority_denial());
        return Err(BlobReachabilityDenial::InvalidProtectedHold {
            counters: registry.stored_counters(),
        });
    };
    Ok(authority)
}
