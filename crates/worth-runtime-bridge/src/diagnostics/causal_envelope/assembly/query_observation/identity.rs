//! An additive reporting grammar, funded through the canonical authority owner.
use crate::identity::BridgeIdentityEvidence;
use std::{alloc::Layout, fmt::Write, sync::atomic::AtomicUsize};
use worth_foundational::facade::{
    prepare_owned_canonical_basis_sequence_admitted, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisPreparationStop,
    CanonicalBasisValue, CanonicalDigestAdmissionStop, CanonicalDigestAlgorithmId,
    CanonicalDigestFrontDoor, CanonicalDigestWorkBudget, CanonicalizationRuleVersion,
};

const SCHEME: &str = "worth.runtime.bridge.query-observation-report.v1";

#[derive(Clone, Copy)]
pub(super) enum ReportIdentityKind {
    AdmissionSummary,
    EvidenceReference,
    AssemblyRequest,
    EvidenceBinding,
    Counters,
    BindingSet,
    Identity,
    ExplanationEnvelope,
    Receipt,
}

impl ReportIdentityKind {
    fn domain(self) -> &'static str {
        match self {
            Self::AdmissionSummary => "query-observation-report.admission-summary.v1",
            Self::EvidenceReference => "query-observation-report.evidence-reference.v1",
            Self::AssemblyRequest => "query-observation-report.assembly-request.v1",
            Self::EvidenceBinding => "query-observation-report.evidence-binding.v1",
            Self::Counters => "query-observation-report.counters.v1",
            Self::BindingSet => "query-observation-report.binding-set.v1",
            Self::Identity => "query-observation-report.identity.v1",
            Self::ExplanationEnvelope => "query-observation-report.envelope.v1",
            Self::Receipt => "query-observation-report.receipt.v1",
        }
    }
}

pub(super) enum ReportPart<'a> {
    Evidence(&'a BridgeIdentityEvidence),
    Shape(&'a str),
    Value(&'a str),
}

impl ReportPart<'_> {
    fn borrowed(&self) -> (CanonicalBasisEntryKind, &str) {
        match self {
            Self::Evidence(value) => (CanonicalBasisEntryKind::Identity, value.as_str()),
            Self::Shape(value) => (CanonicalBasisEntryKind::Shape, value),
            Self::Value(value) => (CanonicalBasisEntryKind::Value, value),
        }
    }
}

/// Resource causes stay distinct from canonical grammar and compiler support.
#[derive(Debug)]
pub enum BridgeQueryObservationReportingStop<Stop> {
    Resource(Stop),
    Basis(CanonicalBasisPreparationStop<Stop>),
    Digest(CanonicalDigestAdmissionStop<Stop>),
    AccountingOverflow,
    InvalidReportingAssociation,
}

pub(super) fn compose<Stop>(
    kind: ReportIdentityKind,
    parts: &[ReportPart<'_>],
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<BridgeIdentityEvidence, BridgeQueryObservationReportingStop<Stop>> {
    use BridgeQueryObservationReportingStop as Failure;
    let count = u32::try_from(parts.len()).map_err(|_| Failure::AccountingOverflow)?;
    let strings = parts
        .iter()
        .try_fold(SCHEME.len() + "sha256".len(), |bytes, part| {
            bytes.checked_add(part.borrowed().1.len())
        })
        .ok_or(Failure::AccountingOverflow)?;
    let bytes = parts
        .len()
        .checked_mul(size_of::<CanonicalBasisEntry>())
        .and_then(|bytes| bytes.checked_add(strings))
        .ok_or(Failure::AccountingOverflow)?;
    let work = bytes
        .checked_add(parts.len())
        .ok_or(Failure::AccountingOverflow)?;
    // The entry ordinal is inline. All copied values, the entry array, rule
    // version and algorithm name are admitted before their constructors run.
    admit(work, bytes).map_err(Failure::Resource)?;
    let domain = CanonicalBasisDomain::Future(kind.domain());
    let mut entries = Vec::with_capacity(parts.len());
    for (ordinal, part) in (0..count).zip(parts) {
        let (kind, value) = part.borrowed();
        entries.push(CanonicalBasisEntry::new(
            domain,
            CanonicalBasisLocus::EntryOrdinal(ordinal),
            kind,
            CanonicalBasisValue::ExactText(value.to_owned().into()),
        ));
    }
    let version = CanonicalizationRuleVersion::new(SCHEME)
        .expect("the fixed query reporting scheme is canonical");
    let sequence =
        prepare_owned_canonical_basis_sequence_admitted(version, domain, entries, &mut *admit)
            .map_err(Failure::Basis)?;
    let ready = CanonicalDigestFrontDoor
        .for_sequence_with_admission(
            sequence,
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalDigestWorkBudget::standard(),
            admit,
        )
        .map_err(Failure::Digest)?;
    let digest = CanonicalDigestFrontDoor
        .derive_with_admission(ready, admit)
        .map_err(Failure::Digest)?;
    let length = SCHEME
        .len()
        .checked_add(1 + "sha256".len() + 1 + 64)
        .ok_or(Failure::AccountingOverflow)?;
    let arc = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::array::<u8>(length).map_err(|_| Failure::AccountingOverflow)?)
        .map_err(|_| Failure::AccountingOverflow)?
        .0
        .pad_to_align()
        .size();
    let output = length.checked_add(arc).ok_or(Failure::AccountingOverflow)?;
    admit(length, output).map_err(Failure::Resource)?;
    let mut token = String::with_capacity(length);
    token.push_str(SCHEME);
    token.push_str(":sha256:");
    for byte in digest.value().bytes() {
        write!(&mut token, "{byte:02x}").expect("fixed hex writes to admitted String");
    }
    Ok(BridgeIdentityEvidence::from_canonical_bridge_evidence(
        token,
        kind.domain(),
    ))
}
