use std::collections::BTreeSet;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::error::{BridgeBuildError, BridgeBuildErrorKind};
use crate::mapping::SubscriptionSliceKind;

use super::{
    BridgeSubscriptionCounters, BridgeSubscriptionDeclarationFamily,
    BridgeSubscriptionDeclarationFamilyIdentity, BridgeSubscriptionDeclarationFamilyKind,
    BridgeSubscriptionFamilyRegistryIdentity,
};

#[cfg(test)]
#[path = "family_registry_tests.rs"]
mod family_registry_tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenSubscriptionFamilyRegistration {
    family: BridgeSubscriptionDeclarationFamily,
}

impl FrozenSubscriptionFamilyRegistration {
    fn new(family: BridgeSubscriptionDeclarationFamily) -> Self {
        Self { family }
    }

    pub fn family_identity(&self) -> &BridgeSubscriptionDeclarationFamilyIdentity {
        self.family.family_identity()
    }

    pub fn family_kind(&self) -> BridgeSubscriptionDeclarationFamilyKind {
        self.family.family_kind()
    }

    pub fn family_label(&self) -> &str {
        self.family.family_label()
    }

    pub fn supported_slice_kinds(&self) -> &[SubscriptionSliceKind] {
        self.family.supported_slice_kinds()
    }

