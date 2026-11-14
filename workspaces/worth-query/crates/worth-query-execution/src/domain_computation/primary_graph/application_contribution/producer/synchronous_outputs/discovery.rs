//! Discovery reads actual appended output through the ordinary C-query owner.
use super::*;
use crate::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryObservationStop;
use crate::domain_computation::primary_graph::{
    application_query::{ApplicationQueryNativeAccess, SourceQueryReaderAccess},
    WorthQueryApplicationProjection,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
type Row<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
impl<S: ApplicationSchema> SynchronousOutputAssembly<'_, '_, '_, '_, S> {
    pub(super) fn discover<B>(
        &self,
        parent: &AppliedOutputParent,
        intent: B::Input,
    ) -> Result<Row<S, B>, CoupledSourceReadStop>
    where
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        Row<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        self.discover_as_at::<B, Row<S, B>>(&parent.pin, parent.source(), intent)
    }
    pub(super) fn discover_authored<B>(
        &self,
        source: &AppliedAuthoredDiscovery,
        intent: B::Input,
    ) -> Result<Row<S, B>, CoupledSourceReadStop>
    where
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        Row<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        self.discover_as_at::<B, Row<S, B>>(&source.pin, source.source, intent)
    }
    pub(super) fn discover_as_at<B, V>(
        &self,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        subject: worth_relational::facade::identity::EntityId,
        intent: B::Input,
    ) -> Result<V, CoupledSourceReadStop>
    where
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        V: WorthQueryApplicationProjection<S, B::Query>,
    {
        let runtime = self.owner.application;
        let external = self.owner.principal.ok_or_else(|| {
            super::super::WorthQueryOutputDemandDenial::new(
                super::super::WorthQueryOutputDemandDenialKind::ForeignSource,
                format!(
                    "{}: current principal unavailable for discovery",
                    B::IDENTITY
                ),
            )
        })?;
        let selected = runtime
            .on_branch(self.owner.before.product_branch())
            .select_with_request(self.execution)
            .map_err(|cause| {
                super::super::WorthQueryOutputDemandDenial::product_selection(cause, B::IDENTITY)
            })?;
        let shared = selected
            .prepare_shared_query_basis()
            .map_err(|(_, cause)| {
                let stop = SourceQueryObservationStop::Read(
                    crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedOneShotStop::Admission(cause),
                );
                super::super::execution::denial::selected_source_read_denied(B::IDENTITY, stop)
            })?;
        let reader = SourceQueryReaderAccess::new(
            runtime,
            &shared,
            None,
            None,
            external,
            self.owner.request,
        )?;
        reader
            .with_applied_plan::<B, _, SourceQueryObservationStop>(
                self.execution,
                intent,
                self.before,
                pin,
                &mut ApplicationQueryNativeAccess::acquire(),
                |plan, _| {
                    let source = runtime
                        .project_applied_source_plan_as::<_, _, _, V, _, _, _>(
                            self.execution,
                            plan,
                            pin,
                        )
                        .map_err(|cause| cause.into_source_query_stop(B::IDENTITY))?;
                    let support = source
                        .retain_candidate_support(
                            runtime,
                            self.execution,
                            self.owner.request,
                            B::IDENTITY,
                            pin,
                        )
                        .map_err(SourceQueryObservationStop::SourceFacts)?;
                    self.owner
                        .retain_discovery_support(self.execution, subject, support)
                        .map_err(SourceQueryObservationStop::CurrentControl)?;
                    Ok(source.value)
                },
            )
            .map_err(|cause| {
                super::super::execution::denial::selected_source_read_denied(B::IDENTITY, cause)
                    .into()
            })
    }
}
