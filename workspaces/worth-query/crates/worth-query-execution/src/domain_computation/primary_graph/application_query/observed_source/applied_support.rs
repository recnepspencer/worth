//! Actual discovery dependencies remain tied to the candidate that read them.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact as Fact,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_relational::facade::{
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};

pub(in crate::domain_computation::primary_graph) struct AppliedQuerySupport {
    query: String,
    pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    facts: Vec<Fact>,
}
impl<Q, V> AppliedQuerySource<Q, V> {
    pub(in crate::domain_computation::primary_graph) fn retain_candidate_support<
        S: worth_query_installation::facade::ApplicationSchema,
    >(
        &self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        query: &str,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<AppliedQuerySupport, WorthQuerySourceExpectationDenial> {
        let foreign = || {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                query,
            )
        };
        let graph = application.runtime.primary_graph().ok_or_else(foreign)?;
        if self.runtime_authority != application.runtime.authority_identity().as_u64()
            || self.candidate.qualification() != pin.candidate_identity().qualification()
        {
            return Err(foreign());
        }
        let facts = fact_conversion::applied_facts(
            self.meaning.footprint(),
            &graph.layout,
            query,
            fact_conversion::SourceFactExpansionControl::in_request(*execution, Some(request)),
        )?;
        Ok(AppliedQuerySupport {
            query: query.into(),
            pin: pin.clone(),
            facts,
        })
    }
}
impl AppliedQuerySupport {
    pub(in crate::domain_computation::primary_graph) fn query(&self) -> &str {
        &self.query
    }
    pub(in crate::domain_computation::primary_graph) fn changed_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        complete: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<&Fact>, WorthQueryApplicationAttemptDenial> {
        for fact in &self.facts {
            if !fact
                .support_equal_after_own_effect(execution, runtime, before, &self.pin, complete)?
            {
                return Ok(Some(fact));
            }
        }
        Ok(None)
    }
}
