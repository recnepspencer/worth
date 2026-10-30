#[allow(unused_imports)]
use crate::certification::ordinary_context::canonicalization::basis::admitted_preparation::*;
pub(crate) use {
    worth_foundational::facade::prepare_owned_canonical_basis_sequence_admitted,
    worth_foundational::facade::FieldKey,
};

#[test]
fn refused_measurement_or_sorting_never_emits_ready_and_preserves_refusal() {
    for refused_call in [1, 4] {
        let mut calls = 0;
        let result = prepare_owned_canonical_basis_sequence_admitted(
            version(),
            worth_foundational::facade::CanonicalBasisDomain::Value,
            vec![entry(1), entry(0)],
            |_, _| {
                calls += 1;
                if calls == refused_call {
                    Err("original refusal")
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(
            result.unwrap_err(),
            worth_foundational::facade::CanonicalBasisPreparationStop::Resource {
                cause: "original refusal",
                ..
            }
        ));
        assert_eq!(calls, refused_call);
    }
}

#[test]
fn domain_rejection_precedes_measurement_and_sort_storage() {
    let mut calls = Vec::new();
    let result = prepare_owned_canonical_basis_sequence_admitted(
        version(),
        worth_foundational::facade::CanonicalBasisDomain::Identity,
        vec![entry(0)],
        |work, bytes| {
            calls.push((work, bytes));
            Ok::<_, ()>(())
        },
    );
    assert_eq!(
        result.unwrap_err(),
        worth_foundational::facade::CanonicalBasisPreparationStop::Construction(
            worth_foundational::facade::CanonicalBasisConstructionDenial::DomainMismatch {
                expected: worth_foundational::facade::CanonicalBasisDomain::Identity,
                actual: worth_foundational::facade::CanonicalBasisDomain::Value,
            },
        )
    );
    assert_eq!(calls, [(4, 0)]);
}

#[test]
fn duplicate_field_path_denial_retains_its_exact_owned_locus() {
    let locus = worth_foundational::facade::CanonicalBasisLocus::AspectField {
        aspect: worth_foundational::facade::AspectKey::new("bank.Account").unwrap(),
        path: worth_foundational::facade::CanonicalFieldPath::new([
            worth_foundational::facade::FieldKey::new("balance").unwrap(),
            worth_foundational::facade::FieldKey::new("currency").unwrap(),
        ])
        .unwrap(),
    };
    let entry = worth_foundational::facade::CanonicalBasisEntry::new(
        worth_foundational::facade::CanonicalBasisDomain::Value,
        locus.clone(),
        worth_foundational::facade::CanonicalBasisEntryKind::Field,
        worth_foundational::facade::CanonicalBasisValue::Bool(true),
    );
    let mut allocation = 0;
    let result = prepare_owned_canonical_basis_sequence_admitted(
        version(),
        worth_foundational::facade::CanonicalBasisDomain::Value,
        vec![entry.clone(), entry],
        |_, bytes| {
            allocation += bytes;
            Ok::<_, ()>(())
        },
    );
    assert_eq!(
        result.unwrap_err(),
        worth_foundational::facade::CanonicalBasisPreparationStop::Construction(
            worth_foundational::facade::CanonicalBasisConstructionDenial::DuplicateEntry {
                domain: worth_foundational::facade::CanonicalBasisDomain::Value,
                locus,
                kind: worth_foundational::facade::CanonicalBasisEntryKind::Field,
            },
        )
    );
    assert!(allocation >= "bank.Accountbalancecurrency".len() + 2 * size_of::<FieldKey>());
}
