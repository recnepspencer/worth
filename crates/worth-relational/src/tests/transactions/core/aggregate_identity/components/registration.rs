//! Installation rejects invalid source meaning; old orbit wire/identity bytes stay exact.
use super::super::fixtures as orbit;
use super::fixtures::*;

#[test]
fn component_sources_normalize_and_refuse_empty_private_or_non_homogeneous_kinds() {
    let mut base = registry();
    let installed = base.relation_kinds.remove(&KindId(7)).unwrap();
    let contract = installed
        .relation_integrity
        .aggregate_identity
        .as_ref()
        .unwrap();
    assert_eq!(contract.source.relations(), [KindId(2), KindId(6)]);
    assert!(base
        .clone()
        .register_relation_kind(installed.clone())
        .is_ok());
    for relations in [vec![], vec![KindId(7)], vec![KindId(4)], vec![KindId(2000)]] {
        let mut hostile = installed.clone();
        hostile
            .relation_integrity
            .aggregate_identity
            .as_mut()
            .unwrap()
            .source = AggregateIdentitySource::Components { relations };
        assert!(base.clone().register_relation_kind(hostile).is_err());
    }
    let mut source = base.relation_kinds.remove(&KindId(6)).unwrap();
    source.relation_integrity.endpoint_kind_contracts[0].allowed_target_kinds = vec![KindId(3)];
    let hostile = base.register_relation_kind(source).unwrap();
    assert!(hostile.register_relation_kind(installed).is_err());
}

#[test]
fn orbit_scalar_wire_and_canonical_bytes_are_preserved_and_components_are_distinct() {
    // This is the accepted old struct grammar, not the new enum's serializer.
    #[derive(serde::Serialize)]
    struct OrbitWire {
        definition: AggregateDefinitionIdentity,
        source_relation: KindId,
        member_kind: KindId,
        membership_kind: KindId,
        storage: AggregateIdentityStorage,
    }
    let contract = orbit::descriptor();
    let old = OrbitWire {
        definition: contract.definition,
        source_relation: KindId(2),
        member_kind: KindId(1),
        membership_kind: KindId(5),
        storage: contract.storage,
    };
    assert_eq!(
        serde_json::to_vec(&contract).unwrap(),
        serde_json::to_vec(&old).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec(&contract).unwrap(),
        rmp_serde::to_vec(&old).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec_named(&contract).unwrap(),
        rmp_serde::to_vec_named(&old).unwrap()
    );
    let decoded: AggregateIdentityContractDeclaration =
        rmp_serde::from_slice(&rmp_serde::to_vec(&old).unwrap()).unwrap();
    assert_eq!(decoded, contract);
    let mut bytes = vec![7; 32];
    for kind in [2_u32, 1, 5, 3, 4] {
        bytes.extend_from_slice(&kind.to_be_bytes());
    }
    for aspect in [3001_u64, 3002] {
        bytes.extend_from_slice(&aspect.to_be_bytes());
    }
    assert_eq!(contract.canonical_bytes(), bytes);
    let mut components = contract.clone();
    components.source = AggregateIdentitySource::Components {
        relations: vec![KindId(2)],
    };
    assert_ne!(components.canonical_bytes(), bytes);
    assert_eq!(
        serde_json::from_slice::<AggregateIdentityContractDeclaration>(
            &serde_json::to_vec(&components).unwrap()
        )
        .unwrap(),
        components
    );
    assert_eq!(
        rmp_serde::from_slice::<AggregateIdentityContractDeclaration>(
            &rmp_serde::to_vec_named(&components).unwrap()
        )
        .unwrap(),
        components
    );
    let encoded = rmp_serde::to_vec(&components).unwrap();
    assert_ne!(encoded, rmp_serde::to_vec(&old).unwrap());
    assert_eq!(
        rmp_serde::from_slice::<AggregateIdentityContractDeclaration>(&encoded).unwrap(),
        components
    );
}
