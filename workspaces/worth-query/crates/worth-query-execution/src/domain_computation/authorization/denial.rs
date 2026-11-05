//! Typed authorization denial topology.

use crate::domain_computation::application_outcome_identity::WorthQueryApplicationOutcomeIdentity;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
mod capacity;
mod workflow_actor;
use workflow_actor::WorkflowActorNativeDenial;
mod relational_read;
pub(super) use relational_read::denial as relational_read_denial;
pub(super) use relational_read::kernel_stop_kind;
pub(in crate::domain_computation) use relational_read::principal_currentness;



/// Why an admitted operation's own request authority lapsed after admission.
///
/// These are the only causes current-authority revalidation can report, so a
/// later phase that must answer with its own refusal matches every cause
/// exactly rather than folding an open authorization kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryAdmissionLapse {
    /// The admitted request was cancelled.
    Cancelled,
    /// The admitted request reached its deadline.
    DeadlineExceeded,
    /// The admitted principal's authentication expired.
    AuthenticationExpired,
}

impl WorthQueryAdmissionLapse {
    pub(in crate::domain_computation) const fn denial_kind(
        self,
    ) -> WorthQueryOperationAuthorizationDenialKind {
        match self {
            Self::Cancelled => WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
            )),
            Self::DeadlineExceeded => WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
                crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
            )),
            Self::AuthenticationExpired => WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ExpiredAuthentication,
            ),
        }
    }
}

mod explanation;
pub use explanation::WorthQueryApplicationAuthorizationExplanationCause;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind;

/// Process-local identity of one authorization denial, for correlation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryOperationAuthorizationDenialIdentity(WorthQueryApplicationOutcomeIdentity);

impl WorthQueryOperationAuthorizationDenialIdentity {
    fn mint() -> Option<Self> {
        WorthQueryApplicationOutcomeIdentity::mint().map(Self)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

/// Refusal by authorization to let an operation or query proceed.
///
/// Decided during admission, before any effect. [`Self::kind`] is the first
/// cause and [`Self::causes`] lists every cause in order;
/// [`Self::explanation_cause`] gives a user-facing cause when one applies, and
/// [`Self::subject`] names what was being authorized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationAuthorizationDenial {
    identity: Option<WorthQueryOperationAuthorizationDenialIdentity>,
    // The execution kind carries resource details; keep those details off
    // the stack of every surrounding workflow and recovery refusal.
    kind: Box<WorthQueryOperationAuthorizationDenialKind>,
    causes: Vec<WorthQueryOperationAuthorizationDenialKind>,
    explanation_cause: Option<WorthQueryApplicationAuthorizationExplanationCause>,
    subject: String,
    workflow_actor_cause: Option<WorkflowActorNativeDenial>,
}

impl WorthQueryOperationAuthorizationDenial {
    pub(in crate::domain_computation) fn new(
        kind: WorthQueryOperationAuthorizationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            identity: WorthQueryOperationAuthorizationDenialIdentity::mint(),
            kind: Box::new(kind),
            causes: vec![kind],
            explanation_cause: WorthQueryApplicationAuthorizationExplanationCause::from_denial_kind(
                kind,
            ),
            subject: subject.into(),
            workflow_actor_cause: None,
        }
    }

    pub(super) fn from_ordered_causes(
        causes: impl IntoIterator<Item = WorthQueryOperationAuthorizationDenialKind>,
        subject: impl Into<String>,
    ) -> Self {
        let mut causes = causes.into_iter().collect::<Vec<_>>();
        if causes.is_empty() {
            causes.push(WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::InconsistentDecision,
            ));
        }
        let kind = causes[0];
        Self {
            identity: WorthQueryOperationAuthorizationDenialIdentity::mint(),
            kind: Box::new(kind),
            causes,
            explanation_cause: WorthQueryApplicationAuthorizationExplanationCause::from_denial_kind(
                kind,
            ),
            subject: subject.into(),
            workflow_actor_cause: None,
        }
    }

    pub(in crate::domain_computation) fn inconsistent(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::InconsistentDecision,
            ),
            subject,
        )
    }

    pub const fn kind(&self) -> WorthQueryOperationAuthorizationDenialKind {
        *self.kind
    }

    pub const fn identity(&self) -> Option<WorthQueryOperationAuthorizationDenialIdentity> {
        self.identity
    }

    pub fn causes(&self) -> &[WorthQueryOperationAuthorizationDenialKind] {
        &self.causes
    }

    pub const fn explanation_cause(
        &self,
    ) -> Option<WorthQueryApplicationAuthorizationExplanationCause> {
        self.explanation_cause
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryOperationAuthorizationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "operation authorization denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryOperationAuthorizationDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.workflow_actor_source_denial()
            .map(|cause| cause as &(dyn std::error::Error + 'static))
    }
}

pub(super) fn product_security_basis_denial(
    denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::from(denial),
        subject,
    )
}


mod court_doors_for_test;
