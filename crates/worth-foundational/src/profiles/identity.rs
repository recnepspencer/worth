use worth_proof::TransitionOutcome;

use super::difference::{
    compare_foundational_profiles, FoundationalProfileCompatibilityClass,
    FoundationalProfileDifferenceReport,
};
use super::{AdmittedFoundationalProfileArtifact, FoundationalProfileSet};
use crate::canonicalization::{
    admit_canonical_sequence_digest_derivation_with_budget, compare_canonical_basis,
    derive_canonical_digest, prepare_canonical_basis_sequence, prepare_canonical_comparison,
    CanonicalBasisConstructionDenial, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact,
    CanonicalBasisSequence, CanonicalBasisValue, CanonicalComparisonOutcome,
    CanonicalDerivedDigest, CanonicalDigestAlgorithmId, CanonicalDigestWorkBudget,
    CanonicalSingleSequenceDigestAlgorithmSlot, CanonicalizationRuleVersion,
};

const PROFILE_DIGEST_BUDGET: CanonicalDigestWorkBudget =
    match CanonicalDigestWorkBudget::new(256, 64 * 1_024) {
        Some(budget) => budget,
        None => panic!("profile digest budget is nonzero"),
    };

#[derive(Debug, Clone)]
pub struct FoundationalProfileIdentity {
    basis: CanonicalBasisSequence,
    digest: CanonicalDerivedDigest,
}

impl FoundationalProfileIdentity {
    pub fn basis(&self) -> &CanonicalBasisSequence {
        &self.basis
    }

    pub fn digest(&self) -> &CanonicalDerivedDigest {
        &self.digest
    }
}

impl PartialEq for FoundationalProfileIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
            && self.basis.version() == other.basis.version()
            && self.basis.domain() == other.basis.domain()
            && self.basis.entries() == other.basis.entries()
    }
}

impl Eq for FoundationalProfileIdentity {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoundationalProfileIdentityDenial {
    BasisConstructionDenied(CanonicalBasisConstructionDenial),
    DigestDerivationDenied(crate::canonicalization::CanonicalDigestDerivationDenial),
}

pub fn prepare_admitted_foundational_profile_for_canonical_basis(
    version: CanonicalizationRuleVersion,
    admitted: &AdmittedFoundationalProfileArtifact,
) -> TransitionOutcome<CanonicalBasisReadyArtifact, CanonicalBasisConstructionDenial> {
    prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::Profile,
        foundational_profile_basis_entries(*admitted.payload().admitted()),
    )
}

pub fn foundational_profile_canonical_basis_entries(
    ready: &CanonicalBasisSequence,
) -> &[CanonicalBasisEntry] {
    ready.entries()
}

