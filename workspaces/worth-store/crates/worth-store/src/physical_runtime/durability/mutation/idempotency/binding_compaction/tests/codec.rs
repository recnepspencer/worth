use super::super::encoding::{
    encode_group_sealed, encode_terminal, encode_unsealed, encode_wal_bound,
};
use super::*;
use crate::physical_runtime::durability::mutation::idempotency::{
    fate::PersistedPhysicalMutationFate, persisted_binding::tests::fixture,
    registry::PhysicalMutationBindingBasis,
};
use crate::physical_runtime::{CompletedPhysicalMutationFact, RecordAppendObservation};
use worth_store_physical_format::PersistedRecordIdentity;

#[test]
fn every_current_compaction_state_preserves_exact_byte_grammar() {
    let (fixture, binding) = fixture::binding();
    let basis = PhysicalMutationBindingBasis::new(
        binding.key().clone(),
        binding.fingerprint(),
        binding.mutation(),
    );
    let context =
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, fixture.idempotency);
    let unsealed = encode_unsealed(&basis);
    let DecodedPhysicalMutationBindingRecord::RebuiltUnsealed(decoded) =
        DecodedPhysicalMutationBindingRecord::decode(&unsealed, context).unwrap()
    else {
        panic!("unsealed state changed");
    };
    assert_eq!(encode_unsealed(&decoded), unsealed);
    let grouped = encode_group_sealed(&basis, binding.group());
    let DecodedPhysicalMutationBindingRecord::RebuiltGroupSealed {
        basis: decoded,
        group,
    } = DecodedPhysicalMutationBindingRecord::decode(&grouped, context).unwrap()
    else {
        panic!("group-sealed state changed");
    };
    assert_eq!(encode_group_sealed(&decoded, group), grouped);
    let wal = encode_wal_bound(&binding);
    let DecodedPhysicalMutationBindingRecord::WalBound { persisted, .. } =
        DecodedPhysicalMutationBindingRecord::decode(&wal, context).unwrap()
    else {
        panic!("WAL-bound state changed");
    };
    assert_eq!(persisted.bytes(), binding.bytes());
    assert_eq!(encode_wal_bound(&persisted), wal);
    let no_effect = PersistedPhysicalMutationFate::proven_no_effect(
        crate::physical_runtime::ProvenNoEffectPhysicalMutation::before_group_seal(
            basis.key().identity(),
            basis.fingerprint(),
            basis.mutation(),
            PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal,
        ),
    );
    let terminal = encode_terminal(&basis, &no_effect);
    let DecodedPhysicalMutationBindingRecord::Terminal {
        basis: decoded,
        fate,
    } = DecodedPhysicalMutationBindingRecord::decode(&terminal, context).unwrap()
    else {
        panic!("terminal state changed");
    };
    assert_eq!(encode_terminal(&decoded, &fate), terminal);
    fixture.media.close();
}

#[test]
fn completed_terminal_records_round_trip_and_hostile_counts_deny_before_allocation() {
    let (fixture, binding) = fixture::binding();
    let basis = PhysicalMutationBindingBasis::new(
        binding.key().clone(),
        binding.fingerprint(),
        binding.mutation(),
    );
    let records = vec![
        PersistedRecordIdentity::new([131; 16], 1).unwrap(),
        PersistedRecordIdentity::new([131; 16], 2).unwrap(),
    ]
    .into_boxed_slice();
    let fact = CompletedPhysicalMutationFact::from_persisted_terminal(
        &binding,
        2,
        7,
        records,
        RecordAppendObservation::from_persisted_fields([0; 13]),
    );
    let fate = PersistedPhysicalMutationFate::completed(binding.clone(), fact);
    let encoded = encode_terminal(&basis, &fate);
    let context =
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, fixture.idempotency);
    let DecodedPhysicalMutationBindingRecord::Terminal {
        basis: decoded,
        fate,
    } = DecodedPhysicalMutationBindingRecord::decode(&encoded, context).unwrap()
    else {
        panic!("completed state changed");
    };
    assert_eq!(encode_terminal(&decoded, &fate), encoded);
    // Terminal basis bytes are identical to the independent unsealed grammar;
    // then class, length-prefixed persisted binding, effect count, root, count.
    let count_at = encode_unsealed(&basis).len() + 1 + 8 + binding.bytes().len() + 4 + 8;
    assert_eq!(&encoded[count_at..count_at + 4], &2_u32.to_le_bytes());
    let mut huge = encoded.clone();
    huge[count_at..count_at + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    for bytes in [
        &huge[..],
        &encoded[..encoded.len() - 1],
        &encoded[..count_at + 4],
    ] {
        assert!(matches!(
            DecodedPhysicalMutationBindingRecord::decode(bytes, context),
            Err(PhysicalBindingCompactionRecordDecodeDenial::Persisted(
                PhysicalPersistedBindingDecodeDenial::Truncated
            ))
        ));
    }
    fixture.media.close();
}
