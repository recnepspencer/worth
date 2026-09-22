//! The revision carries nothing but Foundational's canonical digest of the
//! declared program identity and its normalized records.
//!
//! The framing is restated here instead of imported, so the proof fails if the
//! minted preimage ever drifts from the declared canonical slot.

use std::fmt::Write;

use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestWorkBudget, CanonicalIntegerWidth, CanonicalizationRuleVersion,
};

use super::{manifest_of, revision_of, BaselineProgram, ConnectedTopologyProgram};
use crate::application_program::ApplicationProgramManifest;

#[test]
fn a_minted_revision_equals_the_canonical_slot_digest_of_its_manifest() {
    assert_eq!(
        revision_of::<BaselineProgram>().to_string(),
        independently_derived_digest(&manifest_of::<BaselineProgram>())
    );
    assert_eq!(
        revision_of::<ConnectedTopologyProgram>().to_string(),
        independently_derived_digest(&manifest_of::<ConnectedTopologyProgram>())
    );
}

#[test]
fn the_independent_reconstruction_separates_two_different_programs() {
    assert_ne!(
        independently_derived_digest(&manifest_of::<BaselineProgram>()),
        independently_derived_digest(&manifest_of::<ConnectedTopologyProgram>())
    );
}

/// Derives the same digest the crate publishes, from the manifest alone,
/// through the Foundational canonical slot and nothing else.
fn independently_derived_digest(manifest: &ApplicationProgramManifest) -> String {
    let domain = CanonicalBasisDomain::Future("worth-query.application-program-revision");
    let mut entries = vec![
        CanonicalBasisEntry::new(
            domain,
            CanonicalBasisLocus::Named("program-identity".to_owned().into()),
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::ExactText(manifest.program_identity().to_owned().into()),
        ),
        CanonicalBasisEntry::new(
            domain,
            CanonicalBasisLocus::Named("record-count".to_owned().into()),
            CanonicalBasisEntryKind::Shape,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: manifest.records().len() as u128,
            },
        ),
    ];
    for (index, record) in manifest.records().iter().enumerate() {
        entries.push(CanonicalBasisEntry::new(
            domain,
            CanonicalBasisLocus::Named(format!("record[{index}]").into()),
            CanonicalBasisEntryKind::Field,
            CanonicalBasisValue::ExactText(record.clone().into()),
        ));
    }
    let basis = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("worth-query-application-program-revision-v1")
            .expect("the restated revision rule version is valid"),
        domain,
        entries,
    )
    .into_result()
    .expect("the restated revision basis is a nonempty sequence of unique named loci");
    let budget = CanonicalDigestWorkBudget::new(4_096, 1024 * 1024)
        .expect("the restated revision budget is nonzero");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()
        .expect("the fixture programs fit the declared revision budget");
    let derived = canonicalization().digest().derive(ready);
    let mut rendered = String::with_capacity(64);
    for byte in derived.value().bytes() {
        write!(rendered, "{byte:02x}").expect("writing to a String cannot fail");
    }
    rendered
}
