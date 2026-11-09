//! Read proposed sources and retain actual writers before final group validation.
use super::*;
impl<'a, 'run, 'phase, S: ApplicationSchema> DeclaredProducerAssembly<'a, 'run, 'phase, S> {
    pub(super) fn prepare(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        if let Some(cause) = self.demands.take_failure() {
            return Err(CoupledSourceReadStop::capture(cause));
        }
        let graph = &self.application.runtime.primary_provider.graph;
        let before_pin = graph.with_runtime_mut(execution, |runtime| {
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime, self.before.relational_basis(),
            ).map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ForeignSource, format!("coupled before basis: {cause:?}")))?;
            let indexes = graph.layout.equality_index_ids().collect::<Vec<_>>();
            let pin = runtime.read_truth().pin_read_capability(&snapshot, &indexes, execution);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            pin.map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ForeignSource, format!("coupled before pin: {cause:?}")))
        })?;
        for demand in &self.demands.values {
            self.prepare_one(execution, &before_pin, demand, candidate)?;
        }
        if let Some(tree) = &self.demands.synchronous {
            tree.assemble(self, execution, &before_pin, candidate)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn prepare_one(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before_pin: &worth_relational::facade::runtime::RelationalPinnedRead,
        demand: &crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<super::synchronous_outputs::AppliedOutputParent>, CoupledSourceReadStop>
    {
        let principal = self.principal.ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "coupled producer authentication",
            )
        })?;
        let registry = &self.application.installed_producers;
        let source_entry = registry.declared_coupled_source(demand)?;
        let source = source_entry.executor.read_coupled_source(
            execution,
            self.application,
            principal,
            self.request,
            self.before,
            before_pin,
            demand,
            candidate,
        )?;
        let source = match source {
            super::super::execution::CoupledSourceRead::Retiring => return Ok(None),
            super::super::execution::CoupledSourceRead::Prepared(index) => {
                let writers = self.writers.borrow();
                let writer = writers.get(index).ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSettlement,
                        demand.family,
                    )
                })?;
                if writer.producer.declaration.output_family_type != demand.family_type
                    || writer.producer.declaration.source_type != demand.source_binding_type
                {
                    return Err(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSettlement,
                        demand.family,
                    )
                    .into());
                }
                return Ok(Some(
                    super::super::synchronous_outputs::AppliedOutputParent::new(
                        candidate.pin_source(execution)?,
                        writer.meaning.source_scope,
                    ),
                ));
            }
            super::super::execution::CoupledSourceRead::Verified(parent) => {
                return self
                    .retain_verified_parent(execution, parent)
                    .map(Some)
                    .map_err(Into::into);
            }
            super::super::execution::CoupledSourceRead::Source(source) => source,
        };
        let lifecycle = source.lifecycle;
        let producer = registry.declared_coupled_producer(demand, source.profile, lifecycle)?;
        let pin = candidate.pin_source(execution)?;
        let prepared = producer.executor.prepare_coupled_writer(
            self.phase,
            self.coordination,
            self.application,
            principal,
            self.request,
            self.before,
            before_pin,
            &pin,
            source,
            producer.edition,
        )?;
        let PreparedCoupledWriter {
            writer,
            publication,
            batch,
            content,
        } = prepared;
        // Keep the session even when Native requests an ordinary symbol retry;
        // the next actual assembly round closes it before rerunning the handler.
        let meaning = publication
            .producing_meaning()
            .expect("installed producer participant");
        let source_scope = meaning.source_scope;
        self.writers.borrow_mut().push(CoupledWriter {
            producer: std::sync::Arc::clone(producer),
            stage: Some(CoupledWriterStage::Inspected(writer)),
            meaning: meaning.clone(),
        });
        match self
            .application
            .runtime
            .primary_provider
            .application_attempt_operation_control
            .take_coupled_participant_corruption()
        {
            fault @ (1..=3) => candidate.append_corrupt_participant_for_test(
                execution,
                publication,
                &content,
                vec![batch],
                fault,
            )?,
            _ => candidate.append_publication(execution, publication, &content, vec![batch])?,
        }
        let after_own = candidate.pin_source(execution)?;
        let writers = self.writers.borrow();
        let Some(CoupledWriterStage::Inspected(writer)) =
            writers.last().and_then(|writer| writer.stage.as_ref())
        else {
            unreachable!("actual appended writer")
        };
        self.application
            .runtime
            .primary_provider
            .retain_coupled_after_own(writer.session(), after_own.clone());
        Ok(Some(super::synchronous_outputs::AppliedOutputParent::new(
            after_own,
            source_scope,
        )))
    }
}
