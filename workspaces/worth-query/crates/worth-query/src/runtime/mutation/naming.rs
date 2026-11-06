use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryMutationAuthorityIdentity;

use super::super::{WorthQueryMutationFamily, WorthQueryWriteCommand};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryNamingMutationFamily {
    AttachNewTarget,
    AttachExistingTarget,
    RebindTarget,
    Remove,
}

impl WorthQueryNamingMutationFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AttachNewTarget => "attach_new_target",
            Self::AttachExistingTarget => "attach_existing_target",
            Self::RebindTarget => "rebind_target",
            Self::Remove => "remove",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryNamingMutationIntent {
    family: WorthQueryNamingMutationFamily,
    attachment_identity: WorthQueryMutationAuthorityIdentity,
    prior_authoritative_identity: Option<WorthQueryMutationAuthorityIdentity>,
    target_authoritative_identity: Option<WorthQueryMutationAuthorityIdentity>,
}

impl WorthQueryNamingMutationIntent {
    pub fn attach_new_target(attachment_identity: WorthQueryMutationAuthorityIdentity) -> Self {
        Self {
            family: WorthQueryNamingMutationFamily::AttachNewTarget,
            attachment_identity,
            prior_authoritative_identity: None,
            target_authoritative_identity: None,
        }
    }

    pub fn attach_existing_target(
        attachment_identity: WorthQueryMutationAuthorityIdentity,
        target_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    ) -> Self {
        Self {
            family: WorthQueryNamingMutationFamily::AttachExistingTarget,
            attachment_identity,
            prior_authoritative_identity: None,
            target_authoritative_identity: Some(target_authoritative_identity),
        }
    }

    pub fn rebind_target(
        attachment_identity: WorthQueryMutationAuthorityIdentity,
        prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
        target_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    ) -> Self {
        Self {
            family: WorthQueryNamingMutationFamily::RebindTarget,
            attachment_identity,
            prior_authoritative_identity: Some(prior_authoritative_identity),
            target_authoritative_identity: Some(target_authoritative_identity),
        }
    }

    pub fn remove(
        attachment_identity: WorthQueryMutationAuthorityIdentity,
        prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    ) -> Self {
        Self {
            family: WorthQueryNamingMutationFamily::Remove,
            attachment_identity,
            prior_authoritative_identity: Some(prior_authoritative_identity),
            target_authoritative_identity: None,
        }
    }

    pub fn family(&self) -> WorthQueryNamingMutationFamily {
        self.family
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryNamingMutationDenialKind {
    RequiresSameBatchTargetReference,
    RequiresExistingTruthBinding,
    RequiresDeleteFamily,
}

impl WorthQueryNamingMutationDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RequiresSameBatchTargetReference => "requires_same_batch_target_reference",
            Self::RequiresExistingTruthBinding => "requires_existing_truth_binding",
            Self::RequiresDeleteFamily => "requires_delete_family",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryNamingMutationDenial {
    family: WorthQueryNamingMutationFamily,
    attachment_identity: WorthQueryMutationAuthorityIdentity,
    kind: WorthQueryNamingMutationDenialKind,
    reason: String,
    denial_digest: Result<String, IdentityDerivationDenial>,
}

impl WorthQueryNamingMutationDenial {
    pub(crate) fn new(
        intent: &WorthQueryNamingMutationIntent,
        kind: WorthQueryNamingMutationDenialKind,
        reason: impl Into<String>,
    ) -> Self {
        let reason = reason.into();
        let denial_digest = (|| {
            Ok(worth_query_evidence_identity(
                WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), "naming-mutation-denial")
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                intent.family().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("attachment"),
                intent.attachment_identity().evidence_identity(),
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_value(WorthQueryEvidenceTag::new("reason"), &reason)
            .seal()?
            .as_str()
            .to_string())
        })();
        Self {
            family: intent.family(),
            attachment_identity: intent.attachment_identity().clone(),
            kind,
            reason,
            denial_digest,
        }
    }

    pub fn family(&self) -> WorthQueryNamingMutationFamily {
        self.family
    }

    pub fn attachment_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.attachment_identity
    }

    pub fn kind(&self) -> WorthQueryNamingMutationDenialKind {
        self.kind
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn denial_digest(&self) -> Result<&str, IdentityDerivationDenial> {
        Ok(self.denial_digest.as_ref().map_err(|cause| *cause)?)
    }
}

impl std::fmt::Display for WorthQueryNamingMutationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "naming mutation `{}` denied during {}: {}",
            self.attachment_identity.as_str(),
            self.kind.as_str(),
            self.reason
        )
    }
}

impl std::error::Error for WorthQueryNamingMutationDenial {}

pub(crate) fn admit_naming_intent(
    command: &WorthQueryWriteCommand,
) -> Result<(), WorthQueryNamingMutationDenial> {
    let Some(intent) = command.naming_intent() else {
        return Ok(());
    };
    match intent.family() {
        WorthQueryNamingMutationFamily::AttachNewTarget => {
            if command.mutation_family() == WorthQueryMutationFamily::Delete {
                return Err(WorthQueryNamingMutationDenial::new(
                    intent,
                    WorthQueryNamingMutationDenialKind::RequiresSameBatchTargetReference,
                    "naming attach-to-new-target requires an insert/update lane, not delete",
                ));
            }
            if command.mutation_family() != WorthQueryMutationFamily::Insert
                && command.symbolic_target_reference().is_none()
            {
                return Err(WorthQueryNamingMutationDenial::new(
                    intent,
                    WorthQueryNamingMutationDenialKind::RequiresSameBatchTargetReference,
                    "naming attach-to-new-target requires either the directly inserted target or a same-batch symbolic target reference",
                ));
            }
        }
        WorthQueryNamingMutationFamily::AttachExistingTarget => {
            if command.mutation_family() == WorthQueryMutationFamily::Delete {
                return Err(WorthQueryNamingMutationDenial::new(
                    intent,
                    WorthQueryNamingMutationDenialKind::RequiresExistingTruthBinding,
                    "naming attach-to-existing-target requires an insert/update lane, not delete",
                ));
            }
            if command.existing_truth_binding().is_none() {
                return Err(WorthQueryNamingMutationDenial::new(
                    intent,
                    WorthQueryNamingMutationDenialKind::RequiresExistingTruthBinding,
                    "naming attach-to-existing-target requires an existing-truth binding",
                ));
            }
        }
        WorthQueryNamingMutationFamily::RebindTarget => {
            if command.mutation_family() == WorthQueryMutationFamily::Delete {
                return Err(WorthQueryNamingMutationDenial::new(
                    intent,
                    WorthQueryNamingMutationDenialKind::RequiresExistingTruthBinding,
                    "naming rebind requires an insert/update lane, not delete",
                ));
            }
            if command.existing_truth_binding().is_none()
                && command.symbolic_target_reference().is_none()
            {
                return Err(WorthQueryNamingMutationDenial::new(
                    intent,
                    WorthQueryNamingMutationDenialKind::RequiresExistingTruthBinding,
                    "naming rebind requires an existing-truth binding or same-batch target reference",
                ));
            }
        }
        WorthQueryNamingMutationFamily::Remove => {
            if command.mutation_family() != WorthQueryMutationFamily::Delete {
                return Err(WorthQueryNamingMutationDenial::new(
                    intent,
                    WorthQueryNamingMutationDenialKind::RequiresDeleteFamily,
                    "naming removal requires a delete mutation family",
                ));
            }
        }
    }
    Ok(())
}
