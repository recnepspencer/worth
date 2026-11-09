//! Cold source accounting for real installed declarations. The complete fixture
//! uses ordinary admission; the oracle independently encodes its retained source.
mod all_families;
pub(crate) use all_families::declaration as all_families_declaration;
pub(crate) use families::required as required_families;
mod families;

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisEntry, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial as Denial, CanonicalDigestWorkBudget,
};

use crate::application_schema::WorthQueryInstalledApplicationSchema;

fn ceiling() -> CanonicalDigestWorkBudget {
    CanonicalDigestWorkBudget::for_encoded_byte_ceiling(
        crate::canonical_work::INSTALLATION_MAXIMUM_CANONICAL_BYTES,
    )
    .unwrap()
}

fn measure(
    basis: &CanonicalBasisReadyArtifact,
    budget: CanonicalDigestWorkBudget,
) -> Result<(u32, usize), Denial> {
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis.clone(), CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()?;
    let digest = canonicalization().digest().derive(ready);
    let work = digest.metadata().work();
    Ok((work.canonical_entry_count(), work.canonical_encoded_bytes()))
}

fn locus(entry: &CanonicalBasisEntry) -> &str {
    let CanonicalBasisLocus::Named(name) = entry.locus() else {
        panic!("schema source uses named canonical loci")
    };
    name.raw_text().unwrap()
}

fn subset(
    source: &CanonicalBasisReadyArtifact,
    retain: impl Fn(&str) -> bool,
) -> CanonicalBasisReadyArtifact {
    let payload = source.payload();
    prepare_canonical_basis_sequence(
        payload.version().clone(),
        payload.domain(),
        payload
            .entries()
            .iter()
            .filter(|entry| retain(locus(entry)))
            .cloned(),
    )
    .into_result()
    .unwrap()
}

fn assert_foundation_enforcement_pair(source: &CanonicalBasisReadyArtifact) {
    let (entries, bytes) = measure(source, ceiling()).unwrap();
    let short_entries = CanonicalDigestWorkBudget::new(entries - 1, bytes).unwrap();
    assert!(
        matches!(measure(source, short_entries), Err(Denial::EntryLimitExceeded {
        maximum, actual,
    }) if maximum == entries - 1 && actual == entries)
    );
    let short_bytes = CanonicalDigestWorkBudget::new(entries, bytes - 1).unwrap();
    assert!(
        matches!(measure(source, short_bytes), Err(Denial::EncodedByteLimitExceeded {
        maximum, attempted,
    }) if maximum == bytes - 1 && attempted == bytes)
    );
    assert_eq!(
        measure(
            source,
            CanonicalDigestWorkBudget::new(entries, bytes).unwrap()
        ),
        Ok((entries, bytes))
    );
}

pub(crate) fn assert_complete_source_budget<Schema>(
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
) -> BTreeSet<String> {
    assert_declaration_allowance(&installed.schema, installed.identity_budget)
}

pub(crate) fn assert_declaration_allowance(
    declaration: &worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration,
    allowance: CanonicalDigestWorkBudget,
) -> BTreeSet<String> {
    let source = declaration.identity().canonical_basis();
    let full = measure(source, ceiling()).unwrap();
    // This is a global derivation check, not attribution to the first family.
    assert_eq!(
        allowance.maximum_entry_count(),
        full.0 * 16,
        "global declaration entry allowance for {}",
        declaration.name()
    );
    assert_eq!(
        allowance.maximum_encoded_bytes(),
        full.1 * 16,
        "global declaration byte allowance for {}",
        declaration.name()
    );
    // Index the cold source once; the real fixtures include large declarations.
    let kind_terms: BTreeMap<_, _> = source
        .payload()
        .entries()
        .iter()
        .filter_map(|entry| {
            let prefix = locus(entry).strip_suffix(".kind")?;
            (prefix.starts_with("member[") && !prefix.contains('.')).then_some((prefix, entry))
        })
        .collect();
    let mut families = BTreeMap::<String, BTreeSet<String>>::new();
    for (index, member) in declaration.members().iter().enumerate() {
        let prefix = format!("member[{index}]");
        let kind = kind_terms
            .get(prefix.as_str())
            .unwrap_or_else(|| panic!("declared member {index} has no canonical family term"));
        let CanonicalBasisValue::ExactText(kind) = kind.value() else {
            panic!("the canonical family tag is text")
        };
        let family = families::canonical_family(member);
        assert_eq!(
            kind.raw_text().unwrap(),
            family,
            "canonical declaration family tag"
        );
        families.entry(family.into()).or_default().insert(prefix);
    }
    for (family, prefixes) in &families {
        let belongs = |name: &str| prefixes.contains(name.split('.').next().unwrap());
        let remaining = subset(source, |name| !belongs(name));
        let rest = measure(&remaining, ceiling()).unwrap();
        let term = (full.0 - rest.0, full.1 - rest.1);
        assert!(
            term.0 > 0 && term.1 > 0,
            "{family} must contribute canonical work"
        );
        // Keep shared framing in each counterfactual source; this is a cold
        // encoding oracle, not a fabricated admitted declaration or allowance.
        // These measured pairs check Foundation enforcement, not the derivation.
        let isolated = subset(source, |name| !name.starts_with("member[") || belongs(name));
        assert_foundation_enforcement_pair(&isolated);
        println!(
            "declaration-budget-family schema={} family={family} entries={} bytes={}",
            declaration.name(),
            term.0,
            term.1
        );
    }
    families.into_keys().collect()
}
