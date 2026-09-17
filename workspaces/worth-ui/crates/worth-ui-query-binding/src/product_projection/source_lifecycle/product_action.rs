use crate::{UiScalarProjectionObservation, UiScalarProjectionRegistration};

use super::{
    WorthUiScalarProjectionAdvance, WorthUiScalarProjectionAdvanceError,
    WorthUiScalarProjectionInstallation, WorthUiScalarProjectionLiveOwner,
    WorthUiScalarProjectionPublicationCompletion, WorthUiScalarProjectionSourceCloseError,
    WorthUiScalarProjectionSourceCloseReceipt,
};

mod execution;

pub use crate::product_projection::action_evidence::{
    WorthUiScalarProjectionActionEvidence, WorthUiScalarProjectionActionPreconditionDenial,
};

pub struct WorthUiScalarProjectionActionInstallation {
    inner: WorthUiScalarProjectionInstallation,
}

pub struct WorthUiScalarProjectionActionAdvance {
    inner: WorthUiScalarProjectionAdvance,
}

pub struct WorthUiScalarProjectionActionPublicationCompletion {
    inner: WorthUiScalarProjectionPublicationCompletion,
}

pub struct WorthUiScalarProjectionActionLiveOwner {
    inner: WorthUiScalarProjectionLiveOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiScalarProjectionActionRequest {
    source_revision: u64,
    status: String,
}

pub struct WorthUiScalarProjectionActionExecution {
    evidence: WorthUiScalarProjectionActionEvidence,
    advance: WorthUiScalarProjectionActionAdvance,
}

pub enum WorthUiScalarProjectionActionOutcome {
    Executed(WorthUiScalarProjectionActionExecution),
    Denied(WorthUiScalarProjectionActionDenied),
    Indeterminate(WorthUiScalarProjectionActionIndeterminate),
}

pub struct WorthUiScalarProjectionActionDenied {
    owner: WorthUiScalarProjectionActionLiveOwner,
    denial: WorthUiScalarProjectionActionPreconditionDenial,
    active_revision: u64,
    submitted_revision: u64,
}

pub struct WorthUiScalarProjectionActionIndeterminate {
    owner: WorthUiScalarProjectionLiveOwner,
    detail: String,
}

impl WorthUiScalarProjectionInstallation {
    pub fn into_action_installation(self) -> WorthUiScalarProjectionActionInstallation {
        WorthUiScalarProjectionActionInstallation { inner: self }
    }
}

impl WorthUiScalarProjectionActionInstallation {
    pub fn into_parts(
        self,
    ) -> (
        UiScalarProjectionRegistration,
        WorthUiScalarProjectionActionAdvance,
    ) {
        let (registration, advance) = self.inner.into_parts();
        (
            registration,
            WorthUiScalarProjectionActionAdvance::new(advance),
        )
    }

    pub fn into_parts_with_live_measurement_view(
        self,
        identity: impl Into<String>,
    ) -> Result<
        (
            UiScalarProjectionRegistration,
            WorthUiScalarProjectionActionAdvance,
            crate::WorthUiInstalledLiveQueryView,
        ),
        crate::WorthUiQueryViewDeclarationDenial,
    > {
        let (registration, advance, view) =
            self.inner.into_parts_with_live_measurement_view(identity)?;
        Ok((
            registration,
            WorthUiScalarProjectionActionAdvance::new(advance),
            view,
        ))
    }
}

impl WorthUiScalarProjectionActionAdvance {
    fn new(inner: WorthUiScalarProjectionAdvance) -> Self {
        Self { inner }
    }

    pub fn observation(&self) -> &crate::UiProjectionObservation {
        self.inner.observation()
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::UiProjectionObservation,
        WorthUiScalarProjectionActionPublicationCompletion,
    ) {
        let (observation, completion) = self.inner.into_parts();
        (
            observation,
            WorthUiScalarProjectionActionPublicationCompletion { inner: completion },
        )
    }
}

impl WorthUiScalarProjectionActionPublicationCompletion {
    #[allow(
        clippy::result_large_err,
        reason = "the cold publication denial returns the exact affine fact to its owner"
    )]
    pub fn admit_publication(
        self,
        observation: UiScalarProjectionObservation,
    ) -> Result<WorthUiScalarProjectionActionLiveOwner, UiScalarProjectionObservation> {
        self.inner
            .admit_publication(observation)
            .map(|inner| WorthUiScalarProjectionActionLiveOwner { inner })
    }
}

impl WorthUiScalarProjectionActionRequest {
    pub fn new(source_revision: u64, status: impl Into<String>) -> Result<Self, &'static str> {
        let status = status.into();
        super::super::WorthUiScalarProjectionSourceRecord::new(status.clone(), source_revision)?;
        Ok(Self {
            source_revision,
            status,
        })
    }

    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub fn status(&self) -> &str {
        &self.status
    }
}

impl WorthUiScalarProjectionActionLiveOwner {
    pub fn source_revision(&self) -> u64 {
        self.inner.revision
    }

    pub fn advance_source(
        self,
        record: super::super::WorthUiScalarProjectionSourceRecord,
    ) -> Result<WorthUiScalarProjectionActionAdvance, WorthUiScalarProjectionAdvanceError> {
        self.inner
            .advance(record)
            .map(WorthUiScalarProjectionActionAdvance::new)
    }

    pub fn execute_action(
        self,
        request: WorthUiScalarProjectionActionRequest,
    ) -> WorthUiScalarProjectionActionOutcome {
        if request.source_revision != self.inner.revision {
            return WorthUiScalarProjectionActionOutcome::Denied(
                WorthUiScalarProjectionActionDenied {
                    active_revision: self.inner.revision,
                    submitted_revision: request.source_revision,
                    denial: WorthUiScalarProjectionActionPreconditionDenial::SourceRevisionMismatch,
                    owner: self,
                },
            );
        }
        execution::execute_query_action(self.inner, request)
    }

    #[allow(
        clippy::result_large_err,
        reason = "cold shutdown preserves Query's exact terminal denial topology"
    )]
    pub fn close(
        self,
    ) -> Result<WorthUiScalarProjectionSourceCloseReceipt, WorthUiScalarProjectionSourceCloseError>
    {
        self.inner.close()
    }
}

impl WorthUiScalarProjectionActionExecution {
    pub fn evidence(&self) -> &WorthUiScalarProjectionActionEvidence {
        &self.evidence
    }

    pub fn observation(&self) -> &crate::UiProjectionObservation {
        self.advance.observation()
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthUiScalarProjectionActionEvidence,
        WorthUiScalarProjectionActionAdvance,
    ) {
        (self.evidence, self.advance)
    }
}

impl WorthUiScalarProjectionActionDenied {
    pub const fn denial(&self) -> WorthUiScalarProjectionActionPreconditionDenial {
        self.denial
    }

    pub fn active_revision(&self) -> u64 {
        self.active_revision
    }

    pub fn submitted_revision(&self) -> u64 {
        self.submitted_revision
    }

    pub fn into_owner(self) -> WorthUiScalarProjectionActionLiveOwner {
        self.owner
    }
}

impl WorthUiScalarProjectionActionIndeterminate {
    pub fn detail(&self) -> &str {
        &self.detail
    }

    #[allow(
        clippy::result_large_err,
        reason = "cold indeterminate shutdown preserves Query's exact terminal denial topology"
    )]
    pub fn close(
        self,
    ) -> Result<WorthUiScalarProjectionSourceCloseReceipt, WorthUiScalarProjectionSourceCloseError>
    {
        self.owner.close()
    }
}
