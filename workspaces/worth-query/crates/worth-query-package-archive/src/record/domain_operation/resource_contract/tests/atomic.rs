use super::*;
fn atomic_contract(work: Option<u64>) -> ResourceContract {
    let mut scale = ScaleLimits::selective().with(ScaleAxis::CandidateItems, 0);
    if let Some(work) = work {
        scale = scale.with(ScaleAxis::WorkItems, work);
    }
    ResourceContract::declared([Strategy::new(
        StrategyName::new("test").unwrap(),
        Envelope::atomic(
            scale,
            ResourceLimits::selective().with(ResourceDimension::RetainedBytes, 64),
            SafePoint::new("boundary").unwrap(),
        ),
        ProviderRequirements::new(
            Provider::new("provider").unwrap(),
            AccessProduct::new("access").unwrap(),
            Allocator::new("arena").unwrap(),
        ),
    )])
    .unwrap()
}
fn text(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(&(value.len() as u32).to_be_bytes());
    output.extend_from_slice(value.as_bytes());
}
// Historical tags2/3: 12 scale entries, 20 resource entries; only WorkItems
// has a presence flag in tag3. This oracle does not call the production writer.
fn historical(omitted: bool) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(&(if omitted { 3_u16 } else { 2 }).to_be_bytes());
    output.extend_from_slice(&1_u32.to_be_bytes());
    text(&mut output, "test");
    if omitted {
        output.extend_from_slice(&1_u16.to_be_bytes());
    }
    for index in 0..12 {
        if index != 8 || !omitted {
            output.extend_from_slice(&64_u64.to_be_bytes());
        }
    }
    for _ in 0..20 {
        output.extend_from_slice(&64_u64.to_be_bytes());
    }
    for _ in 0..5 {
        output.extend_from_slice(&1_u16.to_be_bytes());
    }
    for value in ["boundary", "provider", "access", "arena"] {
        text(&mut output, value);
    }
    output
}
// Independent tag4 grammar: CandidateItems is axis7, RetainedBytes resource4.
fn atomic_wire(work: Option<u64>) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(&4_u16.to_be_bytes());
    output.extend_from_slice(&1_u32.to_be_bytes());
    text(&mut output, "test");
    output.extend_from_slice(&1_u16.to_be_bytes());
    for index in 0..12 {
        let value = if index == 7 {
            Some(0)
        } else if index == 8 {
            work
        } else {
            None
        };
        output.extend_from_slice(&(if value.is_some() { 2_u16 } else { 1 }).to_be_bytes());
        if let Some(value) = value {
            output.extend_from_slice(&value.to_be_bytes());
        }
    }
    for index in 0..20 {
        output.extend_from_slice(&(if index == 4 { 2_u16 } else { 1 }).to_be_bytes());
        if index == 4 {
            output.extend_from_slice(&64_u64.to_be_bytes());
        }
    }
    for _ in 0..5 {
        output.extend_from_slice(&1_u16.to_be_bytes());
    }
    for value in ["boundary", "provider", "access", "arena"] {
        text(&mut output, value);
    }
    output
}
#[test]
fn atomic_archive_preserves_historical_bytes_and_has_one_framed_grammar() {
    for omitted in [false, true] {
        let expected = historical(omitted);
        assert_eq!(bytes(&contract(omitted)), expected);
        assert_eq!(decode(&expected).unwrap(), contract(omitted));
    }
    for work in [None, Some(0), Some(7)] {
        let expected = atomic_wire(work);
        let actual = atomic_contract(work);
        assert_eq!(bytes(&actual), expected);
        assert_eq!(decode(&expected).unwrap(), actual);
    }
    assert_ne!(
        atomic_contract(None).canonical_identity(),
        atomic_contract(Some(0)).canonical_identity()
    );
    let mut strategies = atomic_contract(None).strategies().to_vec();
    let mut bounded = contract(true).strategies()[0].clone();
    bounded = Strategy::new(
        StrategyName::new("bounded").unwrap(),
        bounded.envelope().clone(),
        bounded.provider_requirements().clone(),
    );
    strategies.push(bounded);
    let mixed = ResourceContract::declared(strategies).unwrap();
    assert_eq!(decode(&bytes(&mixed)).unwrap(), mixed);
}
#[test]
fn atomic_archive_rejects_unknown_tags_postures_and_redundant_dense_encoding() {
    for offset in [14, 16] {
        // strategy text ends at14; boundary then first axis presence
        let mut malformed = atomic_wire(None);
        malformed[offset..offset + 2].copy_from_slice(&9_u16.to_be_bytes());
        assert_eq!(
            decode(&malformed).unwrap_err().kind(),
            Kind::UnsupportedRecordVariant
        );
    }
    let mut malformed = atomic_wire(None);
    let posture_offset = 16 + 12 * 2 + 8 + 20 * 2 + 8;
    malformed[posture_offset..posture_offset + 2].copy_from_slice(&2_u16.to_be_bytes());
    assert_eq!(
        decode(&malformed).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );
    let mut output = BinaryOutput::with_capacity(512);
    output.u16(4);
    write_sequence(
        &mut output,
        contract(false).strategies(),
        sparse::write_strategy,
    )
    .unwrap();
    assert_eq!(
        decode(&output.into_bytes()).unwrap_err().kind(),
        Kind::NonCanonicalRecordSequence
    );
}

#[test]
fn dense_contract_retains_historical_v2_identity() {
    assert_eq!(
        contract(false).canonical_identity(),
        "d3fccbe9e3def582d7f69e9aa4747741dabed65fcfd9a146f2a081fa26027e37"
    );
}
