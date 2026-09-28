use crate::admission_digest::hash_parts;

use super::admitted_capability::AdmittedBasisCapability;
use super::counters::BasisEligibilityCounters;
use super::lanes::{
    BasisOperationLane, CertificationLaneWitness, InspectionLaneWitness,
    MaterializationLaneWitness, MutationPreparationLaneWitness, ObservationLaneWitness,
    PreviewCloseoutLaneWitness, ReplayLaneWitness, SubscriptionActivationLaneWitness,
    SubscriptionDeclarationLaneWitness,
};
use super::taxonomy::{BasisAuthorityPosture, BasisFamily, BasisLifecyclePosture};

macro_rules! scoped_basis {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        ///
        /// It records the family, authority posture, and lifecycle posture of
        /// the admitted basis capability it was scoped from, with digests that
        /// identify that capability and this scoped basis. It is usable only
        /// in its own lane.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            family: BasisFamily,
            authority: BasisAuthorityPosture,
            lifecycle: BasisLifecyclePosture,
            capability_digest: String,
            expected_lower_runtime_binding_digest: Option<String>,
            scoped_basis_digest: String,
            counters: BasisEligibilityCounters,
        }

        impl $name {
            pub(crate) fn new<L: BasisOperationLane>(
                capability: AdmittedBasisCapability<L>,
            ) -> Self {
                let scoped_basis_digest = hash_parts(&[
                    stringify!($name).to_string(),
                    format!("capability:{}", capability.capability_digest()),
                ]);
                Self {
                    family: capability.normalized().family(),
                    authority: capability.normalized().authority(),
                    lifecycle: capability.normalized().lifecycle(),
                    capability_digest: capability.capability_digest().to_string(),
                    expected_lower_runtime_binding_digest: capability
                        .normalized()
                        .lower_runtime_binding_digest()
                        .map(str::to_string),
                    scoped_basis_digest,
                    counters: BasisEligibilityCounters::scoped_capability(),
                }
            }

            pub fn family(&self) -> BasisFamily {
                self.family
            }

            pub fn authority(&self) -> BasisAuthorityPosture {
                self.authority
            }

            pub fn lifecycle(&self) -> BasisLifecyclePosture {
                self.lifecycle
            }

            pub fn capability_digest(&self) -> &str {
                &self.capability_digest
            }

            pub fn expected_lower_runtime_binding_digest(&self) -> Option<&str> {
                self.expected_lower_runtime_binding_digest.as_deref()
            }

            pub fn scoped_basis_digest(&self) -> &str {
                &self.scoped_basis_digest
            }

            pub fn counters(&self) -> &BasisEligibilityCounters {
                &self.counters
            }
        }
    };
}

scoped_basis!(
    /// An admitted basis scoped to the observation lane.
    ScopedObservationBasis
);
scoped_basis!(
    /// An admitted basis scoped to the mutation-preparation lane.
    ScopedMutationPreparationBasis
);
scoped_basis!(
    /// An admitted basis scoped to the replay lane, for certification replay.
    ///
    /// You get one from `BasisLifecycleIntentDraft::replay` once the basis is
    /// admitted for replay.
    ScopedReplayBasis
);
scoped_basis!(
    /// An admitted basis scoped to the inspection lane.
    ScopedInspectionBasis
);
scoped_basis!(
    /// An admitted basis scoped to the materialization lane.
    ScopedMaterializationBasis
);
scoped_basis!(
    /// An admitted basis scoped to the subscription-declaration lane.
    ScopedSubscriptionDeclarationBasis
);
scoped_basis!(
    /// An admitted basis scoped to the subscription-activation lane.
    ScopedSubscriptionActivationBasis
);
scoped_basis!(
    /// An admitted basis scoped to the preview-closeout lane.
    ScopedPreviewCloseoutBasis
);
scoped_basis!(
    /// An admitted basis scoped to the certification lane.
    ScopedCertificationBasis
);

pub trait ScopedBasisProof: Clone + std::fmt::Debug + Eq + PartialEq {
    fn family(&self) -> BasisFamily;
    fn authority(&self) -> BasisAuthorityPosture;
    fn lifecycle(&self) -> BasisLifecyclePosture;
    fn capability_digest(&self) -> &str;
    fn scoped_basis_digest(&self) -> &str;
    fn expected_lower_runtime_binding_digest(&self) -> Option<&str>;
}

