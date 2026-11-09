use super::*;
use worth_foundational::facade::{
    prepare_aspect_contract_for_canonical_basis, AbsenceLaw, AspectContract,
    AspectContractRevision, AspectEvolutionPolicy, AspectIdentity, AspectKey,
    CanonicalDigestWorkBudget, CanonicalizationRuleVersion, FieldDeclaration, FieldKey,
    FieldRequirement, ScalarAspectType, StructAspectShape,
};

fn contract(revision: u64) -> CanonicalBasisReadyArtifact {
    let fields = ["a", "a|b", "b"].map(|name| {
        FieldDeclaration::new(
            FieldKey::new(name).unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
    });
    prepare_aspect_contract_for_canonical_basis(
        CanonicalizationRuleVersion::new("worth-query-native-contract-v1").unwrap(),
        AspectContract::struct_aspect(
            AspectKey::new("Facts").unwrap(),
            AspectIdentity(37),
            AspectContractRevision(revision),
            StructAspectShape::new(fields).unwrap(),
        ),
    )
    .into_result()
    .unwrap()
}

fn basis() -> InstallationCanonicalIdentityBasis {
    InstallationCanonicalIdentityBasis::new(
        "worth-query.installed-graph-obligations",
        super::super::RULE_VERSION,
        CanonicalDigestWorkBudget::new(4096, 1024 * 1024).unwrap(),
    )
}

#[test]
fn tampered_contract_reference_refuses_instead_of_minting_an_identity() {
    let first = contract(1);
    let second = contract(2);
    for tampered_slot in [1, usize::MAX] {
        let mut table = AspectContractTable::default();
        let mut basis = basis();
        table
            .encode_reference(&mut basis, "projection[0].contract", &first)
            .unwrap();
        table
            .encode_reference(&mut basis, "projection[1].contract", &second)
            .unwrap();
        table
            .sources
            .insert(std::ptr::from_ref(&first), tampered_slot);
        assert_eq!(
            table.encode_reference(&mut basis, "projection[2].contract", &first),
            Err(CanonicalDigestDerivationDenial::InputShapeMismatch),
        );
    }
}

#[test]
fn references_bind_each_occurrence_even_when_first_use_ordinals_are_equal() {
    let first = contract(1);
    let second = contract(2);
    let equal_allocation = contract(1);
    let encode = |contracts: [&CanonicalBasisReadyArtifact; 3]| {
        let mut table = AspectContractTable::default();
        let mut basis = basis();
        for (index, contract) in contracts.into_iter().enumerate() {
            table
                .encode_reference(
                    &mut basis,
                    &format!("projection[{index}].contract"),
                    contract,
                )
                .unwrap();
        }
        basis.derive().unwrap().0
    };
    let original = encode([&first, &second, &first]);
    assert_eq!(original, encode([&first, &second, &first]));
    assert_ne!(original, encode([&first, &second, &second]));
    assert_ne!(original, encode([&second, &first, &second]));
    assert_eq!(
        encode([&first, &first, &first]),
        encode([&first, &equal_allocation, &first]),
        "allocation identity must never change canonical sharing",
    );
}

mod cost {
    use super::*;

    #[test]
    fn repeated_projections_pay_references_while_distinct_contracts_pay_their_facts() {
        let contracts: Vec<_> = (1..=64).map(contract).collect();
        let measure = |projections: usize, distinct: usize| {
            let mut table = AspectContractTable::default();
            let mut basis = basis();
            for index in 0..projections {
                table
                    .encode_reference(
                        &mut basis,
                        &format!("projection[{index}].contract"),
                        &contracts[index % distinct],
                    )
                    .unwrap();
            }
            let work = basis.derive().unwrap().1;
            println!(
                "projections={projections} distinct={distinct} entries={} bytes={}",
                work.canonical_entries(),
                work.canonical_encoded_bytes()
            );
            work.canonical_entries()
        };
        let shared = measure(64, 1);
        let repeated = measure(256, 1);
        let distinct = measure(64, 64);
        assert!(repeated < shared * 4);
        assert!(distinct > shared * 8);
        assert!(
            repeated - shared <= 256,
            "extra projections cannot re-embed contract facts"
        );
    }
}
