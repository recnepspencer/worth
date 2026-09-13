use sha2::{Digest, Sha256};

use super::{RootWireDenial, RootWireIdentity, RootWireRole};

#[test]
fn role_bound_wire_rejects_every_identity_substitution() {
    let scenario: [u8; 32] = Sha256::digest(b"c9-root-scenario-1").into();
    let run: [u8; 32] = Sha256::digest(b"c9-root-run-1").into();
    let store = [7; 16];
    let recovery = RootWireIdentity::bind(RootWireRole::Recovery, scenario, run, store).unwrap();

    assert_eq!(recovery.protocol(), "store.physical.c9-root-localization");
    assert_eq!(recovery.version(), 1);
    assert_eq!(recovery.role(), RootWireRole::Recovery);
    assert_eq!(recovery.scenario_identity(), scenario);
    assert_eq!(recovery.run_identity(), run);
    assert_eq!(recovery.store_identity(), store);
    assert_ne!(recovery.identity(), [0; 32]);
    assert_eq!(
        recovery.require_binding(RootWireRole::Recovery, scenario, run, store),
        Ok(())
    );
    assert_eq!(
        recovery.require_binding(RootWireRole::OfflineVerifier, scenario, run, store),
        Err(RootWireDenial::RoleSubstitution)
    );
    assert_eq!(
        recovery.require_binding(RootWireRole::Recovery, [8; 32], run, store),
        Err(RootWireDenial::ScenarioSubstitution)
    );
    assert_eq!(
        recovery.require_binding(RootWireRole::Recovery, scenario, [9; 32], store),
        Err(RootWireDenial::RunSubstitution)
    );
    assert_eq!(
        recovery.require_binding(RootWireRole::Recovery, scenario, run, [6; 16]),
        Err(RootWireDenial::StoreSubstitution)
    );

    let mut encoded = bincode::serialize(&recovery).unwrap();
    let protocol = recovery.protocol().as_bytes();
    let protocol_offset = encoded
        .windows(protocol.len())
        .position(|window| window == protocol)
        .unwrap();
    encoded[protocol_offset] ^= 1;
    let substituted_protocol: RootWireIdentity = bincode::deserialize(&encoded).unwrap();
    assert_eq!(
        substituted_protocol.require_binding(RootWireRole::Recovery, scenario, run, store),
        Err(RootWireDenial::ProtocolSubstitution)
    );

    let mut encoded = bincode::serialize(&recovery).unwrap();
    *encoded.last_mut().unwrap() ^= 1;
    let substituted_identity: RootWireIdentity = bincode::deserialize(&encoded).unwrap();
    assert_eq!(
        substituted_identity.require_binding(RootWireRole::Recovery, scenario, run, store),
        Err(RootWireDenial::IdentitySubstitution)
    );
}
