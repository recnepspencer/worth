use super::super::retention::BridgeRetentionLedger;
use super::super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan;
use super::*;
use crate::policy::BridgeConditionalRetentionBudget;
use worth_foundational::facade::{AspectValue, ContractValidatedAspectArtifact};

#[path = "payload_oracle.rs"]
mod payload_oracle;

use payload_oracle::{available_mask_bytes, baseline_bytes, observation_bytes, scalar};

#[test]
fn baseline_and_escaped_observations_retain_exact_custody_after_owner_close() {
    let plan = BridgeConditionalSemanticObservationPlan::managed_test_plan();
    let ledger =
        BridgeRetentionLedger::new(BridgeConditionalRetentionBudget::development()).unwrap();
    let current = scalar(AspectValue::UInt64(17));
    let observations = prepare_observations(
        &plan,
        &Default::default(),
        [Some(&current)].into_iter(),
        &ledger,
    )
    .unwrap();
    let retained = observation_bytes(
        current.payload().owned_allocation_capacity_bytes(),
        available_mask_bytes(),
    );
    assert_eq!(ledger.usage(), (0, 0, retained));
    let escaped = observations.clone();
    assert_eq!(
        ledger.usage().2,
        retained,
        "shared backing has one reservation"
    );
    let baseline = BridgeObservationBaselines::new(&ledger).unwrap();
    assert_eq!(ledger.usage().2, retained + baseline_bytes());
    baseline.publish(observations);
    let next = scalar(AspectValue::UInt64(23));
    let successor = prepare_observations(
        &plan,
        &baseline.snapshot().unwrap(),
        [Some(&next)].into_iter(),
        &ledger,
    )
    .unwrap();
    assert_eq!(successor[0].previous(), Some(&current));
    baseline.publish(successor);
    let successor_bytes = observation_bytes(
        current.payload().owned_allocation_capacity_bytes()
            + next.payload().owned_allocation_capacity_bytes(),
        available_mask_bytes(),
    );
    assert_eq!(
        ledger.usage().2,
        retained + baseline_bytes() + successor_bytes
    );
    assert_eq!(escaped.current(0), Some(&current));
    assert_eq!(baseline.snapshot().unwrap().current(0), Some(&next));
    ledger.close();
    drop(baseline);
    assert_eq!(
        ledger.usage().2,
        retained,
        "escaped old backing owns its bytes"
    );
    drop(escaped);
    assert_eq!(ledger.usage(), (0, 0, 0));
    assert_exact_boundary(&plan, &current, retained);
}

#[test]
fn canonical_variable_width_payloads_and_struct_fields_have_exact_retention_boundaries() {
    for case in payload_oracle::variable_width_cases() {
        let plan = BridgeConditionalSemanticObservationPlan::managed_test_plan_for(
            case.artifact.payload().contract().clone(),
            case.mask,
        );
        let expected = observation_bytes(case.payload_bytes, case.mask_bytes);
        assert_exact_boundary(&plan, &case.artifact, expected);
    }
}

#[test]
fn observation_retention_charges_the_shared_source_backing_and_spare_capacity() {
    use worth_foundational::facade::{
        AspectMask, ContractValidatedAspectValueView, InternedString,
    };
    let mut donor = String::with_capacity(4096);
    donor.push_str("retained");
    let artifact = scalar(AspectValue::String(InternedString::Raw(donor)));
    let ContractValidatedAspectValueView::Scalar(AspectValue::String(InternedString::Raw(source))) =
        artifact.payload().view()
    else {
        panic!("source owns a string");
    };
    assert_eq!(source.capacity(), 4096);
    let plan = BridgeConditionalSemanticObservationPlan::managed_test_plan_for(
        artifact.payload().contract().clone(),
        AspectMask::whole_aspect(),
    );
    let expected = observation_bytes(artifact.payload().owned_allocation_capacity_bytes(), 0);
    assert_exact_boundary(&plan, &artifact, expected);
    let ledger = BridgeRetentionLedger::new(BridgeConditionalRetentionBudget {
        maximum_retained_bytes: Some(expected),
        ..BridgeConditionalRetentionBudget::development()
    })
    .unwrap();
    let copied = prepare_observations(
        &plan,
        &Default::default(),
        [Some(&artifact)].into_iter(),
        &ledger,
    )
    .unwrap();
    let ContractValidatedAspectValueView::Scalar(AspectValue::String(InternedString::Raw(
        retained,
    ))) = copied.current(0).unwrap().payload().view()
    else {
        panic!("retained copy owns a string");
    };
    assert_eq!(retained.capacity(), source.capacity());
    assert_eq!(retained.as_ptr(), source.as_ptr());
    assert_eq!(retained.as_str(), source.as_str());
}

