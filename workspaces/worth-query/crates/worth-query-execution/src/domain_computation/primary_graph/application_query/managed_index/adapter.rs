//! Installed typed callbacks; root erasure stays private to their actual adapter.
use super::invocation::{decoded_provider, provider_result};
use super::*;
use crate::domain_computation::primary_graph::access_strategy::coverage::{
    CandidateSelection, CoverageFrame, CoverageVerifier, RecordedFrontier,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity as Session;
use std::{any::Any, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_proof::ManagedCoverageDisposition as Disposition;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndex;

mod execution;
mod history;
pub(super) use execution::ErasedIndexExecutor;

pub(super) trait InstalledIndexProvider<S, I>: Send + Sync
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    fn installed(&self) -> &WorthQueryInstalledManagedIndex<S, I>;
    fn coverage_authority(&self) -> &worth_proof::ManagedIndexCoverageAuthority;
    fn realized_authority(&self) -> worth_proof::RealizedFootprintAuthority;
    fn verify_realized(
        &self,
        captured: &super::realized_footprint::CheckedManagedFootprintCapture<'_>,
    ) -> Result<worth_proof::VerifiedRealizedFootprint, WorthQueryManagedIndexStop>;
    fn build(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        build_session: Session,
        image: source_rows::SourceImageView<'_>,
    ) -> Result<Arc<dyn Any + Send + Sync>, WorthQueryManagedIndexWorkDenial>;
    fn candidates<'frame, 'loan>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        frame: CoverageFrame<'frame, 'loan>,
    ) -> Result<CandidateSelection<'frame, 'loan>, WorthQueryManagedIndexWorkDenial>;
    fn historical_candidate_relevant(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        before: source_rows::SourceImageView<'_>,
        after: source_rows::SourceImageView<'_>,
        probe: &worth_foundational::facade::AspectValue,
        material: &worth_relational::facade::history::RelationalAuthoredManagedSourceMaterial,
    ) -> Result<bool, WorthQueryManagedIndexWorkDenial>;
    fn update(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        before: source_rows::SourceImageView<'_>,
        after: source_rows::SourceImageView<'_>,
        changes: &[WorthQueryManagedIndexSourceChange],
        nodes: &(dyn Any + Send + Sync),
    ) -> Result<Arc<dyn Any + Send + Sync>, WorthQueryManagedIndexWorkDenial>;
}

pub(super) struct TypedIndexProvider<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
> {
    pub(super) installed: WorthQueryInstalledManagedIndex<S, I>,
    pub(super) provider: Arc<P>,
    pub(super) predicate: Option<Arc<dyn super::predicate::PredicateFactory>>,
    pub(super) refinement: Option<refinement::RefinementCallbacks<S, I, P>>,
    pub(super) coverage: CoverageVerifier,
    pub(super) bindings:
        worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
}

