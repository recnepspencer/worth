use std::marker::PhantomData;

use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

mod affinity;
mod authority;
mod denial;
mod outcome;
mod readmission;
#[cfg(test)]
mod tests;

pub use authority::WorthQueryApplicationQueryContinuation;
pub use denial::{
    WorthQueryApplicationContinuationDenial, WorthQueryApplicationContinuationDenialKind,
};

use super::{
    authorized_read::{
        execute_authorized_read, refresh_governed_authorization,
        WorthQueryAuthorizedApplicationReadDenial,
    },
    execution_validation::{
        validate_execution_lifetimes, validate_execution_plan, validate_live_basis,
        WorthQueryApplicationQueryExecutionValidationDenial,
    },
    read_execution::{read_continuation_page, WorthQueryApplicationReadExecutionDenialKind},
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationProjection,
    WorthQueryApplicationQueryAccessReceipt,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use outcome::finalize_continuation_page;

pub struct WorthQueryApplicationContinuationPageResult<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
> {
    rows: Vec<QueryResult>,
    continuation: Option<
        WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>,
    >,
    receipt: WorthQueryApplicationQueryAccessReceipt,
    _query: PhantomData<fn() -> Query>,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn execute_application_query_continuation_page<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        mut plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<
        WorthQueryApplicationContinuationPageResult<Schema, Query, Parameters, QueryResult, Scope>,
        WorthQueryApplicationContinuationDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        validate_execution_plan(self, &plan)
            .map_err(|denial| map_validation_denial(denial, plan.query.name()))?;
        if plan.controls.lane() != WorthQueryApplicationQueryLane::Continuation {
            return Err(denial(
                WorthQueryApplicationContinuationDenialKind::ForeignPlan,
                plan.query.name(),
            ));
        }
        validate_execution_lifetimes(self, &plan.controls, plan.principal)
            .map_err(|denial| map_validation_denial(denial, plan.query.name()))?;
        validate_live_basis(plan.basis.is_live())
            .map_err(|denial| map_validation_denial(denial, plan.query.name()))?;
        refresh_governed_authorization(self, &mut plan)
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;
        let expected_generation = plan
            .continuation_state
            .as_ref()
            .map(|state| state.expected_generation);
        let after = plan
            .continuation_state
            .as_ref()
            .map(|state| state.boundary.clone());
        self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationContinuationDenialKind::StaleInstalledQuery,
                plan.query.name(),
            )
        })?;
        let result_buffer = self.result_buffers.reserve(
            plan.graph_read_plan()
                .budget_check()
                .max_inline_result_bytes(),
        );
        let (raw, authorization_work, read_proof) =
            execute_authorized_read(self, &plan, |runtime, graph, plan| {
                read_continuation_page(runtime, graph, plan, after, result_buffer)
            })
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;
        if expected_generation.is_some() && raw.raw.ordered_index_generation != expected_generation
        {
            return Err(denial(
                WorthQueryApplicationContinuationDenialKind::ContinuationGenerationChanged,
                plan.query.name(),
            ));
        }
        finalize_continuation_page(self, plan, raw, authorization_work, read_proof)
    }
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    WorthQueryApplicationContinuationPageResult<Schema, Query, Parameters, QueryResult, Scope>
{
    pub fn rows(&self) -> &[QueryResult] {
        &self.rows
    }

    pub fn continuation(
        &self,
    ) -> Option<
        &WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>,
    > {
        self.continuation.as_ref()
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryAccessReceipt {
        &self.receipt
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<QueryResult>,
        Option<
            WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>,
        >,
        WorthQueryApplicationQueryAccessReceipt,
    ) {
        (self.rows, self.continuation, self.receipt)
    }

    pub fn into_admitted_disclosed(
        self,
    ) -> (
        super::WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult>,
        Option<
            WorthQueryApplicationQueryContinuation<Schema, Query, Parameters, QueryResult, Scope>,
        >,
    ) {
        (
            super::WorthQueryAdmittedDisclosedApplicationResult::new(self.rows, self.receipt),
            self.continuation,
        )
    }
}

fn map_authorized_read_denial(
    denial_value: WorthQueryAuthorizedApplicationReadDenial,
    subject: &str,
) -> WorthQueryApplicationContinuationDenial {
    let (kind, subject) = match denial_value {
        WorthQueryAuthorizedApplicationReadDenial::StalePrincipal => (
            WorthQueryApplicationContinuationDenialKind::StalePrincipal,
            subject.to_string(),
        ),
        WorthQueryAuthorizedApplicationReadDenial::StaleScope
        | WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope => (
            WorthQueryApplicationContinuationDenialKind::StaleScope,
            subject.to_string(),
        ),
        WorthQueryAuthorizedApplicationReadDenial::Authorization(denial) => {
            return WorthQueryApplicationContinuationDenial::from_authorization(denial);
        }
        WorthQueryAuthorizedApplicationReadDenial::Read(read) => {
            let kind = match read.kind() {
                WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable => {
                    WorthQueryApplicationContinuationDenialKind::PredicateIndexUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::PredicateLookupOverflow => {
                    WorthQueryApplicationContinuationDenialKind::PredicateLookupOverflow
                }
                WorthQueryApplicationReadExecutionDenialKind::ResultLimitExceeded => {
                    WorthQueryApplicationContinuationDenialKind::ResultLimitExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch => {
                    WorthQueryApplicationContinuationDenialKind::CardinalityMismatch
                }
                WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable => {
                    WorthQueryApplicationContinuationDenialKind::TraversalUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable => {
                    WorthQueryApplicationContinuationDenialKind::ContinuationIndexUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::ContinuationBoundaryRejected => {
                    WorthQueryApplicationContinuationDenialKind::ContinuationBoundaryRejected
                }
                WorthQueryApplicationReadExecutionDenialKind::ContinuationGenerationChanged => {
                    WorthQueryApplicationContinuationDenialKind::ContinuationGenerationChanged
                }
                WorthQueryApplicationReadExecutionDenialKind::ContinuationPageWidthInvalid => {
                    WorthQueryApplicationContinuationDenialKind::ContinuationPageWidthInvalid
                }
                WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable => {
                    WorthQueryApplicationContinuationDenialKind::ProjectionUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded => {
                    WorthQueryApplicationContinuationDenialKind::ResultBufferLimitExceeded
                }
                WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable
                | WorthQueryApplicationReadExecutionDenialKind::TargetIdentityLookupOverflow
                | WorthQueryApplicationReadExecutionDenialKind::TargetIdentityNotFound => {
                    WorthQueryApplicationContinuationDenialKind::ProjectionUnavailable
                }
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded => {
                    WorthQueryApplicationContinuationDenialKind::WorkLimitExceeded
                }
            };
            (kind, read.subject().to_string())
        }
        WorthQueryAuthorizedApplicationReadDenial::Session => (
            WorthQueryApplicationContinuationDenialKind::ForeignPlan,
            subject.to_string(),
        ),
    };
    denial(kind, subject)
}

fn map_validation_denial(
    denial_value: WorthQueryApplicationQueryExecutionValidationDenial,
    subject: &str,
) -> WorthQueryApplicationContinuationDenial {
    use WorthQueryApplicationQueryExecutionValidationDenial as Validation;
    let kind = match denial_value {
        Validation::ForeignPlan => WorthQueryApplicationContinuationDenialKind::ForeignPlan,
        Validation::StaleInstalledQuery => {
            WorthQueryApplicationContinuationDenialKind::StaleInstalledQuery
        }
        Validation::StalePrincipal => WorthQueryApplicationContinuationDenialKind::StalePrincipal,
        Validation::Cancelled => WorthQueryApplicationContinuationDenialKind::Cancelled,
        Validation::DeadlineExceeded => {
            WorthQueryApplicationContinuationDenialKind::DeadlineExceeded
        }
        Validation::ExpiredBasis => WorthQueryApplicationContinuationDenialKind::ExpiredBasis,
        Validation::BasisUnavailable => {
            WorthQueryApplicationContinuationDenialKind::BasisUnavailable
        }
    };
    denial(kind, subject)
}

fn denial(
    kind: WorthQueryApplicationContinuationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationContinuationDenial {
    WorthQueryApplicationContinuationDenial::new(kind, subject)
}
