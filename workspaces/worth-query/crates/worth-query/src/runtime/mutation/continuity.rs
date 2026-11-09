use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryWorkspaceError;

use super::super::{WorthQueryMutationFamily, WorthQueryWriteCommand};
use crate::runtime::WorthQueryMutationAuthorityIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryContinuityMutationFamily {
    RebindExistingTarget,
    SplitExistingTarget,
}

impl WorthQueryContinuityMutationFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RebindExistingTarget => "rebind_existing_target",
            Self::SplitExistingTarget => "split_existing_target",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryContinuityMutationOutcomeClass {
    ContinuesAsSingleSuccessor,
    ContinuesAsSplitSuccessors,
    ContinuesViaTruthLoweredCanonicalMergeSuccessor,
}

impl WorthQueryContinuityMutationOutcomeClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ContinuesAsSingleSuccessor => "continues_as_single_successor",
            Self::ContinuesAsSplitSuccessors => "continues_as_split_successors",
            Self::ContinuesViaTruthLoweredCanonicalMergeSuccessor => {
                "continues_via_truth_lowered_canonical_merge_successor"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContinuityMutationIntent {
    family: WorthQueryContinuityMutationFamily,
    outcome_class: WorthQueryContinuityMutationOutcomeClass,
    prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    successor_authoritative_identities: Vec<WorthQueryMutationAuthorityIdentity>,
}

impl WorthQueryContinuityMutationIntent {
    pub fn rebind_existing_target(
        prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
        successor_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Self::new(
            WorthQueryContinuityMutationFamily::RebindExistingTarget,
            WorthQueryContinuityMutationOutcomeClass::ContinuesAsSingleSuccessor,
            prior_authoritative_identity,
            [successor_authoritative_identity],
        )
    }

    pub fn rebind_merge_successor(
        prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
        successor_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Self::new(
            WorthQueryContinuityMutationFamily::RebindExistingTarget,
            WorthQueryContinuityMutationOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor,
            prior_authoritative_identity,
            [successor_authoritative_identity],
        )
    }

    pub fn split_existing_target<I>(
        prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
        successor_authoritative_identities: I,
    ) -> Result<Self, WorthQueryWorkspaceError>
    where
        I: IntoIterator<Item = WorthQueryMutationAuthorityIdentity>,
    {
        Self::new(
            WorthQueryContinuityMutationFamily::SplitExistingTarget,
            WorthQueryContinuityMutationOutcomeClass::ContinuesAsSplitSuccessors,
            prior_authoritative_identity,
            successor_authoritative_identities,
        )
    }

    fn new<I>(
        family: WorthQueryContinuityMutationFamily,
        outcome_class: WorthQueryContinuityMutationOutcomeClass,
        prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
        successor_authoritative_identities: I,
    ) -> Result<Self, WorthQueryWorkspaceError>
    where
        I: IntoIterator<Item = WorthQueryMutationAuthorityIdentity>,
    {
        let successor_authoritative_identities = successor_authoritative_identities
            .into_iter()
            .collect::<Vec<_>>();
        if successor_authoritative_identities.is_empty() {
            return Err(WorthQueryWorkspaceError::new(
                "successor authoritative identity set may not be empty",
            ));
        }
        if family == WorthQueryContinuityMutationFamily::SplitExistingTarget
            && successor_authoritative_identities.len() < 2
        {
            return Err(WorthQueryWorkspaceError::new(
                "split-successor continuity requires at least two successor authoritative identities",
            ));
        }
        Ok(Self {
            family,
            outcome_class,
            prior_authoritative_identity,
            successor_authoritative_identities,
        })
    }

    pub fn family(&self) -> WorthQueryContinuityMutationFamily {
        self.family
    }

    pub fn outcome_class(&self) -> WorthQueryContinuityMutationOutcomeClass {
        self.outcome_class
    }

    pub fn prior_authoritative_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.prior_authoritative_identity
    }

    pub fn successor_authoritative_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        self.successor_authoritative_identities
            .first()
            .expect("continuity intent must retain at least one successor authoritative identity")
    }

    pub fn successor_authoritative_identities(&self) -> &[WorthQueryMutationAuthorityIdentity] {
        &self.successor_authoritative_identities
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryContinuityMutationDenialKind {
    RequiresExistingTruthBinding,
    RequiresUpdateFamily,
    RequiresAuthoritativeLane,
}

impl WorthQueryContinuityMutationDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RequiresExistingTruthBinding => "requires_existing_truth_binding",
            Self::RequiresUpdateFamily => "requires_update_family",
            Self::RequiresAuthoritativeLane => "requires_authoritative_lane",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContinuityMutationDenial {
    family: WorthQueryContinuityMutationFamily,
    kind: WorthQueryContinuityMutationDenialKind,
    prior_authoritative_identity: WorthQueryMutationAuthorityIdentity,
    successor_authoritative_identities: Vec<WorthQueryMutationAuthorityIdentity>,
    basis_binding_digest: Option<crate::runtime::WorthQueryMutationEvidenceDigest>,
    reason: String,
    denial_digest: Result<String, IdentityDerivationDenial>,
}

impl WorthQueryContinuityMutationDenial {
    pub(crate) fn new(
        intent: &WorthQueryContinuityMutationIntent,
        existing_truth_binding: Option<&crate::runtime::WorthQueryExistingTruthTargetBinding>,
        kind: WorthQueryContinuityMutationDenialKind,
        reason: impl Into<String>,
    ) -> Self {
        let reason = reason.into();
        let basis_binding_identity =
            existing_truth_binding.map(|binding| binding.binding_evidence_identity());
        let basis_binding_digest = basis_binding_identity
            .map(|identity| {
                crate::runtime::WorthQueryMutationEvidenceDigest::source_identity(
                    "continuity-basis-binding",
                    identity,
                )
            })
            .transpose()
            .expect("fixed continuity basis role and a sealed binding token fit digest bounds");
        let denial_digest = (|| {
            Ok(worth_query_evidence_identity(
                WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "continuity-mutation-denial",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                intent.family().as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_shape(
                WorthQueryEvidenceTag::new("outcome"),
                intent.outcome_class().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("prior"),
                intent.prior_authoritative_identity().evidence_identity(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("successor"),
                intent
                    .successor_authoritative_identities()
                    .iter()
                    .map(WorthQueryMutationAuthorityIdentity::evidence_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("basis_binding"),
                basis_binding_identity,
            )
            .field_value(WorthQueryEvidenceTag::new("reason"), &reason)
            .seal()?
            .as_str()
            .to_string())
        })();
        Self {
            family: intent.family(),
            kind,
            prior_authoritative_identity: intent.prior_authoritative_identity().clone(),
            successor_authoritative_identities: intent
                .successor_authoritative_identities()
                .to_vec(),
            basis_binding_digest,
            reason,
            denial_digest,
        }
    }

    pub fn family(&self) -> WorthQueryContinuityMutationFamily {
        self.family
    }

    pub fn kind(&self) -> WorthQueryContinuityMutationDenialKind {
        self.kind
    }

    pub fn prior_authoritative_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.prior_authoritative_identity
    }

    pub fn successor_authoritative_identity(&self) -> Option<&WorthQueryMutationAuthorityIdentity> {
        match self.successor_authoritative_identities.as_slice() {
            [only] => Some(only),
            _ => None,
        }
    }

    pub fn successor_authoritative_identities(&self) -> &[WorthQueryMutationAuthorityIdentity] {
        &self.successor_authoritative_identities
    }

    pub fn basis_binding_digest(&self) -> Option<&str> {
        self.basis_binding_digest
            .as_ref()
            .map(crate::runtime::WorthQueryMutationEvidenceDigest::as_str)
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn denial_digest(&self) -> Result<&str, IdentityDerivationDenial> {
        self.denial_digest.as_deref().map_err(|cause| *cause)
    }
}

impl std::fmt::Display for WorthQueryContinuityMutationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "continuity mutation `{}` -> ",
            self.prior_authoritative_identity.as_str(),
        )?;
        write!(f, "[")?;
        for (index, successor) in self.successor_authoritative_identities.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "`{}`", successor.as_str())?;
        }
        write!(f, "] denied during {}: {}", self.kind.as_str(), self.reason)
    }
}

impl std::error::Error for WorthQueryContinuityMutationDenial {}

pub(crate) fn admit_continuity_intent(
    command: &WorthQueryWriteCommand,
) -> Result<(), WorthQueryContinuityMutationDenial> {
    let Some(intent) = command.continuity_intent() else {
        return Ok(());
    };
    if command.mutation_family() != WorthQueryMutationFamily::Update {
        return Err(WorthQueryContinuityMutationDenial::new(
            intent,
            command.existing_truth_binding(),
            WorthQueryContinuityMutationDenialKind::RequiresUpdateFamily,
            "continuity-aware mutation currently requires an update family",
        ));
    }
    if command.existing_truth_binding().is_none() {
        return Err(WorthQueryContinuityMutationDenial::new(
            intent,
            None,
            WorthQueryContinuityMutationDenialKind::RequiresExistingTruthBinding,
            "continuity-aware mutation requires an existing-truth binding",
        ));
    }
    Ok(())
}