#[test]
fn immutable_stored_text_keeps_shared_proof_backing_and_exact_retained_custody() {
    use worth_foundational::facade::{AspectMask, InternedString, StoredValueText};
    let mut donor = String::with_capacity(4096);
    donor.push_str("retained");
    let capacity = donor.capacity();
    let pointer = donor.as_ptr();
    let artifact = scalar(AspectValue::String(InternedString::Stored(
        StoredValueText::new(donor),
    )));
    let plan = BridgeConditionalSemanticObservationPlan::managed_test_plan_for(
        artifact.payload().contract().clone(),
        AspectMask::whole_aspect(),
    );
    let expected = observation_bytes(artifact.payload().owned_allocation_capacity_bytes(), 0);
    assert!(
        expected >= capacity as u64,
        "retention includes the original backing"
    );
    assert_exact_boundary(&plan, &artifact, expected);
    let ledger = BridgeRetentionLedger::new(BridgeConditionalRetentionBudget {
        maximum_retained_bytes: Some(expected),
        ..BridgeConditionalRetentionBudget::development()
    })
    .unwrap();
    let retained = prepare_observations(
        &plan,
        &Default::default(),
        [Some(&artifact)].into_iter(),
        &ledger,
    )
    .unwrap();
    assert_eq!(ledger.usage().2, expected);
    let worth_foundational::facade::ContractValidatedAspectValueView::Scalar(value) =
        retained.current(0).unwrap().payload().view()
    else {
        panic!("scalar fixture")
    };
    assert_eq!(value.raw_text().unwrap().as_ptr(), pointer);
    drop(artifact);
    assert_eq!(value.raw_text(), Some("retained"));
    drop(retained);
    assert_eq!(ledger.usage(), (0, 0, 0));
}

fn assert_exact_boundary(
    plan: &BridgeConditionalSemanticObservationPlan,
    current: &ContractValidatedAspectArtifact,
    expected: u64,
) {
    for bytes in [expected, expected - 1] {
        let ledger = BridgeRetentionLedger::new(BridgeConditionalRetentionBudget {
            maximum_retained_bytes: Some(bytes),
            ..BridgeConditionalRetentionBudget::development()
        })
        .unwrap();
        let result = prepare_observations(
            plan,
            &Default::default(),
            [Some(current)].into_iter(),
            &ledger,
        );
        if bytes == expected {
            let retained = result.unwrap();
            assert_eq!(retained.current(0), Some(current));
            assert_eq!(
                retained[0].projection_mask(),
                plan.projection_mask(0).unwrap()
            );
            assert_eq!(ledger.usage(), (0, 0, expected));
            drop(retained);
        } else {
            assert_eq!(
                result.unwrap_err().kind(),
                super::super::BridgeConditionalDenialKind::ConditionalRetentionCapacity
            );
        }
        assert_eq!(ledger.usage(), (0, 0, 0));
    }
}

#[test]
fn shared_backing_retention_preparation_is_flat_in_unchanged_fields() {
    use worth_foundational::facade::AspectMask;
    let mut visits = Vec::new();
    for fields in [128, 256] {
        let artifact = payload_oracle::wide_struct(fields);
        let mut remaining = Some(usize::MAX);
        let quoted =
            super::super::retention::semantic_payload::artifact(&artifact, &mut remaining).unwrap();
        visits.push(usize::MAX - remaining.unwrap());
        let plan = BridgeConditionalSemanticObservationPlan::managed_test_plan_for(
            artifact.payload().contract().clone(),
            AspectMask::whole_aspect(),
        );
        let ledger =
            BridgeRetentionLedger::new(BridgeConditionalRetentionBudget::development()).unwrap();
        let retained = prepare_observations(
            &plan,
            &Default::default(),
            [Some(&artifact)].into_iter(),
            &ledger,
        )
        .unwrap();
        assert_eq!(retained.current(0), Some(&artifact));
        assert_eq!(ledger.usage().2, observation_bytes(quoted as usize, 0));
        drop(artifact);
        let worth_foundational::facade::ContractValidatedAspectValueView::Struct(value) =
            retained.current(0).unwrap().payload().view()
        else {
            panic!("real struct fixture");
        };
        assert_eq!(value.len(), fields);
    }
    assert!(visits[0] > 0, "the real owner admits its reference work");
    assert!(
        visits[1] * 2 <= visits[0] * 3,
        "retention quote work grows with unchanged fields: {visits:?}"
    );
}
