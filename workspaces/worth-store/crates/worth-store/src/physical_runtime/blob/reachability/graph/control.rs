use std::collections::BTreeSet;

use super::super::scan::{control::ControlFact, SelectedInventory};
use super::super::BlobReachabilityFailure as Failure;

/// A selected manifest is only a proposed release source. The publication
/// must actually be absent from the current route, and its selected V3
/// descriptor and reservation must bind the same Store-authored custody.
pub(super) fn released_sessions(
    inventory: &SelectedInventory,
) -> Result<BTreeSet<[u8; 16]>, Failure> {
    let mut released = BTreeSet::new();
    for (manifest_record, fact) in inventory.facts.iter().filter(|(_, fact)| fact.current) {
        let Some(ControlFact::ReleasedManifest {
            attempt,
            basis,
            frame_digest,
            source_basis_digest,
        }) = fact.control
        else {
            continue;
        };
        if inventory
            .facts
            .get(&basis.publication_record())
            .is_some_and(|fact| fact.current)
        {
            continue;
        }
        let store = basis.publication().store();
        let mut descriptors = 0_u8;
        let mut reservations = 0_u8;
        let mut request = None;
        for selected in inventory.facts.values().filter(|fact| fact.current) {
            match selected.control {
                Some(ControlFact::ReleasedDescriptor {
                    store: found_store,
                    attempt: found_attempt,
                    manifest,
                    manifest_digest,
                    source_basis_digest: found_basis,
                    request: found_request,
                }) if found_attempt == attempt => {
                    if found_store != store
                        || manifest != *manifest_record
                        || manifest_digest != frame_digest
                        || found_basis != source_basis_digest
                        || request.is_some_and(|value| value != found_request)
                    {
                        return Err(Failure::ConflictingSelectedFate);
                    }
                    descriptors = descriptors
                        .checked_add(1)
                        .ok_or(Failure::ConflictingSelectedFate)?;
                    request = Some(found_request);
                }
                Some(ControlFact::Reservation {
                    store: found_store,
                    attempt: found_attempt,
                    manifest,
                    manifest_digest,
                    source_basis_digest: found_basis,
                    request: found_request,
                }) if found_attempt == attempt => {
                    if found_store != store
                        || manifest != *manifest_record
                        || manifest_digest != frame_digest
                        || found_basis != source_basis_digest
                    {
                        return Err(Failure::ConflictingSelectedFate);
                    }
                    reservations = reservations
                        .checked_add(1)
                        .ok_or(Failure::ConflictingSelectedFate)?;
                    if request.is_some_and(|value| value != found_request) {
                        return Err(Failure::ConflictingSelectedFate);
                    }
                    request = Some(found_request);
                }
                _ => {}
            }
        }
        if descriptors != 1 || reservations != 1 {
            return Err(Failure::ConflictingSelectedFate);
        }
        released.insert(basis.session());
    }
    Ok(released)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use sha2::{Digest, Sha256};
    use worth_store_physical_format::{
        BlobGenerationPublicationV1, OriginalDropReservationRequestV1, PersistedRecordIdentity,
        ReleasedGenerationReclaimBasisV1,
    };

    use super::*;
    use crate::physical_runtime::blob::reachability::{
        scan::{Fact, Role},
        BlobReachabilityLimits,
    };

    fn id(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
    }

    fn fact(role: Role, control: ControlFact) -> Fact {
        Fact {
            current: true,
            held: false,
            role,
            edges: vec![],
            declaration: None,
            claim: None,
            closure: None,
            control: Some(control),
            reuse: None,
        }
    }

    #[test]
    fn manifest_alone_cannot_relabel_a_selected_chunk_as_released() {
        let publication = BlobGenerationPublicationV1::new(
            [1; 16],
            [2; 16],
            [3; 16],
            1,
            id(8),
            [4; 32],
            65_536,
            [5; 32],
            65_536,
            [6; 32],
        )
        .unwrap();
        let basis = ReleasedGenerationReclaimBasisV1::new(
            publication,
            id(9),
            Sha256::digest(publication.encode()).into(),
            [7; 32],
        )
        .unwrap();
        let limits = BlobReachabilityLimits::new(
            NonZeroU64::new(8).unwrap(),
            NonZeroU64::new(1024).unwrap(),
            NonZeroU64::new(2).unwrap(),
            NonZeroU64::new(8).unwrap(),
        );
        let mut selected = SelectedInventory::new(limits).unwrap();
        selected.facts.insert(
            id(1),
            fact(
                Role::ReleasedControl([2; 16]),
                ControlFact::ReleasedManifest {
                    attempt: [8; 16],
                    basis,
                    frame_digest: [9; 32],
                    source_basis_digest: [10; 32],
                },
            ),
        );
        assert!(matches!(
            released_sessions(&selected),
            Err(Failure::ConflictingSelectedFate)
        ));
        let request = OriginalDropReservationRequestV1::new([11; 32], [12; 32], 1, 3).unwrap();
        selected.facts.insert(
            id(2),
            fact(
                Role::Control,
                ControlFact::ReleasedDescriptor {
                    store: [1; 16],
                    attempt: [8; 16],
                    manifest: id(1),
                    manifest_digest: [9; 32],
                    source_basis_digest: [10; 32],
                    request,
                },
            ),
        );
        assert!(matches!(
            released_sessions(&selected),
            Err(Failure::ConflictingSelectedFate)
        ));
        selected.facts.insert(
            id(3),
            fact(
                Role::Control,
                ControlFact::Reservation {
                    store: [1; 16],
                    attempt: [8; 16],
                    manifest: id(1),
                    manifest_digest: [9; 32],
                    source_basis_digest: [10; 32],
                    request,
                },
            ),
        );
        assert!(released_sessions(&selected).unwrap().contains(&[2; 16]));
        if let Some(ControlFact::Reservation { request, .. }) =
            &mut selected.facts.get_mut(&id(3)).unwrap().control
        {
            *request = OriginalDropReservationRequestV1::new([13; 32], [12; 32], 1, 3).unwrap();
        }
        assert!(matches!(
            released_sessions(&selected),
            Err(Failure::ConflictingSelectedFate)
        ));
    }
}
