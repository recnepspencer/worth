//! Cold source accounting for real installed declarations, without 16× slack
//! masking a missing family. No installed authority is constructed here.
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

fn exact_admission_pair(source: &CanonicalBasisReadyArtifact) {
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
    let mut families = BTreeMap::<String, BTreeSet<String>>::new();
    for index in 0..declaration.members().len() {
        let prefix = format!("member[{index}]");
        let kind_locus = format!("{prefix}.kind");
        let kind = source
            .payload()
            .entries()
            .iter()
            .find(|entry| locus(entry) == kind_locus)
            .unwrap_or_else(|| panic!("declared member {index} has no canonical family term"));
        let CanonicalBasisValue::ExactText(kind) = kind.value() else {
            panic!("the canonical family tag is text")
        };
        families
            .entry(kind.raw_text().unwrap().into())
            .or_default()
            .insert(prefix);
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
        // Subtract the independently measured complement. A missing minor
        // family or even one uncounted entry/byte cannot hide in total slack.
        assert_eq!(
            allowance.maximum_entry_count().checked_sub(rest.0 * 16),
            Some(term.0 * 16),
            "{family} entry contribution"
        );
        assert_eq!(
            allowance.maximum_encoded_bytes().checked_sub(rest.1 * 16),
            Some(term.1 * 16),
            "{family} byte contribution"
        );
        // Keep shared framing in each counterfactual source; this is a cold
        // encoding oracle, not a fabricated admitted declaration or allowance.
        let isolated = subset(source, |name| !name.starts_with("member[") || belongs(name));
        exact_admission_pair(&isolated);
        println!(
            "declaration-budget-family schema={} family={family} entries={} bytes={}",
            declaration.name(),
            term.0,
            term.1
        );
    }
    families.into_keys().collect()
}
