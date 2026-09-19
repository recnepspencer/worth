//! Bank-owned publication and live-output shapes for account activity.

use bank_domain::queries::{
    AccountActivityQuery, AccountActivityQueryParameters, AccountActivityQueryResult,
};
use bank_domain::schema::{Account, BankSchema};
use worth_query_host::facade::{
    primary_graph::{
        WorthQueryApplicationContinuationPageResult, WorthQueryApplicationLiveOutcome,
        WorthQueryApplicationQueryContinuation,
    },
    publication::domain_computation::{
        publish_application_result, WorthQueryApplicationQueryPublicationReceipt,
        WorthQueryPublishedApplicationResult,
    },
};

use crate::{
    BankApplicationLiveCauseDenial, BankApplicationLiveCloseOutcome, BankApplicationLiveOverflow,
    BankApplicationLiveProjectionDenial, BankAuthorizationDenial,
};

pub type BankAccountActivityQueryResult =
    WorthQueryPublishedApplicationResult<AccountActivityQuery, AccountActivityQueryResult>;
pub type BankAccountActivityHistoricalResult =
    WorthQueryPublishedApplicationResult<AccountActivityQuery, AccountActivityQueryResult>;
type QueryAccountActivityContinuation = WorthQueryApplicationQueryContinuation<
    BankSchema,
    AccountActivityQuery,
    AccountActivityQueryParameters,
    AccountActivityQueryResult,
    Account,
>;

/// Opaque Bank authority for resuming one account-activity query.
///
/// ```compile_fail,E0451
/// use bank_server::BankAccountActivityContinuation;
///
/// let _ = BankAccountActivityContinuation { query: panic!("foreign continuation") };
/// ```
///
/// The wrapper cannot be coerced to Query's continuation authority:
///
/// ```compile_fail,E0308
/// use bank_domain::queries::{
///     AccountActivityQuery, AccountActivityQueryParameters, AccountActivityQueryResult,
/// };
/// use bank_domain::schema::{Account, BankSchema};
/// use bank_server::BankAccountActivityContinuation;
/// use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryContinuation;
///
/// type RawContinuation = WorthQueryApplicationQueryContinuation<
///     BankSchema,
///     AccountActivityQuery,
///     AccountActivityQueryParameters,
///     AccountActivityQueryResult,
///     Account,
/// >;
///
/// fn raw_query_continuation(
///     continuation: &BankAccountActivityContinuation,
/// ) -> &RawContinuation {
///     continuation
/// }
/// ```
///
/// Nor does it expose the former raw-authority accessor:
///
/// ```compile_fail,E0599
/// use bank_domain::queries::{
///     AccountActivityQuery, AccountActivityQueryParameters, AccountActivityQueryResult,
/// };
/// use bank_domain::schema::{Account, BankSchema};
/// use bank_server::BankAccountActivityContinuation;
/// use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryContinuation;
///
/// type RawContinuation = WorthQueryApplicationQueryContinuation<
///     BankSchema,
///     AccountActivityQuery,
///     AccountActivityQueryParameters,
///     AccountActivityQueryResult,
///     Account,
/// >;
///
/// fn raw_query_continuation(
///     continuation: &BankAccountActivityContinuation,
/// ) -> &RawContinuation {
///     continuation.query()
/// }
/// ```
pub struct BankAccountActivityContinuation {
    query: QueryAccountActivityContinuation,
}

pub struct BankAccountActivityPageResult {
    published:
        WorthQueryPublishedApplicationResult<AccountActivityQuery, AccountActivityQueryResult>,
    continuation: Option<BankAccountActivityContinuation>,
}

pub struct BankAccountActivityLiveUpdate {
    published:
        WorthQueryPublishedApplicationResult<AccountActivityQuery, AccountActivityQueryResult>,
}

#[derive(Debug)]
pub enum BankAccountActivityLiveOutcome {
    Delivered(BankAccountActivityLiveUpdate),
    Pending,
    Overflow(BankApplicationLiveOverflow),
    AuthorizationDenied(BankAuthorizationDenial),
    StalePrincipal,
    StaleScope,
    ProjectionDenied(BankApplicationLiveProjectionDenial),
    CauseDenied(BankApplicationLiveCauseDenial),
    Cancelled,
    DeadlineExceeded,
    Closed,
    Unavailable,
}

impl std::fmt::Debug for BankAccountActivityPageResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BankAccountActivityPageResult")
            .field("row_count", &self.published.rows().len())
            .field("has_continuation", &self.continuation.is_some())
            .field("receipt", self.published.receipt())
            .finish()
    }
}

