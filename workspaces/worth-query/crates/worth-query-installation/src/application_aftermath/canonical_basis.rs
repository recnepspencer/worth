//! Foundational canonical basis for installed aftermath contracts.

use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalDigestAlgorithmId, CanonicalDigestDerivationDenial, CanonicalDigestId,
    CanonicalDigestWorkBudget, CanonicalizationRuleVersion,
};
use worth_query_declaration::facade::application_aftermath::{
    PortableApplicationAftermathContract, PortableCorrectionMechanism,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use super::denial::{
    WorthQueryAftermathInstallationDenial, WorthQueryAftermathInstallationDenialKind,
};
use super::external_effect_contract::InstalledExternalEffectContract;
use super::reconciliation::WorthQueryInstalledReconciliationProcedure;
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.application-aftermath-installation");
const RULE_VERSION: &str = "worth-query-application-aftermath-installation-v2";
const MAXIMUM_ENTRY_COUNT: u32 = 128;
const MAXIMUM_CANONICAL_BYTES: usize = 32 * 1_024;
const AFTERMATH_BUDGET: CanonicalDigestWorkBudget =
    match CanonicalDigestWorkBudget::new(MAXIMUM_ENTRY_COUNT, MAXIMUM_CANONICAL_BYTES) {
        Some(budget) => budget,
        None => panic!("fixed aftermath canonical-work budget is valid"),
    };

/// Canonical artifact retained inside an installed aftermath identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAftermathCanonicalArtifact {
    basis: CanonicalBasisReadyArtifact,
    digest: CanonicalDigestId,
    work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryAftermathCanonicalArtifact {
    pub fn basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.basis
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub const fn work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.work
    }

    pub const fn basis_preparation_count(&self) -> usize {
        self.work.basis_preparations() as usize
    }

    pub const fn digest_derivation_count(&self) -> usize {
        self.work.digest_derivations() as usize
    }

    pub const fn digest_text_materializations(&self) -> usize {
        self.work.digest_text_materializations() as usize
    }
}

/// Prepares the canonical basis of one installed aftermath contract.
///
/// Package and schema identity are read off the binding rather than accepted
/// beside it, so no caller can pair one operation's slot with another's package.
///
/// `external_effect` is the operation's own installed lane, not an aftermath
/// axis. It contributes here so the installed aftermath identity moves whenever
/// the escaping lane moves — including on the payload type and byte bound, which
/// the declared aftermath contract never carried at all.
pub(super) fn prepare_aftermath_basis(
    binding: &ApplicationSchemaBindingIdentity,
    operation_slot: &str,
    portable: &PortableApplicationAftermathContract,
    reconciliation: Option<&WorthQueryInstalledReconciliationProcedure>,
    external_effect: &InstalledExternalEffectContract,
) -> Result<WorthQueryAftermathCanonicalArtifact, WorthQueryAftermathInstallationDenial> {
    let mut builder = AftermathBasisBuilder::new(operation_slot);
    builder.text("family", "installed-aftermath");
    builder.digest("package", binding.package_identity());
    builder.digest("schema-or-domain", binding.schema_identity());
    builder.text("operation", operation_slot);
    builder.text("authority", authority_label(portable.authority()));
    push_correction_contract(&mut builder, portable, reconciliation);
    push_external_effect(&mut builder, external_effect);
    builder.finish()
}

fn push_correction_contract(
    builder: &mut AftermathBasisBuilder,
    portable: &PortableApplicationAftermathContract,
    reconciliation: Option<&WorthQueryInstalledReconciliationProcedure>,
) {
    match portable.mechanism() {
        Some(PortableCorrectionMechanism::RecordedInverse(inverse)) => {
            builder.text("mechanism", "recorded-inverse");
            builder.text("inverse-operation", inverse.inverse_operation_slot());
            builder.text(
                "lowering-correspondence",
                inverse.lowering_correspondence().correspondence_slot(),
            );
            builder.text(
                "preimage-byte-bound",
                inverse
                    .preimage_demand()
                    .maximum_encoded_bytes()
                    .to_string(),
            );
            for (index, locus) in inverse.preimage_demand().loci().iter().enumerate() {
                builder.text(format!("preimage-entity-{index}"), locus.entity());
                builder.text(format!("preimage-aspect-{index}"), locus.aspect());
                builder.text(format!("preimage-field-{index}"), locus.field());
            }
            push_postcondition(builder, inverse.postcondition());
        }
        Some(PortableCorrectionMechanism::Compensation(compensation)) => {
            builder.text("mechanism", "compensation");
            builder.text(
                "compensating-operation",
                compensation.compensating_operation_slot(),
            );
            push_postcondition(builder, compensation.postcondition());
        }
        None => {
            builder.text("mechanism", "none");
        }
    }
    if let Some(reconciliation) = reconciliation {
        builder.text("reconciliation", reconciliation.procedure_slot());
    } else {
        builder.text("reconciliation", "none");
    }
}

