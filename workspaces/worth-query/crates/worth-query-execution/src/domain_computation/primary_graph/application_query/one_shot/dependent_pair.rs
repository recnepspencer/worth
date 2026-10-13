//! Scoped admission and execution of one dependent pair branch.

use super::super::{
    derived_view::WorthQueryManagedDerivedViewDenial as Denial,
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationOneShotResult,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryBatchAdmission,
};
use super::WorthQueryApplicationBatchResult;
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_execution::{ExecutionRequest, ExecutionResourceLease, MapKernelContext};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;
use worth_relational::facade::identity::EntityId;

/// Query-owned calling-thread context for exactly one read in a dependent pair.
/// Resolve a local scope, admit its plan with the issued phase, then call
/// `execute` while those borrows remain live. Query retains the genuine result;
/// a callback cannot substitute a result or read a second branch in this stage.
/// The context has no public constructor and grants no plan admission authority.
pub struct WorthQueryDependentPairReadContext<'stage, 'request, 'lease, 'map, 'kernel, S, Q, R> {
    pub(in crate::domain_computation::primary_graph::application_query) application:
        &'stage WorthQueryPrimaryGraphApplicationRuntime<S>,
    pub(in crate::domain_computation::primary_graph::application_query) request:
        &'stage ExecutionRequest<'request, 'lease>,
    pub(in crate::domain_computation::primary_graph::application_query) product:
        &'stage WorthQueryProductBranchLease,
    pub(in crate::domain_computation::primary_graph::application_query) query:
        &'stage WorthQueryInstalledApplicationQueryIdentity,
    pub(in crate::domain_computation::primary_graph::application_query) root: EntityId,
    pub(in crate::domain_computation::primary_graph::application_query) scope_root:
        Option<EntityId>,
    pub(in crate::domain_computation::primary_graph::application_query) batch:
        &'stage WorthQueryApplicationQueryBatchAdmission,
    pub(in crate::domain_computation::primary_graph::application_query) lease:
        Option<&'stage ExecutionResourceLease<'lease>>,
    pub(in crate::domain_computation::primary_graph::application_query) context:
        &'stage mut MapKernelContext<'map, 'kernel>,
    pub(in crate::domain_computation::primary_graph::application_query) request_scope:
        Option<WorthQueryRequestScope>,
    pub(in crate::domain_computation::primary_graph::application_query) attempted: bool,
    pub(in crate::domain_computation::primary_graph::application_query) result:
        Option<WorthQueryApplicationBatchResult<Q, R>>,
}

impl<S: ApplicationSchema, Q, R: WorthQueryApplicationProjection<S, Q>>
    WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, Q, R>
{
    /// Execute this branch's admitted plan under the owner's request and loan.
    /// Returns a borrow for ordinary receipt inspection without releasing rows.
    /// Local scope/principal borrows end after execution. Repeated execution,
    /// a different query, basis, first root or second request is refused.
    pub fn execute<P, A, I, C>(
        &mut self,
        plan: WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    ) -> Result<&WorthQueryApplicationOneShotResult<Q, R>, Denial> {
        if self.attempted {
            // A callback cannot suppress the refusal and publish its earlier read.
            self.result = None;
            return Err(Denial::IncompleteDependencies);
        }
        self.attempted = true;
        if plan.query_identity() != self.query
            || self
                .scope_root
                .is_some_and(|root| plan.scope.entity_id() != root)
        {
            return Err(Denial::ForeignQuery);
        }
        if plan.basis.identity().descriptor() != self.product.relational_basis().descriptor()
            || plan.basis.identity().selection() != &super::super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity::Product(
                crate::basis::WorthQueryProductBranchReadIdentity::from_observation(self.product.observation()))
            || self.request_scope.as_ref().is_some_and(|scope| !scope.same_request(plan.controls.request_scope()))
        {
            return Err(Denial::IncompleteDependencies);
        }
        self.request_scope = Some(plan.controls.request_scope().clone());
        self.result = Some(read::execute(
            self.application,
            self.request,
            plan,
            self.batch,
            self.root,
            self.lease,
            self.context,
        )?);
        Ok(self.result.as_ref().unwrap().result())
    }
}

mod read;
