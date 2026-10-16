use worth_ui::facade::query_binding::{
    UiProjectionObservation, WorthUiQueryViewIdentityError, WorthUiScalarProjectionActionEvidence,
    WorthUiScalarProjectionActionPreconditionDenial, WorthUiStatusOwnerError,
};
pub(crate) enum PlatformPulseQueryActionOutcome {
    Executed {
        evidence: WorthUiScalarProjectionActionEvidence,
        observation: UiProjectionObservation,
    },
    Denied {
        denial: WorthUiScalarProjectionActionPreconditionDenial,
        active_query_source_revision: u64,
        submitted_query_source_revision: u64,
    },
    Indeterminate(PlatformPulseQueryActionIndeterminate),
}

#[derive(Debug)]
pub(crate) enum PlatformPulseQueryActionIndeterminate {
    CurrentRevisionLost,
    DeniedActionCommitted,
    MutationPending,
}

impl std::fmt::Display for PlatformPulseQueryActionIndeterminate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentRevisionLost => {
                formatter.write_str("the current action lost its Query source revision")
            }
            Self::DeniedActionCommitted => {
                formatter.write_str("the revision-mismatched action unexpectedly committed")
            }
            Self::MutationPending => formatter.write_str("original Query operation is retained for fresh-request continuation"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum PlatformPulseQueryLifecycleDenial {
    InitialAlreadyIssued,
    PublicationAlreadyPending,
    PublicationNotPending,
    OwnerNotLive,
    MutationPending,
    RetainedMutation(Box<super::continuation::PlatformPulseQueryMutation>),
    ActionRequest(&'static str),
    Owner(WorthUiStatusOwnerError),
    Projection(WorthUiQueryViewIdentityError),
    ForeignPublication,
    UnsettledPublicationAtClose,
    IndeterminateAtClose,
}

impl std::fmt::Display for PlatformPulseQueryLifecycleDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitialAlreadyIssued => formatter.write_str("initial fact already issued"),
            Self::PublicationAlreadyPending => formatter.write_str("publication already pending"),
            Self::PublicationNotPending => formatter.write_str("publication is not pending"),
            Self::OwnerNotLive => formatter.write_str("live owner unavailable"),
            Self::MutationPending => formatter.write_str("original Query mutation retained; continue with a fresh request"),
            Self::RetainedMutation(_) => formatter.write_str("close retains the original Query mutation and its context"),
            Self::ActionRequest(denial) => write!(formatter, "action request: {denial}"),
            Self::Owner(denial) => write!(formatter, "Query owner: {denial}"),
            Self::Projection(denial) => write!(formatter, "projection: {denial:?}"),
            Self::ForeignPublication => formatter.write_str("foreign publication fact"),
            Self::UnsettledPublicationAtClose => {
                formatter.write_str("Query publication had not reached presentation at close")
            }
            Self::IndeterminateAtClose => {
                formatter.write_str("Query effect outcome remained indeterminate at close")
            }
        }
    }
}
