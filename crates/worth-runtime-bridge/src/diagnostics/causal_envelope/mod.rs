pub(crate) mod assembly;
mod authority;
mod binding;
mod counters;
mod denial;
mod digest_basis;
mod evidence_reference;
mod explanation_envelope;
mod identity;
mod receipt;
pub(crate) mod retained_mapping;
mod retained_source;

pub use assembly::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalInspectionAdmissionSummary,
    BridgeCausalInspectionAdmissionSummaryKind, BridgeConditionalCausalEnvelopeRequest,
};
pub use authority::{BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner};
pub use binding::{BridgeCausalEvidenceBinding, BridgeCausalEvidenceBindingClass};
pub use counters::BridgeCausalEnvelopeCounters;
pub use denial::{BridgeCausalEnvelopeDenial, BridgeCausalEnvelopeDenialKind};
pub use evidence_reference::{
    BridgeCausalEvidenceReference, BridgeCausalEvidenceReferenceIdentity,
};
pub use explanation_envelope::BridgeCausalExplanationEnvelope;
pub use identity::BridgeCausalEnvelopeIdentity;
pub use receipt::BridgeCausalEnvelopeReceipt;

use worth_foundational::facade::{
    derive_canonical_digest, prepare_canonical_basis_bundle, prepare_canonical_basis_sequence,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalDerivedDigest,
    CanonicalDigestAlgorithmId, CanonicalDigestFrontDoor, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

use crate::identity::BridgeIdentityEvidence;
use digest_basis::BridgeCausalEnvelopeDigestArtifact;

const BRIDGE_CAUSAL_ENVELOPE_IDENTITY_DOMAIN: &str =
    "worth_runtime_bridge.causal_envelope.identity";
const BRIDGE_CAUSAL_ENVELOPE_IDENTITY_SCHEME: &str =
    "worth.runtime.bridge.causal-envelope-identity.v1";

enum BridgeCausalEnvelopeIdentityPart<'a> {
    Evidence(&'a BridgeIdentityEvidence),
    Shape(&'a str),
    Value(&'a str),
}

fn evidence_part(identity: &BridgeIdentityEvidence) -> BridgeCausalEnvelopeIdentityPart<'_> {
    BridgeCausalEnvelopeIdentityPart::Evidence(identity)
}

fn shape_part(value: &str) -> BridgeCausalEnvelopeIdentityPart<'_> {
    BridgeCausalEnvelopeIdentityPart::Shape(value)
}

fn value_part(value: &str) -> BridgeCausalEnvelopeIdentityPart<'_> {
    BridgeCausalEnvelopeIdentityPart::Value(value)
}

fn compose_bridge_causal_envelope_evidence_identity(
    artifact: BridgeCausalEnvelopeDigestArtifact,
    parts: &[BridgeCausalEnvelopeIdentityPart<'_>],
) -> BridgeIdentityEvidence {
    let mut entries = vec![
        causal_envelope_entry(
            "bridge.causal.scheme",
            CanonicalBasisEntryKind::Header,
            BRIDGE_CAUSAL_ENVELOPE_IDENTITY_SCHEME,
        ),
        causal_envelope_entry(
            "bridge.causal.artifact",
            CanonicalBasisEntryKind::Header,
            artifact.digest_domain(),
        ),
    ];
    for (index, part) in parts.iter().enumerate() {
        let (kind, value) = match part {
            BridgeCausalEnvelopeIdentityPart::Evidence(identity) => ("identity", identity.as_str()),
            BridgeCausalEnvelopeIdentityPart::Shape(value) => ("shape", *value),
            BridgeCausalEnvelopeIdentityPart::Value(value) => ("value", *value),
        };
        let kind_locus = sequence_locus("kind", index);
        entries.push(causal_envelope_entry(
            kind_locus,
            CanonicalBasisEntryKind::Shape,
            kind,
        ));
        let value_locus = sequence_locus("value", index);
        entries.push(causal_envelope_entry(
            value_locus,
            canonical_entry_kind_for_part(part),
            value,
        ));
    }
    entries.push(causal_envelope_entry(
        "bridge.causal.part.count",
        CanonicalBasisEntryKind::Shape,
        parts.len().to_string(),
    ));
    let digest = derive_canonical_bridge_causal_digest(entries);
    BridgeIdentityEvidence::from_canonical_bridge_evidence(
        canonical_bridge_identity_token(&digest),
        artifact.digest_domain(),
    )
}

fn canonical_entry_kind_for_part(
    part: &BridgeCausalEnvelopeIdentityPart<'_>,
) -> CanonicalBasisEntryKind {
    match part {
        BridgeCausalEnvelopeIdentityPart::Evidence(_) => CanonicalBasisEntryKind::Identity,
        BridgeCausalEnvelopeIdentityPart::Shape(_) => CanonicalBasisEntryKind::Shape,
        BridgeCausalEnvelopeIdentityPart::Value(_) => CanonicalBasisEntryKind::Value,
    }
}

fn sequence_locus(kind: &str, index: usize) -> String {
    let mut locus = String::from("bridge.causal.part.");
    locus.push_str(&index.to_string());
    locus.push('.');
    locus.push_str(kind);
    locus
}

fn causal_envelope_entry(
    locus: impl Into<String>,
    kind: CanonicalBasisEntryKind,
    value: impl Into<String>,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Future(BRIDGE_CAUSAL_ENVELOPE_IDENTITY_DOMAIN),
        CanonicalBasisLocus::Named(locus.into().into()),
        kind,
        CanonicalBasisValue::ExactText(value.into().into()),
    )
}

fn derive_canonical_bridge_causal_digest(
    entries: Vec<CanonicalBasisEntry>,
) -> CanonicalDerivedDigest {
    let version = CanonicalizationRuleVersion::new(BRIDGE_CAUSAL_ENVELOPE_IDENTITY_SCHEME)
        .expect("bridge causal envelope evidence scheme must remain canonical");
    let ready = causal_basis_from_entries(version.clone(), entries);
    let bundle = match prepare_canonical_basis_bundle(version, [ready]) {
        TransitionOutcome::Success(bundle) => bundle,
        outcome => {
            panic!("bridge causal envelope identity bundle should prepare cleanly: {outcome:?}")
        }
    };
    let digest_ready = match CanonicalDigestFrontDoor
        .for_bundle(bundle, CanonicalDigestAlgorithmId::sha256())
    {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("bridge causal envelope digest derivation should succeed: {outcome:?}"),
    };
    derive_canonical_digest(digest_ready)
}

fn causal_basis_from_entries(
    version: CanonicalizationRuleVersion,
    entries: Vec<CanonicalBasisEntry>,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::Future(BRIDGE_CAUSAL_ENVELOPE_IDENTITY_DOMAIN),
        entries,
    ) {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("bridge causal envelope basis should prepare cleanly: {outcome:?}"),
    }
}

fn canonical_bridge_identity_token(digest: &CanonicalDerivedDigest) -> String {
    use std::fmt::Write;

    let mut token = String::from(BRIDGE_CAUSAL_ENVELOPE_IDENTITY_SCHEME);
    token.push(':');
    token.push_str(digest.metadata().algorithm().id().as_str());
    token.push(':');
    for byte in digest.value().bytes() {
        write!(&mut token, "{byte:02x}").expect("writing to String cannot fail");
    }
    token
}
