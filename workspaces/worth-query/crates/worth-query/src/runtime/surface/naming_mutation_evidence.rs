use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryMutationAuthorityIdentity, WorthQueryMutationTargetCollectionIdentity,
};
use worth_runtime_bridge::facade::{
    BridgeNamingMutationBundle, BridgeNamingMutationFamily, BridgeNamingMutationOutcome,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryNamingMutationOutcome {
    AttachedToNewTarget,
    AttachedToExistingTarget,
    ReboundTarget,
    Removed,
}

impl WorthQueryNamingMutationOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AttachedToNewTarget => "attached_to_new_target",
            Self::AttachedToExistingTarget => "attached_to_existing_target",
            Self::ReboundTarget => "rebound_target",
            Self::Removed => "removed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryNamingMutationEvidence {
    family: crate::runtime::WorthQueryNamingMutationFamily,
    outcome: WorthQueryNamingMutationOutcome,
    attachment_identity: WorthQueryMutationAuthorityIdentity,
    prior_authoritative_identity: Option<WorthQueryMutationAuthorityIdentity>,
    target_authoritative_identity: Option<WorthQueryMutationAuthorityIdentity>,
    resolved_target_entity_identity: Option<WorthQueryEntityIdentity>,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
}

impl WorthQueryNamingMutationEvidence {
    #[cfg(test)]
    pub(in crate::runtime) fn from_bridge(
        bundle: &BridgeNamingMutationBundle,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::from_bridge_with_query_context(bundle, None, None)
    }

    pub(in crate::runtime) fn from_bridge_with_query_context(
        bundle: &BridgeNamingMutationBundle,
        resolved_target_entity_identity: Option<&WorthQueryEntityIdentity>,
        target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            family: match bundle.family() {
                BridgeNamingMutationFamily::AttachNewTarget => {
                    crate::runtime::WorthQueryNamingMutationFamily::AttachNewTarget
                }
                BridgeNamingMutationFamily::AttachExistingTarget => {
                    crate::runtime::WorthQueryNamingMutationFamily::AttachExistingTarget
                }
                BridgeNamingMutationFamily::RebindTarget => {
                    crate::runtime::WorthQueryNamingMutationFamily::RebindTarget
                }
                BridgeNamingMutationFamily::Remove => {
                    crate::runtime::WorthQueryNamingMutationFamily::Remove
                }
            },
            outcome: match bundle.outcome() {
                BridgeNamingMutationOutcome::AttachedToNewTarget => {
                    WorthQueryNamingMutationOutcome::AttachedToNewTarget
                }
                BridgeNamingMutationOutcome::AttachedToExistingTarget => {
                    WorthQueryNamingMutationOutcome::AttachedToExistingTarget
                }
                BridgeNamingMutationOutcome::ReboundTarget => {
                    WorthQueryNamingMutationOutcome::ReboundTarget
                }
                BridgeNamingMutationOutcome::Removed => WorthQueryNamingMutationOutcome::Removed,
            },
            attachment_identity:
                WorthQueryMutationAuthorityIdentity::from_bridge_naming_attachment(
                    "naming-attachment",
                    bundle.attachment_identity_handle(),
                )?,
            prior_authoritative_identity: bundle
                .prior_authoritative_identity_handle()
                .map(|identity| {
                    WorthQueryMutationAuthorityIdentity::from_bridge_naming_authority(
                        "naming-prior",
                        identity,
                    )
                })
                .transpose()?,
            target_authoritative_identity: bundle
                .target_authoritative_identity_handle()
                .map(|identity| {
                    WorthQueryMutationAuthorityIdentity::from_bridge_naming_authority(
                        "naming-target",
                        identity,
                    )
                })
                .transpose()?,
            resolved_target_entity_identity: bundle
                .resolved_target_entity_identity_handle()
                .map(|identity| {
                    WorthQueryEntityIdentity::from_runtime_receipt_record(
                        identity.relational_record_parts(),
                    )
                })
                .or_else(|| resolved_target_entity_identity.cloned()),
            target_collection: bundle
                .target_collection()
                .map(|collection| {
                    WorthQueryMutationTargetCollectionIdentity::new("naming-target", collection)
                })
                .transpose()?
                .or_else(|| target_collection.cloned()),
        })
    }

    pub(in crate::runtime) fn from_intent(
        intent: &crate::runtime::WorthQueryNamingMutationIntent,
        resolved_target_entity_identity: Option<&WorthQueryEntityIdentity>,
        target_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
    ) -> Self {
        Self {
            family: intent.family(),
            outcome: match intent.family() {
                crate::runtime::WorthQueryNamingMutationFamily::AttachNewTarget => {
                    WorthQueryNamingMutationOutcome::AttachedToNewTarget
                }
                crate::runtime::WorthQueryNamingMutationFamily::AttachExistingTarget => {
                    WorthQueryNamingMutationOutcome::AttachedToExistingTarget
                }
                crate::runtime::WorthQueryNamingMutationFamily::RebindTarget => {
                    WorthQueryNamingMutationOutcome::ReboundTarget
                }
                crate::runtime::WorthQueryNamingMutationFamily::Remove => {
                    WorthQueryNamingMutationOutcome::Removed
                }
            },
            attachment_identity: intent.attachment_identity().clone(),
            prior_authoritative_identity: intent.prior_authoritative_identity().cloned(),
            target_authoritative_identity: intent.target_authoritative_identity().cloned(),
            resolved_target_entity_identity: resolved_target_entity_identity.cloned(),
            target_collection: target_collection.cloned(),
        }
    }

    pub fn family(&self) -> crate::runtime::WorthQueryNamingMutationFamily {
        self.family
    }

    pub fn outcome(&self) -> WorthQueryNamingMutationOutcome {
        self.outcome
    }

    pub fn attachment_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.attachment_identity
    }

    pub fn prior_authoritative_identity(&self) -> Option<&WorthQueryMutationAuthorityIdentity> {
        self.prior_authoritative_identity.as_ref()
    }

    pub fn target_authoritative_identity(&self) -> Option<&WorthQueryMutationAuthorityIdentity> {
        self.target_authoritative_identity.as_ref()
    }

    pub fn resolved_target_entity_identity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.resolved_target_entity_identity.as_ref()
    }

    pub fn target_collection(&self) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }
}