pub fn derive_foundational_profile_identity(
    version: CanonicalizationRuleVersion,
    admitted: &AdmittedFoundationalProfileArtifact,
) -> TransitionOutcome<FoundationalProfileIdentity, FoundationalProfileIdentityDenial> {
    let basis = match prepare_admitted_foundational_profile_for_canonical_basis(
        version.clone(),
        admitted,
    ) {
        TransitionOutcome::Success(ready) => ready,
        TransitionOutcome::Denied(denial) => {
            return TransitionOutcome::denied(
                FoundationalProfileIdentityDenial::BasisConstructionDenied(denial),
            );
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => {
            unreachable!("canonical basis preparation uses only denied")
        }
    };
    let basis_sequence = basis.payload().clone();

    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Profile,
        version,
    );

    let derivation = match admit_canonical_sequence_digest_derivation_with_budget(
        basis,
        slot,
        PROFILE_DIGEST_BUDGET,
    ) {
        TransitionOutcome::Success(ready) => ready,
        TransitionOutcome::Denied(denial) => {
            return TransitionOutcome::denied(
                FoundationalProfileIdentityDenial::DigestDerivationDenied(denial),
            );
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => unreachable!("profile digest admission uses only denied"),
    };

    TransitionOutcome::success(FoundationalProfileIdentity {
        basis: basis_sequence,
        digest: derive_canonical_digest(derivation),
    })
}

pub fn compare_foundational_profile_identities(
    left: &FoundationalProfileIdentity,
    right: &FoundationalProfileIdentity,
) -> (
    FoundationalProfileDifferenceReport,
    CanonicalComparisonOutcome,
) {
    let left_profile = profile_set_from_basis(left.basis());
    let right_profile = profile_set_from_basis(right.basis());
    let report = compare_foundational_profiles(left_profile, right_profile);
    let comparison = match prepare_canonical_comparison(
        crate::canonicalization::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        rebuild_profile_basis_ready(left.basis()),
        rebuild_profile_basis_ready(right.basis()),
    ) {
        TransitionOutcome::Success(ready) => compare_canonical_basis(&ready),
        TransitionOutcome::Denied(_)
        | TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => unreachable!("comparison readiness is infallible"),
    };

    (report, comparison)
}

pub fn classify_foundational_profile_compatibility(
    left: &FoundationalProfileIdentity,
    right: &FoundationalProfileIdentity,
) -> FoundationalProfileCompatibilityClass {
    compare_foundational_profile_identities(left, right)
        .0
        .compatibility_class()
}

fn foundational_profile_basis_entries(profile: FoundationalProfileSet) -> [CanonicalBasisEntry; 8] {
    [
        profile_text_entry(
            "diagnostic_richness",
            diagnostic_richness_token(profile.diagnostic_richness()),
        ),
        profile_text_entry(
            "support_posture",
            support_posture_token(profile.support_posture()),
        ),
        profile_text_entry(
            "compatibility_posture",
            compatibility_posture_token(profile.compatibility_posture()),
        ),
        profile_text_entry(
            "admission_readiness",
            admission_readiness_token(profile.admission_readiness()),
        ),
        profile_text_entry(
            "retention_delivery",
            retention_delivery_token(profile.retention_delivery()),
        ),
        profile_text_entry(
            "certification_posture",
            certification_posture_token(profile.certification_posture()),
        ),
        profile_text_entry(
            "execution_objective",
            execution_objective_token(profile.execution_objective()),
        ),
        profile_text_entry(
            "observation_activation",
            observation_activation_token(profile.observation_activation()),
        ),
    ]
}

fn profile_text_entry(locus: &'static str, value: &'static str) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Profile,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Profile,
        CanonicalBasisValue::ExactText(value.to_string().into()),
    )
}

fn rebuild_profile_basis_ready(sequence: &CanonicalBasisSequence) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        sequence.version().clone(),
        sequence.domain(),
        sequence.entries().iter().cloned(),
    ) {
        TransitionOutcome::Success(ready) => ready,
        TransitionOutcome::Denied(denial) => {
            panic!("stored profile basis must rebuild cleanly: {denial:?}")
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => unreachable!("profile basis rebuild uses only denied"),
    }
}

fn diagnostic_richness_token(value: crate::profiles::DiagnosticRichnessProfile) -> &'static str {
    match value {
        crate::profiles::DiagnosticRichnessProfile::OperationalMinimal => "operational-minimal",
        crate::profiles::DiagnosticRichnessProfile::Standard => "standard",
        crate::profiles::DiagnosticRichnessProfile::Forensic => "forensic",
    }
}

fn support_posture_token(value: crate::profiles::SupportPostureProfile) -> &'static str {
    match value {
        crate::profiles::SupportPostureProfile::InternalOnly => "internal-only",
        crate::profiles::SupportPostureProfile::SupportReady => "support-ready",
        crate::profiles::SupportPostureProfile::CertificationReady => "certification-ready",
    }
}

fn compatibility_posture_token(
    value: crate::profiles::CompatibilityPostureProfile,
) -> &'static str {
    match value {
        crate::profiles::CompatibilityPostureProfile::NativeOnly => "native-only",
        crate::profiles::CompatibilityPostureProfile::CompatibilityLowered => {
            "compatibility-lowered"
        }
        crate::profiles::CompatibilityPostureProfile::CompatibilityRequired => {
            "compatibility-required"
        }
    }
}

fn admission_readiness_token(value: crate::profiles::AdmissionReadinessProfile) -> &'static str {
    match value {
        crate::profiles::AdmissionReadinessProfile::CandidateOnly => "candidate-only",
        crate::profiles::AdmissionReadinessProfile::Admitted => "admitted",
        crate::profiles::AdmissionReadinessProfile::ProductionGateReady => "production-gate-ready",
    }
}

fn retention_delivery_token(value: crate::profiles::RetentionDeliveryProfile) -> &'static str {
    match value {
        crate::profiles::RetentionDeliveryProfile::Ephemeral => "ephemeral",
        crate::profiles::RetentionDeliveryProfile::Retained => "retained",
        crate::profiles::RetentionDeliveryProfile::Durable => "durable",
    }
}

