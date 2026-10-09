use super::*;
use crate::binary_output::BinaryOutput;
use crate::limits::WorthQueryPackageArchiveLimits;
use std::num::NonZeroU32;

fn attempt(bytes: &[u8]) -> RecordDecodeAttempt {
    RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap()
}

#[test]
fn variable_family_has_its_own_wire_without_changing_legacy_exact_bytes() {
    let family =
        WorthQueryDecisionFactFamily::new("a", WorthQueryDecisionFactKind::ObservedValue).unwrap();
    let variable = WorthQueryOperationDecisionFactContract::declared([family
        .clone()
        .with_variable_fact_count()])
    .unwrap();
    let mut output = BinaryOutput::with_capacity(32);
    write_decision_facts(&mut output, &variable).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(bytes, [0, 2, 0, 0, 0, 1, 0, 0, 0, 1, b'a', 0, 1, 0, 3]);
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(
        decode_decision_facts(&mut input, &mut attempt(&bytes)).unwrap(),
        variable
    );
    assert!(input.is_finished());
    let exact_bytes = [
        0, 2, 0, 0, 0, 1, 0, 0, 0, 1, b'a', 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2,
    ];
    let exact = WorthQueryOperationDecisionFactContract::declared([family
        .with_exact_fact_count(2)
        .unwrap()])
    .unwrap();
    let mut input = BinaryInput::new(&exact_bytes);
    assert_eq!(
        decode_decision_facts(&mut input, &mut attempt(&exact_bytes)).unwrap(),
        exact
    );
    let mut output = BinaryOutput::with_capacity(32);
    write_decision_facts(&mut output, &exact).unwrap();
    assert_eq!(output.into_bytes(), exact_bytes);
}

#[test]
fn optional_invariant_bounds_preserve_zero_and_refuse_redundant_dense_framing() {
    let requirement = WorthQueryInstalledInvariantExecutionRequirement::with_optional_bounds(
        "s",
        "f",
        NonZeroU32::new(1).unwrap(),
        WorthQueryInvariantEnforcement::Blocking,
        "p",
        ["l"],
        None,
        Some(0),
    )
    .unwrap();
    let contract = WorthQueryInvariantExecutionContract::declared([requirement]).unwrap();
    let bytes = optional_wire(None, Some(0));
    let mut output = BinaryOutput::with_capacity(64);
    write_invariant_execution(&mut output, &contract).unwrap();
    assert_eq!(output.into_bytes(), bytes);
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(
        decode_invariant_execution(&mut input, &mut attempt(&bytes)).unwrap(),
        contract
    );
    assert!(input.is_finished());
    let redundant = optional_wire(Some(1), Some(2));
    let denial =
        decode_invariant_execution(&mut BinaryInput::new(&redundant), &mut attempt(&redundant))
            .unwrap_err();
    assert_eq!(denial.kind(), Kind::NonCanonicalRecordSequence);
    let dense = WorthQueryInstalledInvariantExecutionRequirement::new(
        "s",
        "f",
        NonZeroU32::new(1).unwrap(),
        WorthQueryInvariantEnforcement::Blocking,
        "p",
        ["l"],
        1,
        2,
    )
    .unwrap();
    let dense = WorthQueryInvariantExecutionContract::declared([dense]).unwrap();
    let mut output = BinaryOutput::with_capacity(64);
    write_invariant_execution(&mut output, &dense).unwrap();
    let dense_bytes = output.into_bytes();
    assert_eq!(&dense_bytes[..2], &[0, 2]);
    assert_eq!(
        decode_invariant_execution(
            &mut BinaryInput::new(&dense_bytes),
            &mut attempt(&dense_bytes)
        )
        .unwrap(),
        dense
    );
}

fn optional_wire(state: Option<u64>, work: Option<u64>) -> Vec<u8> {
    // Independent explicit new grammar: one slot/family/version/enforcement,
    // executor and load family followed by two presence-framed values.
    let mut bytes = vec![
        0, 3, 0, 0, 0, 1, 0, 0, 0, 1, b's', 0, 0, 0, 1, b'f', 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, b'p',
        0, 0, 0, 1, 0, 0, 0, 1, b'l',
    ];
    for value in [state, work] {
        match value {
            None => bytes.extend_from_slice(&[0, 1]),
            Some(value) => {
                bytes.extend_from_slice(&[0, 2]);
                bytes.extend_from_slice(&value.to_be_bytes());
            }
        }
    }
    bytes
}
