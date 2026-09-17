use bank_domain::{
    queries::{
        EstateEmergencyAccessActivity, EstateEmergencyAccessActivityQuery,
        EstateEmergencyAccessActivityQueryParameters,
    },
    schema::{
        BankSchema, EstateCase, ViewEstateEmergencyProtectionCapability,
        ViewRestrictedEstateOperation,
    },
};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    primary_graph::{
        WorthQueryApplicationQueryContinuation, WorthQueryApplicationQueryResumeControls,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
    publication::domain_computation::{
        publish_application_result, WorthQueryPublishedApplicationResult,
    },
};

use super::{admission::BankEstateEmergencyAccessActivityAdmission, bounded::ActivityPlan};
use crate::BankApplicationQueryDenial;

type QueryEstateEmergencyAccessActivityContinuation = WorthQueryApplicationQueryContinuation<
    BankSchema,
    EstateEmergencyAccessActivityQuery,
    EstateEmergencyAccessActivityQueryParameters,
    EstateEmergencyAccessActivity,
    EstateCase,
>;

/// Opaque Bank authority for resuming emergency-access activity.
///
/// ```compile_fail,E0451
/// use bank_server::BankEstateEmergencyAccessActivityContinuation;
///
/// let _ = BankEstateEmergencyAccessActivityContinuation {
///     query: panic!("foreign continuation"),
/// };
/// ```
///
/// The wrapper cannot be coerced to Query's continuation authority:
///
/// ```compile_fail,E0308
/// use bank_domain::queries::{
///     EstateEmergencyAccessActivity, EstateEmergencyAccessActivityQuery,
///     EstateEmergencyAccessActivityQueryParameters,
/// };
/// use bank_domain::schema::{BankSchema, EstateCase};
/// use bank_server::BankEstateEmergencyAccessActivityContinuation;
/// use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryContinuation;
///
/// type RawContinuation = WorthQueryApplicationQueryContinuation<
///     BankSchema,
///     EstateEmergencyAccessActivityQuery,
///     EstateEmergencyAccessActivityQueryParameters,
///     EstateEmergencyAccessActivity,
///     EstateCase,
/// >;
///
/// fn raw_query_continuation(
///     continuation: &BankEstateEmergencyAccessActivityContinuation,
/// ) -> &RawContinuation {
///     continuation
/// }
/// ```
///
/// Nor does it expose the former raw-authority accessor:
///
/// ```compile_fail,E0599
/// use bank_domain::queries::{
///     EstateEmergencyAccessActivity, EstateEmergencyAccessActivityQuery,
///     EstateEmergencyAccessActivityQueryParameters,
/// };
/// use bank_domain::schema::{BankSchema, EstateCase};
/// use bank_server::BankEstateEmergencyAccessActivityContinuation;
/// use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryContinuation;
///
/// type RawContinuation = WorthQueryApplicationQueryContinuation<
///     BankSchema,
///     EstateEmergencyAccessActivityQuery,
///     EstateEmergencyAccessActivityQueryParameters,
///     EstateEmergencyAccessActivity,
///     EstateCase,
/// >;
///
/// fn raw_query_continuation(
///     continuation: &BankEstateEmergencyAccessActivityContinuation,
/// ) -> &RawContinuation {
///     continuation.query()
/// }
/// ```
pub struct BankEstateEmergencyAccessActivityContinuation {
    query: QueryEstateEmergencyAccessActivityContinuation,
}

pub struct BankEstateEmergencyAccessActivityPageResult {
    published: WorthQueryPublishedApplicationResult<
        EstateEmergencyAccessActivityQuery,
        EstateEmergencyAccessActivity,
    >,
    continuation: Option<BankEstateEmergencyAccessActivityContinuation>,
}

pub struct BankAdmittedEstateEmergencyAccessActivityContinuation<'a> {
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    plan: ActivityPlan<'a>,
}

