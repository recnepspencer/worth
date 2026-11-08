//! Values only: recorded identities do not assert live actor authority.
use super::*;

#[test]
fn ending_provenance_keeps_principal_coordinates_and_home_owner_scope_distinct() {
    let actors = [
        WorkflowEndingActorProvenance::Principal(EntityId::new(PartitionId(19), 83, 7)),
        WorkflowEndingActorProvenance::HomeWorldOwner(DurableWorldOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([9; 16]).unwrap(),
            WorldOwnerSlotIdentity::from_recorded(5),
        )),
    ];
    assert_ne!(actors[0].recorded_bytes(), actors[1].recorded_bytes());
    for actor in actors {
        assert_eq!(
            WorkflowEndingActorProvenance::from_recorded_bytes(actor.recorded_bytes().as_bytes()),
            Ok(actor)
        );
    }
}

#[test]
fn ending_provenance_refuses_unknown_versions_kinds_and_incomplete_or_extra_payloads() {
    use WorkflowEndingActorDecodeDenial as Denial;
    let actor =
        WorkflowEndingActorProvenance::HomeWorldOwner(DurableWorldOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([9; 16]).unwrap(),
            WorldOwnerSlotIdentity::from_recorded(5),
        ));
    let encoded = actor.recorded_bytes();
    let bytes = encoded.as_bytes();
    for end in 0..bytes.len() {
        assert!(WorkflowEndingActorProvenance::from_recorded_bytes(&bytes[..end]).is_err());
    }
    let mut extra = bytes.to_vec();
    extra.push(0);
    assert_eq!(
        WorkflowEndingActorProvenance::from_recorded_bytes(&extra),
        Err(Denial::Malformed)
    );
    let mut unknown = bytes.to_vec();
    unknown[8] = 2;
    assert_eq!(
        WorkflowEndingActorProvenance::from_recorded_bytes(&unknown),
        Err(Denial::UnknownActorKind(2))
    );
    let mut future = bytes.to_vec();
    future[4..8].copy_from_slice(&2_u32.to_le_bytes());
    assert!(matches!(
        WorkflowEndingActorProvenance::from_recorded_bytes(&future),
        Err(Denial::UnsupportedVersion(_))
    ));
    let mut absent_home = bytes.to_vec();
    absent_home[9..25].fill(0);
    assert_eq!(
        WorkflowEndingActorProvenance::from_recorded_bytes(&absent_home),
        Err(Denial::Malformed)
    );
    let mut wrong_kind = bytes.to_vec();
    wrong_kind[8] = 0;
    assert_eq!(
        WorkflowEndingActorProvenance::from_recorded_bytes(&wrong_kind),
        Err(Denial::Malformed)
    );
}
