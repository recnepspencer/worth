use crate::application::{
    WorthQueryDeclarationEntryOrchestrationStage, WorthQueryDeclarationInput,
    WorthQueryDomainEntryMarker,
};
use crate::binding_pipeline::WorthQueryBindingLinkedArtifacts;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary_outcome::{
    WorthQueryOrdinaryCheckedTopology, WorthQueryOrdinaryContributionComposedCheckedTopologyKind,
    WorthQueryOrdinaryNextStep, WorthQueryOrdinaryOutcome, WorthQueryOrdinaryPosture,
};

use super::artifact::WorthQueryContributionComposedOrchestration;
use super::aspect::WorthQueryContributionComposedDeclarationAspectRecord;
use super::composition::WorthQueryContributionComposedStop;
use super::intent_result::WorthQueryContributionComposedIntentRequestDescriptor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryContributionComposedOrchestrationCheckedKind {
    Deferred,
    DeclarationDenied,
    ContributionDenied,
    Stale,
    RebindRequired,
    Unsupported,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContributionComposedOrchestrationPosture<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    kind: WorthQueryContributionComposedOrchestrationCheckedKind,
    stop: WorthQueryContributionComposedStop,
    stop_stage: WorthQueryDeclarationEntryOrchestrationStage,
    reason: String,
    linked_artifacts: WorthQueryBindingLinkedArtifacts,
    contribution_digest: Option<String>,
    declaration_aspect_record: Option<WorthQueryContributionComposedDeclarationAspectRecord>,
    primary_intent_descriptor: Option<WorthQueryContributionComposedIntentRequestDescriptor>,
    _marker: std::marker::PhantomData<(D, I)>,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryContributionComposedOrchestrationPosture<D, I>
{
    pub fn new(
        kind: WorthQueryContributionComposedOrchestrationCheckedKind,
        stop: WorthQueryContributionComposedStop,
        stop_stage: WorthQueryDeclarationEntryOrchestrationStage,
        reason: impl Into<String>,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
        contribution_digest: Option<String>,
        declaration_aspect_record: Option<WorthQueryContributionComposedDeclarationAspectRecord>,
        primary_intent_descriptor: Option<WorthQueryContributionComposedIntentRequestDescriptor>,
    ) -> Self {
        Self {
            kind,
            stop,
            stop_stage,
            reason: reason.into(),
            linked_artifacts,
            contribution_digest,
            declaration_aspect_record,
            primary_intent_descriptor,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn kind(&self) -> WorthQueryContributionComposedOrchestrationCheckedKind {
        self.kind
    }

    pub fn stop(&self) -> WorthQueryContributionComposedStop {
        self.stop
    }

    pub fn stop_stage(&self) -> WorthQueryDeclarationEntryOrchestrationStage {
        self.stop_stage
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn linked_artifacts(&self) -> &WorthQueryBindingLinkedArtifacts {
        &self.linked_artifacts
    }

    pub fn contribution_digest(&self) -> Option<&str> {
        self.contribution_digest.as_deref()
    }

    pub fn declaration_aspect_record(
        &self,
    ) -> Option<&WorthQueryContributionComposedDeclarationAspectRecord> {
        self.declaration_aspect_record.as_ref()
    }

    pub fn primary_intent_descriptor(
        &self,
    ) -> Option<&WorthQueryContributionComposedIntentRequestDescriptor> {
        self.primary_intent_descriptor.as_ref()
    }
}

pub enum WorthQueryContributionComposedOrchestrationOutcome<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Bound(WorthQueryContributionComposedOrchestration<D, I>),
    Deferred(WorthQueryContributionComposedOrchestrationPosture<D, I>),
    DeclarationDenied(WorthQueryContributionComposedOrchestrationPosture<D, I>),
    ContributionDenied(WorthQueryContributionComposedOrchestrationPosture<D, I>),
    Stale(WorthQueryContributionComposedOrchestrationPosture<D, I>),
    RebindRequired(WorthQueryContributionComposedOrchestrationPosture<D, I>),
    Unsupported(WorthQueryContributionComposedOrchestrationPosture<D, I>),
    Failed(WorthQueryContributionComposedOrchestrationPosture<D, I>),
}

pub type WorthQueryContributionComposedOrchestrationChecked<D, I> =
    WorthQueryContributionComposedOrchestrationOutcome<D, I>;

pub(crate) fn ordinary_outcome_from_contribution_composed_checked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    checked: WorthQueryContributionComposedOrchestrationChecked<D, I>,
) -> Result<
    WorthQueryOrdinaryOutcome<WorthQueryContributionComposedOrchestration<D, I>>,
    IdentityDerivationDenial,
> {
    Ok(match checked {
        WorthQueryContributionComposedOrchestrationOutcome::EvidenceIdentityDerivationDenied(
            cause,
        ) => return Err(cause),
        WorthQueryContributionComposedOrchestrationOutcome::Bound(value) => {
            WorthQueryOrdinaryOutcome::Bound(value)
        }
        WorthQueryContributionComposedOrchestrationOutcome::Deferred(value) => {
            WorthQueryOrdinaryOutcome::Deferred(ordinary_posture(
                value,
                WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Deferred,
            ))
        }
        WorthQueryContributionComposedOrchestrationOutcome::DeclarationDenied(value) => {
            WorthQueryOrdinaryOutcome::Denied(ordinary_posture(
                value,
                WorthQueryOrdinaryContributionComposedCheckedTopologyKind::DeclarationDenied,
            ))
        }
        WorthQueryContributionComposedOrchestrationOutcome::ContributionDenied(value) => {
            WorthQueryOrdinaryOutcome::Denied(ordinary_posture(
                value,
                WorthQueryOrdinaryContributionComposedCheckedTopologyKind::ContributionDenied,
            ))
        }
        WorthQueryContributionComposedOrchestrationOutcome::Stale(value) => {
            WorthQueryOrdinaryOutcome::Stale(ordinary_posture(
                value,
                WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Stale,
            ))
        }
        WorthQueryContributionComposedOrchestrationOutcome::RebindRequired(value) => {
            WorthQueryOrdinaryOutcome::RebindRequired(ordinary_posture(
                value,
                WorthQueryOrdinaryContributionComposedCheckedTopologyKind::RebindRequired,
            ))
        }
        WorthQueryContributionComposedOrchestrationOutcome::Unsupported(value) => {
            WorthQueryOrdinaryOutcome::Unsupported(ordinary_posture(
                value,
                WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Unsupported,
            ))
        }
        WorthQueryContributionComposedOrchestrationOutcome::Failed(value) => {
            WorthQueryOrdinaryOutcome::Failed(ordinary_posture(
                value,
                WorthQueryOrdinaryContributionComposedCheckedTopologyKind::Failed,
            ))
        }
    })
}

fn ordinary_posture<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    posture: WorthQueryContributionComposedOrchestrationPosture<D, I>,
    kind: WorthQueryOrdinaryContributionComposedCheckedTopologyKind,
) -> WorthQueryOrdinaryPosture {
    WorthQueryOrdinaryPosture::new(
        posture.reason().to_string(),
        ordinary_posture_kind(posture.kind()),
        ordinary_next_step(posture.kind()),
        WorthQueryOrdinaryCheckedTopology::contribution_composed(
            kind,
            posture.linked_artifacts().clone(),
            posture.contribution_digest().map(str::to_string),
        ),
    )
}

fn ordinary_posture_kind(
    kind: WorthQueryContributionComposedOrchestrationCheckedKind,
) -> crate::ordinary_outcome::WorthQueryOrdinaryPostureKind {
    match kind {
        WorthQueryContributionComposedOrchestrationCheckedKind::Deferred => {
            crate::ordinary_outcome::WorthQueryOrdinaryPostureKind::Deferred
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::DeclarationDenied
        | WorthQueryContributionComposedOrchestrationCheckedKind::ContributionDenied => {
            crate::ordinary_outcome::WorthQueryOrdinaryPostureKind::Denied
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Stale => {
            crate::ordinary_outcome::WorthQueryOrdinaryPostureKind::Stale
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::RebindRequired => {
            crate::ordinary_outcome::WorthQueryOrdinaryPostureKind::RebindRequired
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Unsupported => {
            crate::ordinary_outcome::WorthQueryOrdinaryPostureKind::Unsupported
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Failed => {
            crate::ordinary_outcome::WorthQueryOrdinaryPostureKind::Failed
        }
    }
}

fn ordinary_next_step(
    kind: WorthQueryContributionComposedOrchestrationCheckedKind,
) -> WorthQueryOrdinaryNextStep {
    match kind {
        WorthQueryContributionComposedOrchestrationCheckedKind::Deferred => {
            WorthQueryOrdinaryNextStep::RetryLater
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::DeclarationDenied
        | WorthQueryContributionComposedOrchestrationCheckedKind::ContributionDenied => {
            WorthQueryOrdinaryNextStep::InspectCheckedLane
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Stale => {
            WorthQueryOrdinaryNextStep::RefreshBasis
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::RebindRequired => {
            WorthQueryOrdinaryNextStep::RebindContext
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Unsupported => {
            WorthQueryOrdinaryNextStep::CheckSupport
        }
        WorthQueryContributionComposedOrchestrationCheckedKind::Failed => {
            WorthQueryOrdinaryNextStep::EscalateFailure
        }
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryContributionComposedOrchestrationOutcome<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