impl std::fmt::Debug for BankEstateEmergencyAccessActivityPageResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BankEstateEmergencyAccessActivityPageResult")
            .field("row_count", &self.published.rows().len())
            .field("has_continuation", &self.continuation.is_some())
            .field("receipt", self.published.receipt())
            .finish()
    }
}

impl BankEstateEmergencyAccessActivityPageResult {
    pub fn rows(&self) -> &[EstateEmergencyAccessActivity] {
        self.published.rows()
    }

    pub fn receipt(
        &self,
    ) -> &worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt{
        self.published.receipt()
    }

    pub const fn continuation(&self) -> Option<&BankEstateEmergencyAccessActivityContinuation> {
        self.continuation.as_ref()
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPublishedApplicationResult<
            EstateEmergencyAccessActivityQuery,
            EstateEmergencyAccessActivity,
        >,
        Option<BankEstateEmergencyAccessActivityContinuation>,
    ) {
        (self.published, self.continuation)
    }
}

impl BankEstateEmergencyAccessActivityContinuation {
    const fn from_query(query: QueryEstateEmergencyAccessActivityContinuation) -> Self {
        Self { query }
    }
}

impl BankAdmittedEstateEmergencyAccessActivityContinuation<'_> {
    pub fn execute(
        self,
    ) -> Result<BankEstateEmergencyAccessActivityPageResult, BankApplicationQueryDenial> {
        let page = self
            .application
            .execute_application_query_continuation_page(self.plan)
            .map_err(BankApplicationQueryDenial::from_continuation_execution)?;
        Ok(publish_page(page))
    }
}

impl BankEstateEmergencyAccessActivityAdmission<'_, '_, '_, '_> {
    pub(crate) fn page(
        self,
    ) -> Result<BankEstateEmergencyAccessActivityPageResult, BankApplicationQueryDenial> {
        let capability_input = self.request.capability_request();
        let page = self
            .runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .query(self.request)
            .page_approved(
                self.approved.query(),
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .map_err(BankApplicationQueryDenial::from_request_query)?;
        Ok(publish_page(page))
    }

    pub(crate) fn resume(
        self,
        continuation: BankEstateEmergencyAccessActivityContinuation,
        controls: WorthQueryApplicationQueryResumeControls<'_>,
    ) -> Result<BankEstateEmergencyAccessActivityPageResult, BankApplicationQueryDenial> {
        self.readmit_resume(continuation, controls, |admitted| admitted.execute())
    }

    pub(crate) fn readmit_resume<Output>(
        self,
        continuation: BankEstateEmergencyAccessActivityContinuation,
        controls: WorthQueryApplicationQueryResumeControls<'_>,
        after_readmission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccessActivityContinuation<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        let BankEstateEmergencyAccessActivityContinuation {
            query: continuation,
        } = continuation;
        let capability_input = self.request.capability_request();
        self.runtime
            .application_runtime()
            .request(self.principal.external(), controls.request_scope())
            .query(self.request)
            .readmit_resume_approved(
                self.approved.query(),
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
                continuation,
                controls.maximum_page_width(),
                controls.maximum_work(),
                |application, plan| {
                    after_readmission(BankAdmittedEstateEmergencyAccessActivityContinuation {
                        application,
                        plan,
                    })
                },
            )
            .map_err(BankApplicationQueryDenial::from_request_query)?
    }
}

fn publish_page(
    page: worth_query_host::facade::primary_graph::WorthQueryApplicationContinuationPageResult<
        BankSchema,
        EstateEmergencyAccessActivityQuery,
        EstateEmergencyAccessActivityQueryParameters,
        EstateEmergencyAccessActivity,
        EstateCase,
    >,
) -> BankEstateEmergencyAccessActivityPageResult {
    let (admitted, continuation) = page.into_admitted_disclosed();
    BankEstateEmergencyAccessActivityPageResult {
        published: publish_application_result(admitted),
        continuation: continuation.map(BankEstateEmergencyAccessActivityContinuation::from_query),
    }
}
