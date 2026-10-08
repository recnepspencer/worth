use super::*;
use crate::{
    binary_output::BinaryOutput, limits::WorthQueryPackageArchiveLimits,
    record::decode_budget::WorthQueryPackageArchiveDecodeWork,
};

fn contract(omitted: bool) -> ResourceContract {
    let scale = ScaleLimits::bounded(64);
    let scale = if omitted {
        scale.without_work_budget()
    } else {
        scale
    };
    ResourceContract::declared([Strategy::new(
        StrategyName::new("test").unwrap(),
        Envelope::new(
            scale,
            ResourceLimits::bounded(64),
            Mode::Synchronous,
            None,
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
fn bytes(contract: &ResourceContract) -> Vec<u8> {
    let mut output = BinaryOutput::with_capacity(512);
    write_resource_contract(&mut output, contract).unwrap();
    output.into_bytes()
}
fn decode(bytes: &[u8]) -> Result<ResourceContract, Denial> {
    let mut input = BinaryInput::new(bytes);
    let mut budget = RecordDecodeAttempt::begin(
        WorthQueryPackageArchiveDecodeWork::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let contract = decode_resource_contract(&mut input, &mut budget)?;
    assert!(input.is_finished());
    Ok(contract)
}
#[test]
fn optional_work_tag_preserves_bounded_shape_and_omitted_identity() {
    let bounded = contract(false);
    let omitted = contract(true);
    let old = bytes(&bounded);
    let new = bytes(&omitted);
    assert_eq!(&old[..2], &2_u16.to_be_bytes());
    assert_eq!(&new[..2], &3_u16.to_be_bytes());
    assert_eq!(decode(&old).unwrap(), bounded);
    assert_eq!(decode(&new).unwrap(), omitted);
    assert_ne!(bounded.canonical_identity(), omitted.canonical_identity());
}
#[test]
fn optional_work_presence_and_noncanonical_record_tags_are_refused() {
    let mut malformed = bytes(&contract(true));
    let mut input = BinaryInput::new(&malformed);
    assert_eq!(input.u16().unwrap(), 3);
    assert_eq!(input.u32().unwrap(), 1);
    assert_eq!(input.text().unwrap(), "test");
    let offset = malformed.len() - input.remaining_len();
    malformed[offset..offset + 2].copy_from_slice(&9_u16.to_be_bytes());
    assert_eq!(
        decode(&malformed).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
    malformed[..2].copy_from_slice(&9_u16.to_be_bytes());
    assert_eq!(
        decode(&malformed).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
    let ResourceContract::Declared { strategies } = contract(false) else {
        unreachable!()
    };
    let mut noncanonical = BinaryOutput::with_capacity(512);
    noncanonical.u16(3);
    write_sequence(&mut noncanonical, &strategies, |output, strategy| {
        write_strategy_variant(output, strategy, true)
    })
    .unwrap();
    assert_eq!(
        decode(&noncanonical.into_bytes()).unwrap_err().kind(),
        Kind::NonCanonicalRecordSequence
    );
}

mod atomic;
