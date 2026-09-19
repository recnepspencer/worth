use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};

use super::super::ApplicationProgramManifest;

/// Canonicalization rule binding every revision digest to this exact framing.
const REVISION_RULE_VERSION: &str = "worth-query-application-program-revision-v1";

/// Basis domain separating program-revision meaning from every other canonical
/// sequence this crate mints.
const REVISION_BASIS_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.application-program-revision");

/// Canonical work one program revision is allowed to cost.
///
/// Meaning larger than this is denied while the program is validated, so a
/// program can never be identified by an unbounded hash nobody budgeted for.
const REVISION_WORK_BUDGET: CanonicalDigestWorkBudget =
    match CanonicalDigestWorkBudget::new(4_096, 1024 * 1024) {
        Some(budget) => budget,
        None => panic!("the declared application-program revision budget is nonzero"),
    };

/// Canonical content identity of one validated application program.
///
/// The revision is derived through Foundational's admitted canonical digest
/// slot over the authored program identity and the normalized manifest records,
/// so it is stable across Rust type renames and across declaration order
/// wherever the manifest is already normalized. It is descriptive content
/// identity only: it establishes that two validated programs declare the same
/// meaning, never which branch currently uses that meaning, whether a host
/// supports it, or that any caller may publish under it.
///
/// The `Contributions` tuple is deliberately outside the revision: it is the
/// Rust implementation a host links for the program, not declared meaning, and
/// it has no authored identity to canonicalize. Which implementation serves a
/// revision is the installed support binding's fact, so two programs that
/// declare the same meaning share a revision whatever code implements them.
///
/// Consumers name the type and read what it carries:
///
/// ```
/// use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
/// fn render(revision: &ApplicationProgramRevision) -> String {
///     format!("{revision} over {} bytes", revision.as_bytes().len())
/// }
/// ```
///
/// A revision is minted only while a program is validated, so the same path
/// cannot assemble one from bytes:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
/// let forged = ApplicationProgramRevision([0_u8; 32]);
/// ```
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
/// let forged = ApplicationProgramRevision::from([0_u8; 32]);
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationProgramRevision([u8; 32]);

impl ApplicationProgramRevision {
    /// Mints the canonical revision of already-validated program meaning
    /// through the Foundational canonical digest slot.
    ///
    /// Only `super::super::program` validation reaches this constructor, which
    /// is why a revision in hand always names meaning that passed declaration
    /// validation. Meaning that outgrows the declared canonical budget is
    /// refused here and denied there; it is never hashed silently.
    pub(in crate::application_program) fn mint(
        manifest: &ApplicationProgramManifest,
    ) -> Result<Self, ApplicationProgramRevisionBudgetDenial> {
        let basis = prepare_canonical_basis_sequence(
            revision_rule_version(),
            REVISION_BASIS_DOMAIN,
            revision_basis_entries(manifest),
        )
        .into_result()
        .expect("a program revision basis is one nonempty domain of unique named loci");
        let ready = canonicalization()
            .digest()
            .for_sequence_with_budget(
                basis,
                CanonicalDigestAlgorithmId::sha256(),
                REVISION_WORK_BUDGET,
            )
            .into_result()
            .map_err(exceeded_revision_budget)?;
        Ok(Self(
            *canonicalization().digest().derive(ready).value().bytes(),
        ))
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for ApplicationProgramRevision {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Refusal to identify declared program meaning that exceeds the canonical work
/// this crate spends on one revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::application_program) enum ApplicationProgramRevisionBudgetDenial {
    BasisEntries { maximum: u32, attempted: u32 },
    EncodedBytes { maximum: usize, attempted: usize },
}

impl std::fmt::Display for ApplicationProgramRevisionBudgetDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BasisEntries { maximum, attempted } => write!(
                formatter,
                "canonical revision basis entries {attempted} exceed the declared maximum {maximum}"
            ),
            Self::EncodedBytes { maximum, attempted } => write!(
                formatter,
                "canonical revision encoded bytes {attempted} exceed the declared maximum {maximum}"
            ),
        }
    }
}

/// Names the fixed canonical framing every program revision is derived under.
fn revision_rule_version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(REVISION_RULE_VERSION)
        .expect("the fixed application-program revision rule is valid")
}

/// Lays out the durable preimage: the authored program identity, how many
/// normalized records describe it, and each record at its own named locus.
fn revision_basis_entries(manifest: &ApplicationProgramManifest) -> Vec<CanonicalBasisEntry> {
    let mut entries = Vec::with_capacity(manifest.records().len().saturating_add(2));
    entries.push(revision_basis_entry(
        "program-identity".to_owned(),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::ExactText(manifest.program_identity().to_owned().into()),
    ));
    entries.push(revision_basis_entry(
        "record-count".to_owned(),
        CanonicalBasisEntryKind::Shape,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: manifest.records().len() as u128,
        },
    ));
    entries.extend(
        manifest
            .records()
            .iter()
            .enumerate()
            .map(|(index, record)| {
                revision_basis_entry(
                    format!("record[{index}]"),
                    CanonicalBasisEntryKind::Field,
                    CanonicalBasisValue::ExactText(record.clone().into()),
                )
            }),
    );
    entries
}

fn revision_basis_entry(
    locus: String,
    kind: CanonicalBasisEntryKind,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        REVISION_BASIS_DOMAIN,
        CanonicalBasisLocus::Named(locus.into()),
        kind,
        value,
    )
}

/// Names which declared canonical budget the presented meaning exceeded.
///
/// The remaining slot refusals describe an algorithm, rule version, input shape
/// or input domain that disagrees with the basis; this slot derives all four
/// from the basis it has just prepared, so they cannot occur here.
fn exceeded_revision_budget(
    denial: CanonicalDigestDerivationDenial,
) -> ApplicationProgramRevisionBudgetDenial {
    match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { maximum, actual } => {
            ApplicationProgramRevisionBudgetDenial::BasisEntries {
                maximum,
                attempted: actual,
            }
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { maximum, attempted } => {
            ApplicationProgramRevisionBudgetDenial::EncodedBytes { maximum, attempted }
        }
        CanonicalDigestDerivationDenial::UnsupportedAlgorithm
        | CanonicalDigestDerivationDenial::RuleVersionMismatch
        | CanonicalDigestDerivationDenial::InputShapeMismatch
        | CanonicalDigestDerivationDenial::InputDomainMismatch => unreachable!(
            "the revision digest slot is built from the basis sequence it just prepared"
        ),
    }
}