fn certification_posture_token(
    value: crate::profiles::CertificationPostureProfile,
) -> &'static str {
    match value {
        crate::profiles::CertificationPostureProfile::Uncertified => "uncertified",
        crate::profiles::CertificationPostureProfile::EvidenceBacked => "evidence-backed",
        crate::profiles::CertificationPostureProfile::ProductionCertified => "production-certified",
    }
}

fn execution_objective_token(value: crate::profiles::ExecutionObjectiveProfile) -> &'static str {
    value.token()
}

fn observation_activation_token(
    value: crate::profiles::ObservationActivationProfile,
) -> &'static str {
    value.token()
}

fn profile_set_from_basis(ready: &CanonicalBasisSequence) -> FoundationalProfileSet {
    let entry = |name: &str| {
        ready
            .entries()
            .iter()
            .find(|entry| entry.locus() == &CanonicalBasisLocus::Named(name.to_string().into()))
            .expect("profile basis entry exists")
    };
    let text = |name: &str| match entry(name).value() {
        CanonicalBasisValue::ExactText(value) => match value {
            crate::values::InternedString::Raw(value) => value.as_str(),
            crate::values::InternedString::Stored(value) => value.as_str(),
            crate::values::InternedString::Symbol(symbol) => {
                panic!("expected raw profile token, got symbol {}", symbol.0)
            }
        },
        other => panic!("expected text value, got {other:?}"),
    };

    FoundationalProfileSet::new(super::FoundationalProfileSetInput {
        diagnostic_richness: match text("diagnostic_richness") {
            "operational-minimal" => crate::profiles::DiagnosticRichnessProfile::OperationalMinimal,
            "standard" => crate::profiles::DiagnosticRichnessProfile::Standard,
            "forensic" => crate::profiles::DiagnosticRichnessProfile::Forensic,
            other => panic!("unexpected diagnostic richness token {other}"),
        },
        support_posture: match text("support_posture") {
            "internal-only" => crate::profiles::SupportPostureProfile::InternalOnly,
            "support-ready" => crate::profiles::SupportPostureProfile::SupportReady,
            "certification-ready" => crate::profiles::SupportPostureProfile::CertificationReady,
            other => panic!("unexpected support posture token {other}"),
        },
        compatibility_posture: match text("compatibility_posture") {
            "native-only" => crate::profiles::CompatibilityPostureProfile::NativeOnly,
            "compatibility-lowered" => {
                crate::profiles::CompatibilityPostureProfile::CompatibilityLowered
            }
            "compatibility-required" => {
                crate::profiles::CompatibilityPostureProfile::CompatibilityRequired
            }
            other => panic!("unexpected compatibility posture token {other}"),
        },
        admission_readiness: match text("admission_readiness") {
            "candidate-only" => crate::profiles::AdmissionReadinessProfile::CandidateOnly,
            "admitted" => crate::profiles::AdmissionReadinessProfile::Admitted,
            "production-gate-ready" => {
                crate::profiles::AdmissionReadinessProfile::ProductionGateReady
            }
            other => panic!("unexpected admission readiness token {other}"),
        },
        retention_delivery: match text("retention_delivery") {
            "ephemeral" => crate::profiles::RetentionDeliveryProfile::Ephemeral,
            "retained" => crate::profiles::RetentionDeliveryProfile::Retained,
            "durable" => crate::profiles::RetentionDeliveryProfile::Durable,
            other => panic!("unexpected retention delivery token {other}"),
        },
        certification_posture: match text("certification_posture") {
            "uncertified" => crate::profiles::CertificationPostureProfile::Uncertified,
            "evidence-backed" => crate::profiles::CertificationPostureProfile::EvidenceBacked,
            "production-certified" => {
                crate::profiles::CertificationPostureProfile::ProductionCertified
            }
            other => panic!("unexpected certification posture token {other}"),
        },
        execution_objective: match text("execution_objective") {
            "latency-bounded" => crate::profiles::ExecutionObjectiveProfile::LatencyBounded,
            "balanced" => crate::profiles::ExecutionObjectiveProfile::Balanced,
            "throughput" => crate::profiles::ExecutionObjectiveProfile::Throughput,
            other => panic!("unexpected execution objective token {other}"),
        },
        observation_activation: match text("observation_activation") {
            "on-demand" => crate::profiles::ObservationActivationProfile::OnDemand,
            "continuous" => crate::profiles::ObservationActivationProfile::Continuous,
            other => panic!("unexpected observation activation token {other}"),
        },
    })
    .expect("profile basis entries reconstruct coherent profile")
}

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
