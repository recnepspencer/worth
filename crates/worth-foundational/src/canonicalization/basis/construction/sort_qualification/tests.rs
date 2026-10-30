use super::*;
use crate::facade::{
    prepare_owned_canonical_basis_sequence_admitted, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisPreparationStop,
    CanonicalBasisValue, CanonicalizationRuleVersion,
};

fn prepare<Stop>(
    admit: impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<crate::facade::CanonicalBasisReadyArtifact, CanonicalBasisPreparationStop<Stop>> {
    prepare_owned_canonical_basis_sequence_admitted(
        CanonicalizationRuleVersion::new("sort-qualification.v1").unwrap(),
        CanonicalBasisDomain::Value,
        vec![CanonicalBasisEntry::new(
            CanonicalBasisDomain::Value,
            CanonicalBasisLocus::Named("field".into()),
            CanonicalBasisEntryKind::Field,
            CanonicalBasisValue::ExactText("value".into()),
        )],
        admit,
    )
}

#[test]
fn refused_sort_qualification_keeps_actual_context_and_restores_nested_unwinding() {
    let original = require_qualified_sort();
    with_refused_canonical_sort_qualification_for_test(|| {
        let CanonicalBasisPreparationStop::UnsupportedSortImplementation(cause) =
            prepare(|_, _| -> Result<(), ()> { panic!("qualification precedes preparation") })
                .unwrap_err()
        else {
            panic!("the deliberate qualification refusal is not resource exhaustion")
        };
        assert_eq!(
            cause.compiler_release(),
            env!("WORTH_CANONICAL_SORT_COMPILER_RELEASE")
        );
        assert_eq!(
            cause.compiler_commit(),
            env!("WORTH_CANONICAL_SORT_COMPILER_COMMIT")
        );
        assert_eq!(cause.target_pointer_width(), usize::BITS);
        let nested = std::panic::catch_unwind(|| {
            with_refused_canonical_sort_qualification_for_test(|| panic!("scope unwind"));
        });
        assert!(nested.is_err());
        assert_eq!(require_qualified_sort(), Err(cause));
    });
    assert_eq!(require_qualified_sort(), original);
}

#[test]
#[cfg(canonical_verified_std_sort)]
fn actual_qualified_compiler_prepares_after_refusal_and_keeps_request_refusal_distinct() {
    with_refused_canonical_sort_qualification_for_test(|| {
        assert!(prepare(|_, _| Ok::<_, ()>(())).is_err())
    });
    assert_eq!(
        prepare(|_, _| Ok::<_, ()>(()))
            .unwrap()
            .payload()
            .entries()
            .len(),
        1
    );
    let stopped = prepare(|_, _| Err("original request refusal"));
    assert!(matches!(
        stopped,
        Err(CanonicalBasisPreparationStop::Resource {
            cause: "original request refusal",
            ..
        })
    ));
}