impl<S, I, P> InstalledIndexProvider<S, I> for TypedIndexProvider<S, I, P>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    fn installed(&self) -> &WorthQueryInstalledManagedIndex<S, I> {
        &self.installed
    }
    fn coverage_authority(&self) -> &worth_proof::ManagedIndexCoverageAuthority {
        self.coverage.authority()
    }
    fn realized_authority(&self) -> worth_proof::RealizedFootprintAuthority {
        self.coverage.realized_authority()
    }
    fn verify_realized(
        &self,
        captured: &super::realized_footprint::CheckedManagedFootprintCapture<'_>,
    ) -> Result<worth_proof::VerifiedRealizedFootprint, WorthQueryManagedIndexStop> {
        self.coverage.verify_realized(captured)
    }
    fn build(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        build_session: Session,
        image: source_rows::SourceImageView<'_>,
    ) -> Result<Arc<dyn Any + Send + Sync>, WorthQueryManagedIndexWorkDenial> {
        let (nodes, structure) =
            build::build::<S, I, P>(execution, self.provider.as_ref(), build_session, image)?;
        node_backing::retain::<S, I, P>(execution, nodes, structure).map_err(Into::into)
    }
    fn candidates<'frame, 'loan>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        frame: CoverageFrame<'frame, 'loan>,
    ) -> Result<CandidateSelection<'frame, 'loan>, WorthQueryManagedIndexWorkDenial> {
        let image = frame.image;
        let probe = frame.probe;
        let payload = frame.nodes;
        decoded_provider(execution, image.bindings, |control, session| {
            let source_control = image
                .source_budget
                .map(|budget| control.under_source(budget));
            let control = source_control.as_ref().unwrap_or(control);
            let probe = session
                .decode_managed_probe::<I::ProbeBinding>(probe)
                .ok_or_else(|| {
                    control.refuse(WorthQueryManagedIndexStop::ValueDecode(
                        session
                            .denial()
                            .expect("failed managed probe latches its refusal"),
                    ))
                })?;
            let nodes = payload
                .as_ref()
                .downcast_ref::<node_backing::ProviderNodeBacking<S, I, P>>()
                .ok_or_else(|| {
                    WorthQueryManagedIndexWorkDenial::Controlled(
                        WorthQueryManagedIndexStop::ForeignNode,
                    )
                })?;
            let source =
                WorthQueryManagedIndexSourceRows::<S, I>::new(image, session, control, *execution);
            use worth_query_declaration::facade::application_query::ApplicationManagedIndexMembership as Membership;
            let complete = I::MEMBERSHIP != Membership::Candidates;
            if complete {
                provider_result(
                    execution,
                    control,
                    self.provider.validate_probe(&probe, control),
                )?;
            }
            let refinement = if frame.posture == worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined {
                if !complete { return Err(control.refuse(WorthQueryManagedIndexStop::MembershipUnavailable).into()); }
                let callbacks = self.refinement.as_ref().ok_or_else(|| control.refuse(WorthQueryManagedIndexStop::RefinementUnavailable))?;
                provider_result(execution, control, (callbacks.validate)(&self.provider, &probe, control))?;
                Some(callbacks)
            } else { None };
            let mut candidates =
                WorthQueryManagedIndexCandidateWriter::allocate(execution, &source, control)?;
            let mut frontier = RecordedFrontier::new();
            if complete {
                let tree = nodes.structure.as_ref().ok_or_else(|| {
                    control.refuse(WorthQueryManagedIndexStop::MembershipUnavailable)
                })?;
                let context = source_structure::SummaryContext::new(tree, &source, control);
                crate::domain_computation::primary_graph::access_strategy::coverage::visit(
                    tree,
                    nodes.nodes.root().map(|id| id.ordinal),
                    control,
                    &mut |ordinal, slot| {
                        if I::MEMBERSHIP == Membership::CheckedSummarySourceTree {
                            let node = nodes.nodes.node_at_ordinal(ordinal, control)?.ok_or_else(
                                || control.refuse(WorthQueryManagedIndexStop::IncompleteTraversal),
                            )?;
                            let summary = provider_result(
                                execution,
                                control,
                                self.provider.node_summary(*node, control),
                            )?;
                            let sources =
                                WorthQueryManagedIndexSummarySources::new(&context, ordinal);
                            if !provider_result(
                                execution,
                                control,
                                self.provider
                                    .summary_may_match(summary, &sources, &probe, control),
                            )? {
                                frontier.record(
                                    execution,
                                    ordinal,
                                    Disposition::Pruned,
                                    control,
                                )?;
                                return Ok(false);
                            }
                        }
                        if let Some(slot) = slot {
                            let row = source.get(slot)?;
                            let matches = provider_result(
                                execution,
                                control,
                                self.provider.matches_source(row, &probe, control),
                            )?;
                            if matches {
                                candidates.emit(slot)?;
                            }
                            frontier.record(
                                execution,
                                ordinal,
                                if matches {
                                    Disposition::Matched
                                } else {
                                    Disposition::Excluded
                                },
                                control,
                            )?;
                        } else {
                            frontier.record(execution, ordinal, Disposition::Branch, control)?;
                        }
                        Ok(true)
                    },
                )?;
            } else {
                provider_result(
                    execution,
                    control,
                    self.provider
                        .query(&source, &nodes.nodes, &probe, &mut candidates, control),
                )?;
            }
            source.check()?;
            let slots = candidates.finish(execution)?;
            if complete {
                let selected = self.coverage.complete(
                    execution,
                    frame,
                    nodes.structure.as_ref().unwrap(),
                    frontier,
                    I::MEMBERSHIP == Membership::CheckedSummarySourceTree,
                    slots,
                    control,
                )?;
                let selected = match refinement {
                    Some(callbacks) => selected.refine(execution, control, |slot| {
                        provider_result(
                            execution,
                            control,
                            (callbacks.evaluate)(
                                &self.provider,
                                source.get(slot)?,
                                &probe,
                                control,
                            ),
                        )
                    })?,
                    None => selected,
                };
                source.check()?;
                Ok(selected)
            } else {
                Ok(CandidateSelection::conservative(frame, slots))
            }
        })
    }
    fn historical_candidate_relevant(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        before: source_rows::SourceImageView<'_>,
        after: source_rows::SourceImageView<'_>,
        probe: &worth_foundational::facade::AspectValue,
        material: &worth_relational::facade::history::RelationalAuthoredManagedSourceMaterial,
    ) -> Result<bool, WorthQueryManagedIndexWorkDenial> {
        history::relevant(self, execution, before, after, probe, material)
    }
    fn update(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        before: source_rows::SourceImageView<'_>,
        after: source_rows::SourceImageView<'_>,
        changes: &[WorthQueryManagedIndexSourceChange],
        nodes: &(dyn Any + Send + Sync),
    ) -> Result<Arc<dyn Any + Send + Sync>, WorthQueryManagedIndexWorkDenial> {
        decoded_provider(execution, after.bindings, |control, decode| {
            let source_control = after
                .source_budget
                .map(|budget| control.under_source(budget));
            let control = source_control.as_ref().unwrap_or(control);
            let retained = nodes
                .downcast_ref::<node_backing::ProviderNodeBacking<S, I, P>>()
                .ok_or_else(|| control.refuse(WorthQueryManagedIndexStop::ForeignNode))?;
            let before =
                WorthQueryManagedIndexSourceRows::<S, I>::new(before, decode, control, *execution);
            let after =
                WorthQueryManagedIndexSourceRows::<S, I>::new(after, decode, control, *execution);
            let proposed_capacity = provider_result(
                execution,
                control,
                self.provider.node_capacity(after.len(), control),
            )?;
            // Old live nodes can coexist with the provider's proposed nodes
            // until the edit retires them. This bounds live nodes, not sparse
            // ordinals; pages are shared and storage is admitted on each write.
            let capacity = retained
                .nodes
                .len()
                .checked_add(proposed_capacity)
                .ok_or_else(|| control.refuse(WorthQueryManagedIndexStop::CapacityOverflow))?;
            let mut writer = retained
                .nodes
                .edit(*execution, capacity, after.len(), control)?;
            if let Some(previous) = retained.structure.as_ref() {
                writer.track_changes(previous);
            }
            let root = provider_result(
                execution,
                control,
                self.provider.update(
                    &before,
                    &after,
                    changes,
                    retained.nodes.root(),
                    &mut writer,
                    control,
                ),
            )?;
            before.check()?;
            after.check()?;
            let mut nodes = writer.seal(execution, root)?;
            let structure = match (I::STRUCTURE, retained.structure.as_ref()) {
                (worth_query_declaration::facade::application_query::ApplicationManagedIndexSourceStructure::Opaque, None) => None,
                (worth_query_declaration::facade::application_query::ApplicationManagedIndexSourceStructure::BinarySourceTree, Some(previous)) => super::source_structure::qualify(execution, self.provider.as_ref(), &after, &mut nodes,
                    super::source_structure::Step::Warm { before: previous, changes }, control)?,
                _ => return Err(control.refuse(WorthQueryManagedIndexStop::SourceStructureIncomplete).into()),
            };
            node_backing::retain::<S, I, P>(execution, nodes, structure).map_err(Into::into)
        })
    }
}