    pub fn allows_delivery_intent_identity(&self) -> bool {
        self.family.allows_delivery_intent_identity()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenSubscriptionFamilyRegistry {
    registry_identity: BridgeSubscriptionFamilyRegistryIdentity,
    registrations: Arc<[FrozenSubscriptionFamilyRegistration]>,
    detail_exact_index: usize,
    collection_membership_index: usize,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
}

impl FrozenSubscriptionFamilyRegistry {
    pub(crate) fn freeze(
        mut families: Vec<BridgeSubscriptionDeclarationFamily>,
    ) -> Result<Self, BridgeBuildError> {
        if families.is_empty() {
            return Err(BridgeBuildError::new(
                BridgeBuildErrorKind::BuilderConfigurationConflict,
                "Bridge subscription family registry requires at least one declaration family.",
            ));
        }

        families.sort_by(canonical_family_order);
        validate_family_set(&families)?;

        let registrations = Arc::<[FrozenSubscriptionFamilyRegistration]>::from(
            families
                .into_iter()
                .map(FrozenSubscriptionFamilyRegistration::new)
                .collect::<Vec<_>>(),
        );

        let detail_exact_index = registrations
            .iter()
            .position(|registration| {
                registration.family_kind() == BridgeSubscriptionDeclarationFamilyKind::DetailExact
            })
            .ok_or_else(|| {
                BridgeBuildError::new(
                    BridgeBuildErrorKind::BuilderConfigurationConflict,
                    "Bridge subscription family registry is missing the required `detail_exact` family.",
                )
            })?;
        let collection_membership_index = registrations
            .iter()
            .position(|registration| {
                registration.family_kind()
                    == BridgeSubscriptionDeclarationFamilyKind::CollectionMembership
            })
            .ok_or_else(|| {
                BridgeBuildError::new(
                    BridgeBuildErrorKind::BuilderConfigurationConflict,
                    "Bridge subscription family registry is missing the required `collection_membership` family.",
                )
            })?;

        let mut basis = format!(
            "bridge-subscription-family-registry|family-count={}",
            registrations.len()
        );
        for registration in registrations.iter() {
            basis.push_str("|family=");
            basis.push_str(registration.family_identity().as_str());
            basis.push_str("|kind=");
            basis.push_str(registration.family_kind().as_str());
            basis.push_str("|label=");
            basis.push_str(registration.family_label());
            basis.push_str("|delivery-intent-identity=");
            basis.push_str(if registration.allows_delivery_intent_identity() {
                "true"
            } else {
                "false"
            });
            for slice_kind in registration.supported_slice_kinds() {
                basis.push_str("|slice-kind=");
                basis.push_str(match slice_kind {
                    SubscriptionSliceKind::SignalAspect => "signal_aspect",
                    SubscriptionSliceKind::SignalField => "signal_field",
                    SubscriptionSliceKind::SignalLens => "signal_lens",
                    SubscriptionSliceKind::SignalRegion => "signal_region",
                    SubscriptionSliceKind::SignalPartition => "signal_partition",
                    SubscriptionSliceKind::SignalFacet => "signal_facet",
                    SubscriptionSliceKind::SignalLifecycle => "signal_lifecycle",
                    SubscriptionSliceKind::RegisteredCoarseWidening => "registered_coarse_widening",
                });
            }
        }
        let canonical_basis = Arc::<str>::from(basis);
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let family_count = registrations.len();
        let family_supported_slice_kind_count = registrations
            .iter()
            .map(|registration| registration.supported_slice_kinds().len())
            .sum();
        Ok(Self {
            registry_identity: BridgeSubscriptionFamilyRegistryIdentity::admit_bridge_owned(
                format!("bridge-subscription-family-registry:sha256:{digest:x}"),
            ),
            registrations,
            detail_exact_index,
            collection_membership_index,
            counters: BridgeSubscriptionCounters::from_frozen_registry(
                family_count,
                family_supported_slice_kind_count,
            ),
            canonical_basis,
        })
    }

    pub fn registry_identity(&self) -> &BridgeSubscriptionFamilyRegistryIdentity {
        &self.registry_identity
    }

    pub fn registrations(&self) -> &[FrozenSubscriptionFamilyRegistration] {
        &self.registrations
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub(crate) fn family_for_kind(
        &self,
        family_kind: BridgeSubscriptionDeclarationFamilyKind,
    ) -> &FrozenSubscriptionFamilyRegistration {
        let index = match family_kind {
            BridgeSubscriptionDeclarationFamilyKind::DetailExact => self.detail_exact_index,
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership => {
                self.collection_membership_index
            }
        };
        &self.registrations[index]
    }
}

pub(crate) fn freeze_subscription_family_registry(
) -> Result<FrozenSubscriptionFamilyRegistry, BridgeBuildError> {
    FrozenSubscriptionFamilyRegistry::freeze(phase_one_subscription_families()?)
}

pub(crate) fn phase_one_subscription_families(
) -> Result<Vec<BridgeSubscriptionDeclarationFamily>, BridgeBuildError> {
    Ok(vec![
        BridgeSubscriptionDeclarationFamily::new(
            BridgeSubscriptionDeclarationFamilyIdentity::admit_bridge_owned(
                "subscription-family:collection-membership",
            ),
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership,
            "collection_membership",
            vec![
                SubscriptionSliceKind::SignalPartition,
                SubscriptionSliceKind::SignalRegion,
            ],
            false,
        )?,
        BridgeSubscriptionDeclarationFamily::new(
            BridgeSubscriptionDeclarationFamilyIdentity::admit_bridge_owned(
                "subscription-family:detail-exact",
            ),
            BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            "detail_exact",
            vec![
                SubscriptionSliceKind::SignalField,
                SubscriptionSliceKind::SignalLens,
            ],
            false,
        )?,
    ])
}

fn canonical_family_order(
    left: &BridgeSubscriptionDeclarationFamily,
    right: &BridgeSubscriptionDeclarationFamily,
) -> std::cmp::Ordering {
    left.family_kind()
        .cmp(&right.family_kind())
        .then_with(|| left.family_identity().cmp(right.family_identity()))
}

fn validate_family_set(
    families: &[BridgeSubscriptionDeclarationFamily],
) -> Result<(), BridgeBuildError> {
    let mut family_kinds = BTreeSet::new();
    let mut family_identities = BTreeSet::new();
    let mut family_labels = BTreeSet::new();

    for family in families {
        let supported_slice_kinds = family.supported_slice_kinds();
        if supported_slice_kinds.is_empty() {
            return Err(BridgeBuildError::new(
                BridgeBuildErrorKind::BuilderConfigurationConflict,
                format!(
                    "Bridge subscription family `{}` admitted zero supported slice kinds.",
                    family.family_kind().as_str()
                ),
            ));
        }
        if supported_slice_kinds
            .windows(2)
            .any(|window| window[0] >= window[1])
        {
            return Err(BridgeBuildError::new(
                BridgeBuildErrorKind::BuilderConfigurationConflict,
                format!(
                    "Bridge subscription family `{}` contained non-canonical or duplicate supported slice kinds.",
                    family.family_kind().as_str()
                ),
            ));
        }
        if !family_kinds.insert(family.family_kind()) {
            return Err(BridgeBuildError::new(
                BridgeBuildErrorKind::BuilderConfigurationConflict,
                format!(
                    "Bridge subscription family registry contained duplicate family kind `{}`.",
                    family.family_kind().as_str()
                ),
            ));
        }
        if !family_identities.insert(family.family_identity().clone()) {
            return Err(BridgeBuildError::new(
                BridgeBuildErrorKind::BuilderConfigurationConflict,
                format!(
                    "Bridge subscription family registry contained duplicate family identity `{}`.",
                    family.family_identity().as_str()
                ),
            ));
        }
        if !family_labels.insert(Arc::<str>::from(family.family_label())) {
            return Err(BridgeBuildError::new(
                BridgeBuildErrorKind::BuilderConfigurationConflict,
                format!(
                    "Bridge subscription family registry contained duplicate family label `{}`.",
                    family.family_label()
                ),
            ));
        }
    }

    Ok(())
}
