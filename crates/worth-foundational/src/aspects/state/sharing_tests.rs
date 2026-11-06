use super::*;
use crate::aspects::patches::AuthoritativeRecordAspectPatch;
use crate::aspects::validation::{validate_aspect_value, ContractValidationInput};
use crate::aspects::{AspectContract, AspectContractRevision, AspectIdentity, AspectKey};
use crate::values::{AspectValue, ScalarAspectType};
use worth_proof::TransitionOutcome;

fn entry(index: usize, value: i64) -> crate::aspects::validation::ContractValidatedAspectArtifact {
    let contract = AspectContract::scalar(
        AspectKey::new(format!("aspect-{index:05}")).unwrap(),
        AspectIdentity(index as u64),
        AspectContractRevision(1),
        ScalarAspectType::Int64,
    );
    let TransitionOutcome::Success(entry) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::Int64(value)),
    ) else {
        panic!("valid scalar")
    };
    entry
}

#[test]
fn fixed_patch_shares_untouched_state_and_preserves_the_prior_snapshot() {
    for size in [128, 256, 512] {
        let entries = (0..size).map(|i| entry(i, i as i64).into_parts().into_parts().0);
        let state = AuthoritativeRecordAspectState::from_validated_entries(entries).unwrap();
        let cloned = state.aspects().cloned_entries();
        assert!(cloned.ptr_eq(&state.aspects().entries));
        let TransitionOutcome::Success(patch) =
            AuthoritativeRecordAspectPatch::whole_aspect([entry(0, -1)], [])
        else {
            panic!("valid patch")
        };
        let TransitionOutcome::Success(after) = patch.apply_to(&state) else {
            panic!("valid application")
        };
        for (key, before) in state.aspects().entries().skip(1) {
            assert!(std::ptr::eq(
                before.contract().shape(),
                after.payload().get(key).unwrap().contract().shape()
            ));
        }
        assert_eq!(
            state
                .get(&AspectKey::new("aspect-00000").unwrap())
                .unwrap()
                .view(),
            crate::aspects::validation::ContractValidatedAspectValueView::Scalar(
                &AspectValue::Int64(0)
            )
        );
        assert_eq!(
            after
                .payload()
                .get(&AspectKey::new("aspect-00000").unwrap())
                .unwrap()
                .view(),
            crate::aspects::validation::ContractValidatedAspectValueView::Scalar(
                &AspectValue::Int64(-1)
            )
        );
    }
}

mod cost {
    use super::*;

    #[test]
    fn one_aspect_patch_work_stays_flat_as_the_record_grows() {
        fn work(size: usize) -> usize {
            let entries = (0..size).map(|i| entry(i, i as i64).into_parts().into_parts().0);
            let state = AuthoritativeRecordAspectState::from_validated_entries(entries).unwrap();
            let TransitionOutcome::Success(patch) =
                AuthoritativeRecordAspectPatch::whole_aspect([entry(0, -1)], [])
            else {
                panic!("valid patch")
            };
            let observation = crate::aspects::validation::CloneObservation::start();
            let TransitionOutcome::Success(_) = patch.apply_to(&state) else {
                panic!("valid application")
            };
            observation.work()
        }
        let baseline = work(128);
        for actual in [work(256), work(512)] {
            assert!(
                (actual as f64) / (baseline as f64) < 1.5,
                "whole-record clone growth: {baseline} -> {actual}"
            );
        }
    }
}
