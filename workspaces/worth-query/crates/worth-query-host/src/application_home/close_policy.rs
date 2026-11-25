//! Fund Home retirement without losing the owner on lease refusal.
use super::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeCloseRefused,
    WorthQueryApplicationHomeClosed, WorthQueryApplicationHomeOpenRefused,
    WorthQueryApplicationHomeOpeningCloseRefused, WorthQueryApplicationHomeOpeningClosed,
};
use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseDenial, LeaseRequest,
};
use worth_foundational::facade::ExecutionRequestPolicy;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaComposition,
};

/// A refusal retains the opening owner or the native close continuation.
#[must_use = "a close refusal retains its owner"]
pub enum WorthQueryHomeClosePolicyRefused<Owner, CloseRefused> {
    Admission {
        owner: Box<Owner>,
        cause: LeaseDenial,
    },
    Close(CloseRefused),
}

impl<Owner, CloseRefused: std::fmt::Debug> std::fmt::Debug
    for WorthQueryHomeClosePolicyRefused<Owner, CloseRefused>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admission { cause, .. } => {
                formatter.debug_tuple("Admission").field(cause).finish()
            }
            Self::Close(cause) => formatter.debug_tuple("Close").field(cause).finish(),
        }
    }
}

impl<Schema: ApplicationSchema, Program: 'static> WorthQueryApplicationHome<Schema, Program> {
    /// Closes this Home using an owned serial lease with the requested budgets.
    pub fn close_with_policy(
        self,
        policy: ExecutionRequestPolicy,
    ) -> Result<
        WorthQueryApplicationHomeClosed,
        WorthQueryHomeClosePolicyRefused<
            Self,
            WorthQueryApplicationHomeCloseRefused<Schema, Program>,
        >,
    > {
        with_close_policy(self, policy, |home, request| home.close(request))
    }
}

impl<Schema: ApplicationSchema, Program: 'static>
    WorthQueryApplicationHomeCloseRefused<Schema, Program>
{
    /// Re-admits a serial lease and continues the same Home close.
    pub fn try_close_with_policy(
        self,
        policy: ExecutionRequestPolicy,
    ) -> Result<WorthQueryApplicationHomeClosed, WorthQueryHomeClosePolicyRefused<Self, Self>> {
        with_close_policy(self, policy, |home, request| home.try_close(request))
    }
}

impl<Schema: ApplicationSchemaComposition, Program: 'static>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    /// Closes the exact Home retained by a refused opening under an owned serial lease.
    pub fn close_with_policy(
        self,
        policy: ExecutionRequestPolicy,
    ) -> Result<
        WorthQueryApplicationHomeOpeningClosed,
        WorthQueryHomeClosePolicyRefused<
            Self,
            WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
        >,
    > {
        with_close_policy(self, policy, |opening, request| opening.close(request))
    }
}

impl<Schema: ApplicationSchemaComposition, Program: 'static>
    WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>
{
    /// Re-admits a serial lease and continues the same refused-opening close.
    pub fn try_close_with_policy(
        self,
        policy: ExecutionRequestPolicy,
    ) -> Result<WorthQueryApplicationHomeOpeningClosed, WorthQueryHomeClosePolicyRefused<Self, Self>>
    {
        with_close_policy(self, policy, |opening, request| opening.try_close(request))
    }
}

fn with_close_policy<Owner, Closed, CloseRefused>(
    owner: Owner,
    policy: ExecutionRequestPolicy,
    close: impl FnOnce(Owner, &ExecutionRequest<'_, '_>) -> Result<Closed, CloseRefused>,
) -> Result<Closed, WorthQueryHomeClosePolicyRefused<Owner, CloseRefused>> {
    use WorthQueryHomeClosePolicyRefused as Refused;
    let lease = match ExecutionResourceLease::serial(LeaseRequest {
        policy,
        cancellation: CancellationToken::new(),
        deadline: None,
    }) {
        Ok(lease) => lease,
        Err(cause) => {
            return Err(Refused::Admission {
                owner: Box::new(owner),
                cause,
            })
        }
    };
    close(owner, &ExecutionRequest::leased(&lease)).map_err(Refused::Close)
}