fn push_external_effect(
    builder: &mut AftermathBasisBuilder,
    external_effect: &InstalledExternalEffectContract,
) {
    match external_effect {
        InstalledExternalEffectContract::None => builder.text("external-effect", "none"),
        InstalledExternalEffectContract::Declared {
            correlation_family,
            effect,
            rust_payload_type,
            protocol,
            maximum_payload_bytes,
            inbound,
        } => {
            builder.text("external-effect", "declared");
            builder.text("external-correlation", correlation_family.as_str());
            builder.text("external-emission", effect);
            builder.text("external-rust-payload-type", rust_payload_type.as_str());
            builder.text("external-protocol-identity", protocol.identity().as_str());
            builder.u64(
                "external-protocol-version",
                u64::from(protocol.version().get()),
            );
            builder.u64("external-maximum-payload-bytes", *maximum_payload_bytes);
            if let Some(inbound) = inbound {
                builder.text("inbound-effect", inbound.effect());
                builder.text(
                    "inbound-protocol-identity",
                    inbound.protocol().identity().as_str(),
                );
                builder.u64(
                    "inbound-protocol-version",
                    u64::from(inbound.protocol().version().get()),
                );
                builder.text("inbound-source-identity", inbound.source_identity());
                let limits = inbound.limits();
                builder.u64(
                    "inbound-maximum-envelope-bytes",
                    limits.maximum_envelope_bytes.get(),
                );
                builder.u64(
                    "inbound-maximum-payload-bytes",
                    limits.maximum_payload_bytes.get(),
                );
                builder.u64(
                    "inbound-maximum-outstanding-dispatch-provenance",
                    limits.maximum_outstanding_dispatch_provenance.get(),
                );
                builder.u64(
                    "inbound-maximum-accepted-occurrences",
                    limits.maximum_accepted_occurrences.get(),
                );
                builder.u64(
                    "inbound-maximum-accepted-bytes",
                    limits.maximum_accepted_bytes.get(),
                );
                builder.u64(
                    "inbound-maximum-concurrent-publications",
                    limits.maximum_concurrent_publications.get(),
                );
                builder.u64(
                    "inbound-maximum-discovery-work",
                    limits.maximum_discovery_work.get(),
                );
                builder.u64(
                    "inbound-replay-window-milliseconds",
                    limits.replay_window_milliseconds.get(),
                );
                builder.u64(
                    "inbound-maximum-cleanup-work",
                    limits.maximum_cleanup_work.get(),
                );
            }
        }
    }
}

struct AftermathBasisBuilder {
    subject: String,
    entries: Vec<CanonicalBasisEntry>,
}

impl AftermathBasisBuilder {
    fn new(subject: &str) -> Self {
        Self {
            subject: subject.to_owned(),
            entries: Vec::with_capacity(32),
        }
    }

    fn text(&mut self, locus: impl Into<String>, value: impl AsRef<str>) {
        self.value(
            locus,
            CanonicalBasisValue::ExactText(value.as_ref().to_owned().into()),
        );
    }

    fn digest(&mut self, locus: impl Into<String>, value: &CanonicalDigestId) {
        self.value(locus, CanonicalBasisValue::BytesDigest(*value));
    }

    fn u64(&mut self, locus: impl Into<String>, value: u64) {
        self.text(locus, value.to_string());
    }

    fn value(&mut self, locus: impl Into<String>, value: CanonicalBasisValue) {
        self.entries.push(CanonicalBasisEntry::new(
            DOMAIN,
            CanonicalBasisLocus::Named(locus.into().into()),
            CanonicalBasisEntryKind::Identity,
            value,
        ));
    }

    fn finish(
        self,
    ) -> Result<WorthQueryAftermathCanonicalArtifact, WorthQueryAftermathInstallationDenial> {
        let version = CanonicalizationRuleVersion::new(RULE_VERSION)
            .expect("the installed aftermath rule is valid");
        let basis = prepare_canonical_basis_sequence(version, DOMAIN, self.entries)
            .into_result()
            .expect("installed aftermath basis loci are unique and typed");
        let ready = canonicalization()
            .digest()
            .for_sequence_with_budget(
                basis.clone(),
                CanonicalDigestAlgorithmId::sha256(),
                AFTERMATH_BUDGET,
            )
            .into_result()
            .map_err(|denial| canonical_denial(&self.subject, denial))?;
        let derived = canonicalization().digest().derive(ready);
        Ok(WorthQueryAftermathCanonicalArtifact {
            basis,
            digest: CanonicalDigestId::new(*derived.value().bytes()),
            work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
        })
    }
}

fn push_postcondition(
    builder: &mut AftermathBasisBuilder,
    postcondition: &worth_query_declaration::facade::application_aftermath::DeclaredAftermathPostcondition,
) {
    use worth_query_declaration::facade::application_aftermath::DeclaredAftermathPostcondition;
    match postcondition {
        DeclaredAftermathPostcondition::ExactPriorTruth => {
            builder.text("postcondition", "exact-prior-truth");
        }
        DeclaredAftermathPostcondition::InvariantRestored { invariant } => {
            builder.text("postcondition", "invariant-restored");
            builder.text("postcondition-invariant", invariant);
        }
        DeclaredAftermathPostcondition::BusinessPostcondition { identity } => {
            builder.text("postcondition", "business-postcondition");
            builder.text("postcondition-business", identity);
        }
    }
}

fn authority_label(
    authority: worth_query_declaration::facade::application_aftermath::DeclaredCorrectionAuthority,
) -> &'static str {
    use worth_query_declaration::facade::application_aftermath::DeclaredCorrectionAuthority;
    match authority {
        DeclaredCorrectionAuthority::RuntimeAlone => "runtime-alone",
        DeclaredCorrectionAuthority::RuntimeWithExternalOwner => "runtime-with-external-owner",
        DeclaredCorrectionAuthority::NotCorrectable => "not-correctable",
    }
}

fn canonical_denial(
    subject: &str,
    denial: CanonicalDigestDerivationDenial,
) -> WorthQueryAftermathInstallationDenial {
    let kind = match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { .. } => {
            WorthQueryAftermathInstallationDenialKind::CanonicalEntryLimitExceeded
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. } => {
            WorthQueryAftermathInstallationDenialKind::CanonicalByteLimitExceeded
        }
        _ => WorthQueryAftermathInstallationDenialKind::CanonicalDigestSlotRejected,
    };
    WorthQueryAftermathInstallationDenial::new(kind, subject)
}