impl std::fmt::Debug for BankAccountActivityLiveUpdate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BankAccountActivityLiveUpdate")
            .field("row_count", &self.published.rows().len())
            .field("receipt", self.published.receipt())
            .finish()
    }
}

impl BankAccountActivityPageResult {
    pub fn rows(&self) -> &[AccountActivityQueryResult] {
        self.published.rows()
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryPublicationReceipt {
        self.published.receipt()
    }

    pub const fn continuation(&self) -> Option<&BankAccountActivityContinuation> {
        self.continuation.as_ref()
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPublishedApplicationResult<AccountActivityQuery, AccountActivityQueryResult>,
        Option<BankAccountActivityContinuation>,
    ) {
        (self.published, self.continuation)
    }
}

impl BankAccountActivityContinuation {
    const fn from_query(query: QueryAccountActivityContinuation) -> Self {
        Self { query }
    }

    pub(super) fn into_query(self) -> QueryAccountActivityContinuation {
        self.query
    }
}

impl BankAccountActivityLiveUpdate {
    pub fn result(&self) -> &AccountActivityQueryResult {
        self.published
            .rows()
            .first()
            .expect("Query live delivery always contains its one projected result")
    }

    pub fn rows(&self) -> &[AccountActivityQueryResult] {
        self.published.rows()
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryPublicationReceipt {
        self.published.receipt()
    }
}

pub(super) fn publish_page(
    page: WorthQueryApplicationContinuationPageResult<
        BankSchema,
        AccountActivityQuery,
        AccountActivityQueryParameters,
        AccountActivityQueryResult,
        Account,
    >,
) -> BankAccountActivityPageResult {
    let (admitted, continuation) = page.into_admitted_disclosed();
    BankAccountActivityPageResult {
        published: publish_application_result(admitted),
        continuation: continuation.map(BankAccountActivityContinuation::from_query),
    }
}

pub(super) fn publish_live_outcome(
    outcome: WorthQueryApplicationLiveOutcome<AccountActivityQuery, AccountActivityQueryResult>,
) -> BankAccountActivityLiveOutcome {
    match outcome {
        WorthQueryApplicationLiveOutcome::Delivered(update) => {
            let (_, admitted) = update.into_admitted_disclosed();
            BankAccountActivityLiveOutcome::Delivered(BankAccountActivityLiveUpdate {
                published: publish_application_result(admitted),
            })
        }
        WorthQueryApplicationLiveOutcome::Pending => BankAccountActivityLiveOutcome::Pending,
        WorthQueryApplicationLiveOutcome::Overflow(overflow) => {
            BankAccountActivityLiveOutcome::Overflow(BankApplicationLiveOverflow::from_query(
                overflow,
            ))
        }
        WorthQueryApplicationLiveOutcome::AuthorizationDenied(denial) => {
            BankAccountActivityLiveOutcome::AuthorizationDenied(
                BankAuthorizationDenial::from_query(*denial),
            )
        }
        WorthQueryApplicationLiveOutcome::StalePrincipal => {
            BankAccountActivityLiveOutcome::StalePrincipal
        }
        WorthQueryApplicationLiveOutcome::StaleScope => BankAccountActivityLiveOutcome::StaleScope,
        WorthQueryApplicationLiveOutcome::ProjectionDenied(kind) => {
            BankAccountActivityLiveOutcome::ProjectionDenied(
                BankApplicationLiveProjectionDenial::from_query(kind),
            )
        }
        WorthQueryApplicationLiveOutcome::CauseDenied(kind) => {
            BankAccountActivityLiveOutcome::CauseDenied(BankApplicationLiveCauseDenial::from_query(
                kind,
            ))
        }
        WorthQueryApplicationLiveOutcome::Cancelled => BankAccountActivityLiveOutcome::Cancelled,
        WorthQueryApplicationLiveOutcome::DeadlineExceeded => {
            BankAccountActivityLiveOutcome::DeadlineExceeded
        }
        WorthQueryApplicationLiveOutcome::Closed => BankAccountActivityLiveOutcome::Closed,
        WorthQueryApplicationLiveOutcome::Unavailable => {
            BankAccountActivityLiveOutcome::Unavailable
        }
    }
}

pub(super) fn publish_close(
    outcome: worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome,
) -> BankApplicationLiveCloseOutcome {
    BankApplicationLiveCloseOutcome::from_query(outcome)
}
