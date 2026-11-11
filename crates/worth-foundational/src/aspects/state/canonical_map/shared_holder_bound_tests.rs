//! Each sharing holder's declared bound covers the observed shared backing.
use crate::aspects::*;
use crate::values::{AspectValue, ScalarAspectType};
use worth_proof::TransitionOutcome;

fn admitted(key: String) -> ContractValidatedAspectValue {
    let contract = AspectContract::scalar(
        AspectKey::new(key).unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::Bool,
    );
    let TransitionOutcome::Success(value) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::Bool(true)),
    ) else {
        panic!("canonical scalar")
    };
    value.into_parts().into_parts().0
}

#[test]
fn per_holder_bound_covers_backing_shared_by_two_holders_and_counts_it_twice() {
    for count in [0, 1, 31, 65, 127] {
        let ((first, second, live), _) = crate::allocation_observation::peak(|| {
            let first = AuthoritativeRecordAspectState::from_validated_entries(
                (0..count).map(|n| admitted(format!("key-{n:04}"))),
            )
            .unwrap();
            // The second holder shares the persistent roots and admitted values.
            let second = first.clone();
            let live = crate::allocation_observation::live_bytes();
            (first, second, live)
        });
        let bound = first.aspects().retained_storage_bytes().unwrap();
        assert_eq!(second.aspects().retained_storage_bytes(), Some(bound));
        assert!(live > 0, "even an empty state owns persistent roots");
        // One holder's declared bound already covers everything both reach.
        assert!(
            live <= bound,
            "count={count}, observed shared backing={live}, holder bound={bound}"
        );
        // Summing per holder therefore overbounds; it never undercounts.
        assert!(live < bound.checked_mul(2).unwrap());
        drop(first);
        assert_eq!(second.aspects().len(), count);
    }
}
