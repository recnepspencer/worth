use worth_foundational::facade::{
    canonical_basis_sequence_material, prepare_canonical_basis_sequence, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalizationRuleVersion,
};

pub fn canonical_operation_material(entries: Vec<(&str, String)>) -> String {
    canonical_owned_operation_material(
        entries
            .into_iter()
            .map(|(locus, value)| (locus.to_owned(), value)),
    )
}

pub fn canonical_indexed_operation_material(
    locus: &str,
    values: impl IntoIterator<Item = String>,
) -> String {
    let values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return canonical_owned_operation_material([(
            format!("{locus}.empty"),
            "explicitly-empty".into(),
        )]);
    }
    canonical_owned_operation_material(
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| (format!("{locus}.{index}"), value)),
    )
}

fn canonical_owned_operation_material(
    entries: impl IntoIterator<Item = (String, String)>,
) -> String {
    let entries = entries.into_iter().map(|(locus, value)| {
        CanonicalBasisEntry::new(
            CanonicalBasisDomain::Future("query-operation-identity"),
            CanonicalBasisLocus::Named(locus.into()),
            CanonicalBasisEntryKind::Value,
            CanonicalBasisValue::ExactText(value.into()),
        )
    });
    let ready = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("query-operation-identity-v1")
            .expect("static canonicalization rule version is valid"),
        CanonicalBasisDomain::Future("query-operation-identity"),
        entries,
    )
    .into_result()
    .expect("Query operation identity basis is structurally canonical");
    canonical_basis_sequence_material(ready.payload())
}
