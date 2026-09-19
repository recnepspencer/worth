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
    let retained = observation_bytes(2 * "balance".len(), available_mask_bytes());
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
    let successor_bytes = observation_bytes(4 * "balance".len(), available_mask_bytes());
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
fn observation_retention_charges_its_copy_without_the_sources_spare_capacity() {
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
    let expected = observation_bytes(2 * "balance".len() + "retained".len(), 0);
    assert_exact_boundary(&plan, &artifact, expected);
    let ledger = BridgeRetentionLedger::new(BridgeConditionalRetentionBudget {
        maximum_retained_bytes: expected,
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
    assert_eq!(retained.capacity(), "retained".len());
    assert_eq!(retained.as_str(), source.as_str());
}

fn assert_exact_boundary(
    plan: &BridgeConditionalSemanticObservationPlan,
    current: &ContractValidatedAspectArtifact,
    expected: u64,
) {
    for bytes in [expected, expected - 1] {
        let ledger = BridgeRetentionLedger::new(BridgeConditionalRetentionBudget {
            maximum_retained_bytes: bytes,
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
