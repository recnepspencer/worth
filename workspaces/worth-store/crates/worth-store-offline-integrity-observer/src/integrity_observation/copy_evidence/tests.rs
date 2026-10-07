use super::{
    sha256, CopyFinalFact, CopyIntentFact, BINDING_DOMAIN, CLASSIFIED_INTENT_DOMAIN,
    CURRENT_PROJECTION_DOMAIN, FINAL_DOMAIN, INTENT_DOMAIN,
};

const STORE: [u8; 16] = [5; 16];
const OPERATION: [u8; 32] = [7; 32];

fn field(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}

fn intent_and_record(classified: bool) -> (Vec<u8>, [u8; 24]) {
    let mut body = [0; 200];
    body[..32].copy_from_slice(&OPERATION);
    body[32..40].copy_from_slice(&4_u64.to_le_bytes());
    body[40..56].copy_from_slice(&[3; 16]);
    body[56..64].copy_from_slice(&9_u64.to_le_bytes());
    body[72..80].copy_from_slice(&3_u64.to_le_bytes());
    body[80..88].copy_from_slice(&1_u64.to_le_bytes());
    body[88..96].copy_from_slice(&2_u64.to_le_bytes());
    body[96..104].copy_from_slice(&2_u64.to_le_bytes());
    body[112..120].copy_from_slice(&4096_u64.to_le_bytes());
    body[120..128].copy_from_slice(&3_u64.to_le_bytes());
    body[136..144].copy_from_slice(&4096_u64.to_le_bytes());
    body[144..152].copy_from_slice(&10_u64.to_le_bytes());
    body[152..160].copy_from_slice(&4096_u64.to_le_bytes());
    body[160..164].copy_from_slice(&1024_u32.to_le_bytes());
    body[164..168].copy_from_slice(&4_u32.to_le_bytes());
    let mut intent = Vec::from(if classified {
        CLASSIFIED_INTENT_DOMAIN
    } else {
        INTENT_DOMAIN
    });
    intent.push(1);
    intent.extend_from_slice(&body);
    if classified {
        intent.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0]); // opaque, primary
        intent.extend_from_slice(&[1, 0, 0, 0, 2, 0, 0]); // opaque, cold
    }
    (intent, body[40..64].try_into().unwrap())
}

fn final_payload(
    domain: &[u8],
    semantic: Option<&[u8]>,
    classified: bool,
    final_destination_tier: u8,
) -> Vec<u8> {
    let (intent, record) = intent_and_record(classified);
    let mut projection = Vec::new();
    field(&mut projection, domain);
    projection.extend_from_slice(&4_u64.to_le_bytes());
    field(&mut projection, b"root");
    projection.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut projection, &record);
    projection.push(1); // SourceCopy
    field(&mut projection, &intent);
    projection.extend_from_slice(&10_u64.to_le_bytes());
    projection.extend_from_slice(&sha256(&intent));
    projection.extend_from_slice(&1_u64.to_le_bytes());
    let mut destination = vec![2];
    destination.extend_from_slice(&record);
    for value in [3_u64, 2, 10, 3, 0, 4096] {
        destination.extend_from_slice(&value.to_le_bytes());
    }
    destination.extend_from_slice(&if classified {
        [1, 0, 0, 0, final_destination_tier, 0, 0]
    } else {
        [0; 7]
    });
    field(&mut projection, &destination);
    projection.extend_from_slice(&0_u64.to_le_bytes());
    projection.extend_from_slice(&0_u64.to_le_bytes());
    if let Some(semantic) = semantic {
        field(&mut projection, semantic);
    }

    let mut redo = Vec::new();
    field(&mut redo, FINAL_DOMAIN);
    redo.extend_from_slice(&20_u64.to_le_bytes());
    field(&mut redo, &projection);

    let mut binding = Vec::new();
    field(&mut binding, BINDING_DOMAIN);
    field(&mut binding, &OPERATION);
    field(&mut binding, &STORE);
    field(&mut binding, &[1; 32]);
    binding.extend_from_slice(&[1; 16]);
    field(&mut binding, &[1; 32]);
    field(&mut binding, &[1; 32]);
    field(&mut binding, &STORE);
    binding.extend_from_slice(&record);
    field(&mut binding, &[1; 32]);
    binding.extend_from_slice(&1_u32.to_le_bytes());
    binding.extend_from_slice(&1_u32.to_le_bytes());
    field(&mut binding, &[1; 32]);
    field(&mut binding, &[1; 32]);
    binding.extend_from_slice(&20_u64.to_le_bytes());
    binding.extend_from_slice(&21_u64.to_le_bytes());
    field(&mut binding, &sha256(&redo));

    let mut payload = Vec::new();
    field(&mut payload, &binding);
    field(&mut payload, &redo);
    payload
}

#[test]
fn final_copy_accepts_literal_current_none_operation() {
    let final_fact = CopyFinalFact::decode(
        &final_payload(CURRENT_PROJECTION_DOMAIN, Some(&[0]), false, 0),
        STORE,
        (20, 21),
    )
    .expect("literal current copy publication should bind");
    assert_eq!(final_fact.source_root, 4);
    assert_eq!(final_fact.result_root, 5);
    assert_eq!(final_fact.intent.lsn, 10);
}

#[test]
fn final_copy_rejects_missing_or_non_none_current_operation() {
    for semantic in [None, Some(&[][..]), Some(&[0, 0][..]), Some(&[1][..])] {
        assert!(CopyFinalFact::decode(
            &final_payload(CURRENT_PROJECTION_DOMAIN, semantic, false, 0),
            STORE,
            (20, 21),
        )
        .is_none());
    }
    assert!(CopyFinalFact::decode(
        &final_payload(
            b"store.physical.recovery-projection.v5",
            Some(&[0]),
            false,
            0
        ),
        STORE,
        (20, 21),
    )
    .is_none());
}

#[test]
fn classified_intent_preserves_exact_route_class_and_target_tier() {
    let (intent, _) = intent_and_record(true);
    let admitted = CopyIntentFact::decode(&intent, (10, 11)).expect("current classified intent");
    assert_eq!(admitted.source_route_metadata, [1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(admitted.destination_route_metadata, [1, 0, 0, 0, 2, 0, 0]);
    let metadata = CLASSIFIED_INTENT_DOMAIN.len() + 1 + 200;
    for (index, byte) in [(0, 0), (12, 1)] {
        let mut changed = intent.clone();
        changed[metadata + index] = byte;
        assert!(CopyIntentFact::decode(&changed, (10, 11)).is_none());
    }
    let mut changed_class = intent.clone();
    changed_class[metadata + 7..metadata + 11].copy_from_slice(&[2, 1, 0, 0]);
    assert!(CopyIntentFact::decode(&changed_class, (10, 11)).is_none());
    assert!(CopyIntentFact::decode(&intent[..intent.len() - 1], (10, 11)).is_none());
    assert!(CopyIntentFact::decode(&intent, (10, 12)).is_none());
}

#[test]
fn classified_final_copy_requires_embedded_destination_route() {
    let admitted = CopyFinalFact::decode(
        &final_payload(CURRENT_PROJECTION_DOMAIN, Some(&[0]), true, 2),
        STORE,
        (20, 21),
    )
    .expect("current classified copy publication");
    assert_eq!(admitted.intent.destination_route_metadata[4], 2);
    assert!(CopyFinalFact::decode(
        &final_payload(CURRENT_PROJECTION_DOMAIN, Some(&[0]), true, 1),
        STORE,
        (20, 21),
    )
    .is_none());
}