macro_rules! scoped_basis_proof {
    ($name:ident) => {
        impl ScopedBasisProof for $name {
            fn family(&self) -> BasisFamily {
                self.family()
            }

            fn authority(&self) -> BasisAuthorityPosture {
                self.authority()
            }

            fn lifecycle(&self) -> BasisLifecyclePosture {
                self.lifecycle()
            }

            fn capability_digest(&self) -> &str {
                self.capability_digest()
            }

            fn scoped_basis_digest(&self) -> &str {
                self.scoped_basis_digest()
            }

            fn expected_lower_runtime_binding_digest(&self) -> Option<&str> {
                self.expected_lower_runtime_binding_digest()
            }
        }
    };
}

scoped_basis_proof!(ScopedObservationBasis);
scoped_basis_proof!(ScopedMutationPreparationBasis);
scoped_basis_proof!(ScopedReplayBasis);
scoped_basis_proof!(ScopedInspectionBasis);
scoped_basis_proof!(ScopedMaterializationBasis);
scoped_basis_proof!(ScopedSubscriptionDeclarationBasis);
scoped_basis_proof!(ScopedSubscriptionActivationBasis);
scoped_basis_proof!(ScopedPreviewCloseoutBasis);
scoped_basis_proof!(ScopedCertificationBasis);

pub fn activate_subscription_basis(
    declaration: &ScopedSubscriptionDeclarationBasis,
) -> ScopedSubscriptionActivationBasis {
    let capability_digest = hash_parts(&[
        "subscription_activation_capability_from_declaration_v1".to_string(),
        format!("declaration:{}", declaration.capability_digest()),
    ]);
    let scoped_basis_digest = hash_parts(&[
        "ScopedSubscriptionActivationBasis".to_string(),
        format!("capability:{capability_digest}"),
    ]);
    ScopedSubscriptionActivationBasis {
        family: declaration.family,
        authority: declaration.authority,
        lifecycle: declaration.lifecycle,
        capability_digest,
        expected_lower_runtime_binding_digest: declaration
            .expected_lower_runtime_binding_digest
            .clone(),
        scoped_basis_digest,
        counters: BasisEligibilityCounters::scoped_capability(),
    }
}

pub fn scope_basis_for_observation(
    capability: AdmittedBasisCapability<ObservationLaneWitness>,
) -> ScopedObservationBasis {
    ScopedObservationBasis::new(capability)
}

pub fn scope_basis_for_mutation_preparation(
    capability: AdmittedBasisCapability<MutationPreparationLaneWitness>,
) -> ScopedMutationPreparationBasis {
    ScopedMutationPreparationBasis::new(capability)
}

pub fn scope_basis_for_replay(
    capability: AdmittedBasisCapability<ReplayLaneWitness>,
) -> ScopedReplayBasis {
    ScopedReplayBasis::new(capability)
}

pub fn scope_basis_for_inspection(
    capability: AdmittedBasisCapability<InspectionLaneWitness>,
) -> ScopedInspectionBasis {
    ScopedInspectionBasis::new(capability)
}

pub fn scope_basis_for_materialization(
    capability: AdmittedBasisCapability<MaterializationLaneWitness>,
) -> ScopedMaterializationBasis {
    ScopedMaterializationBasis::new(capability)
}

pub fn scope_basis_for_subscription_declaration(
    capability: AdmittedBasisCapability<SubscriptionDeclarationLaneWitness>,
) -> ScopedSubscriptionDeclarationBasis {
    ScopedSubscriptionDeclarationBasis::new(capability)
}

pub fn scope_basis_for_subscription_activation(
    capability: AdmittedBasisCapability<SubscriptionActivationLaneWitness>,
) -> ScopedSubscriptionActivationBasis {
    ScopedSubscriptionActivationBasis::new(capability)
}

pub fn scope_basis_for_preview_closeout(
    capability: AdmittedBasisCapability<PreviewCloseoutLaneWitness>,
) -> ScopedPreviewCloseoutBasis {
    ScopedPreviewCloseoutBasis::new(capability)
}

pub fn scope_basis_for_certification(
    capability: AdmittedBasisCapability<CertificationLaneWitness>,
) -> ScopedCertificationBasis {
    ScopedCertificationBasis::new(capability)
}
