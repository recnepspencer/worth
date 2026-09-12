use bank_domain::{
    model::BankPrincipalId,
    queries::{
        EstateEmergencyAccessActivity, EstateEmergencyAccessActivityQuery,
        EstateEmergencyAccessActivityQueryBinding, EstateEmergencyAccessActivityQueryParameters,
    },
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, Principal,
        ViewEstateEmergencyProtectionCapability, ViewRestrictedEstateOperation,
    },
};
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet,
    primary_graph::{
        WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryContinuation,
        WorthQueryApplicationQueryResumeControls, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
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
        let application = self.runtime.application_runtime();
        let selected = application
            .on_branch(application.current_world())
            .select()
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let query_binding = application
            .installed_schema()
            .installed_query_binding::<EstateEmergencyAccessActivityQueryBinding>()
            .map_err(BankApplicationQueryDenial::from_installation)?;

        let query = query_binding.query();
        let capability = application
            .installed_schema()
            .capability(
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
            )
            .map_err(BankApplicationQueryDenial::from_capability_installation)?;
        let capability_access = selected
            .admit_approved_elevation_access(
                self.approved.query(),
                self.principal.query(),
                &capability,
                self.request.capability_request(),
                self.controls.request(),
            )
            .map_err(BankApplicationQueryDenial::from_capability_admission)?;
        let scope = selected
            .resolve_entity(
                EstateCaseIdentityField::reference(),
                self.request.estate(),
                self.controls.request(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(BankApplicationQueryDenial::from_scope_resolution)?;
        let access = WorthQueryApplicationQueryAccessContext::<
            BankSchema,
            Principal,
            BankPrincipalId,
            EstateCase,
        >::new(self.principal.query(), &scope);
        let plan = selected
            .admit_governed_application_query_continuation(
                query,
                &access,
                capability_access,
                ApplicationQueryParameterSet::<EstateEmergencyAccessActivityQuery>::new(),
                WorthQueryProductQueryControls::new(
                    self.controls.maximum_result_count(),
                    self.controls.maximum_work(),
                    self.controls.request(),
                ),
            )
            .map_err(BankApplicationQueryDenial::from_admission)?;
        let page = application
            .execute_application_query_continuation_page(plan)
            .map_err(BankApplicationQueryDenial::from_continuation_execution)?;
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
        let application = self.runtime.application_runtime();
        let query_binding = application
            .installed_schema()
            .installed_query_binding::<EstateEmergencyAccessActivityQueryBinding>()
            .map_err(BankApplicationQueryDenial::from_installation)?;

        let query = query_binding.query();
        let capability = application
            .installed_schema()
            .capability(
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
            )
            .map_err(BankApplicationQueryDenial::from_capability_installation)?;
        let current = application
            .on_branch(application.current_world())
            .select()
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let capability_access = current
            .admit_approved_elevation_access(
                self.approved.query(),
                self.principal.query(),
                &capability,
                self.request.capability_request(),
                controls.request_scope(),
            )
            .map_err(BankApplicationQueryDenial::from_capability_admission)?;
        let scope = current
            .resolve_entity(
                EstateCaseIdentityField::reference(),
                self.request.estate(),
                controls.request_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(BankApplicationQueryDenial::from_scope_resolution)?;
        let access = WorthQueryApplicationQueryAccessContext::<
            BankSchema,
            Principal,
            BankPrincipalId,
            EstateCase,
        >::new(self.principal.query(), &scope);
        let plan = application
            .readmit_governed_application_query_continuation(
                query,
                &access,
                capability_access,
                ApplicationQueryParameterSet::<EstateEmergencyAccessActivityQuery>::new(),
                continuation,
                controls,
            )
            .map_err(BankApplicationQueryDenial::from_admission)?;
        after_readmission(BankAdmittedEstateEmergencyAccessActivityContinuation {
            application,
            plan,
        })
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
