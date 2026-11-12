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

#[test]
fn role_bound_wire_uses_fixed_binary_digests_and_refuses_retired_array_fields() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct RetiredWire<'a> {
        protocol: &'a str,
        version: u16,
        role: RootWireRole,
        scenario_identity: [u8; 32],
        run_identity: [u8; 32],
        store_identity: [u8; 16],
        identity: [u8; 32],
    }

    let mut encoded_length = None;
    for value in [1, 127, 128, 255] {
        let wire =
            RootWireIdentity::bind(RootWireRole::Recovery, [value; 32], [value; 32], [7; 16])
                .unwrap();
        let encoded = bincode::serialize(&wire).unwrap();
        assert_eq!(*encoded_length.get_or_insert(encoded.len()), encoded.len());
        let decoded: RootWireIdentity = bincode::deserialize(&encoded).unwrap();
        assert_eq!(decoded, wire);
        assert_eq!(
            decoded.require_binding(RootWireRole::Recovery, [value; 32], [value; 32], [7; 16],),
            Ok(())
        );
        let retired = RetiredWire {
            protocol: wire.protocol(),
            version: wire.version(),
            role: wire.role(),
            scenario_identity: [value; 32],
            run_identity: [value; 32],
            store_identity: wire.store_identity(),
            identity: Sha256::digest(
                bincode::serialize(&(
                    wire.protocol(),
                    wire.version(),
                    wire.role(),
                    [value; 32],
                    [value; 32],
                    wire.store_identity(),
                ))
                .unwrap(),
            )
            .into(),
        };
        assert!(
            bincode::deserialize::<RootWireIdentity>(&bincode::serialize(&retired).unwrap(),)
                .is_err()
        );
    }
}
