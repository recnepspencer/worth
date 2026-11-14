//! Read proposed sources and retain actual writers before final group validation.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand;
enum PendingOutputDemand<'a> {
    Prefix(&'a DeclaredCoupledOutputDemand),
    Stage(DeclaredCoupledOutputDemand),
}
impl PendingOutputDemand<'_> {
    fn demand(&self) -> &DeclaredCoupledOutputDemand {
        match self {
            Self::Prefix(demand) => demand,
            Self::Stage(demand) => demand,
        }
    }
}
impl<'a, 'run, 'phase, S: ApplicationSchema> CoupledApplicationAssembly<'a, 'run, 'phase, S> {
    pub(super) fn prepare(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        if let Some(cause) = self.demands.take_failure() {
            return Err(CoupledSourceReadStop::capture(cause));
        }
        let graph = &self.application.runtime.primary_provider.graph;
        let before_pin = graph.with_runtime(execution, |runtime| {
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime, self.before.relational_basis(),
            ).map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ForeignSource, format!("coupled before basis: {cause:?}")))?;
            let indexes = graph.layout.equality_index_ids().collect::<Vec<_>>();
            let pin = runtime.read_truth().pin_read_capability(&snapshot, &indexes, execution);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            pin.map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ForeignSource, format!("coupled before pin: {cause:?}")))
        })?;
        self.require_authored_basis()?;
        if self.all_stage_captures().is_empty() {
            for demand in &self.demands.values {
                self.prepare_one(execution, &before_pin, demand, candidate)?;
            }
            let roots = self.demands.synchronous.iter().collect::<Vec<_>>();
            return synchronous_outputs::CapturedSynchronousOutputs::assemble(
                &roots,
                self,
                execution,
                &before_pin,
                candidate,
            );
        }
        if let Some(prefix) = self.all_stage_captures().first() {
            let pin = candidate.pin_source(execution)?;
            self.application
                .runtime
                .primary_provider
                .retain_authored_prefix_after_own(
                    &self.root_session,
                    prefix.identity,
                    prefix.binding,
                    pin,
                )
                .map_err(CoupledSourceReadStop::capture)?;
        }
        let captured = self
            .stage_captures()
            .iter()
            .filter_map(|stage| stage.required.as_ref())
            .map(|capture| capture(execution))
            .collect::<Vec<_>>();
        let roots = self
            .demands
            .synchronous
            .iter()
            .chain(captured.iter())
            .collect::<Vec<_>>();
        let mut wave = synchronous_outputs::SynchronousOutputWave::new();
        let mut order = synchronous_outputs::CapturedSynchronousOutputs::seed(
            &roots,
            self,
            execution,
            &before_pin,
            candidate,
            &mut wave,
        )?;
        let mut stage_demands = Vec::new();
        let mut pending = self
            .demands
            .values
            .iter()
            .map(PendingOutputDemand::Prefix)
            .collect::<Vec<_>>();
        for (physical, capture) in self.stage_captures().iter().enumerate() {
            self.prepare_pending(
                execution,
                &before_pin,
                candidate,
                &mut pending,
                Some(&capture.producer_barrier),
            )?;
            if let Some(order) = &order {
                let context = synchronous_outputs::SynchronousOutputAssembly {
                    owner: self,
                    execution,
                    before: &before_pin,
                    order,
                };
                wave.drive(&context, candidate, Some(&capture.producer_barrier))
                    .map_err(|cause| cause.0)?;
            }
            let mut demands = self.prepare_stage(execution, &before_pin, candidate, physical)?;
            if let Some(cause) = demands.take_failure() {
                return Err(CoupledSourceReadStop::capture(cause));
            }
            pending.extend(demands.values.drain(..).map(PendingOutputDemand::Stage));
            let roots = demands.synchronous.iter().collect::<Vec<_>>();
            if let Some(next_order) = synchronous_outputs::CapturedSynchronousOutputs::seed(
                &roots,
                self,
                execution,
                &before_pin,
                candidate,
                &mut wave,
            )? {
                order = Some(next_order);
            }
            stage_demands.push(demands);
        }
        self.prepare_pending(execution, &before_pin, candidate, &mut pending, None)?;
        if let Some(order) = &order {
            let context = synchronous_outputs::SynchronousOutputAssembly {
                owner: self,
                execution,
                before: &before_pin,
                order,
            };
            wave.drive(&context, candidate, None)
                .map_err(|cause| cause.0)?;
        }
        Ok(())
    }
    fn prepare_pending(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &worth_relational::facade::runtime::RelationalPinnedRead,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        pending: &mut Vec<PendingOutputDemand<'_>>,
        allowed: Option<&[String]>,
    ) -> Result<(), CoupledSourceReadStop> {
        let mut position = 0;
        while position < pending.len() {
            if allowed.is_none_or(|families| {
                families
                    .iter()
                    .any(|family| family == pending[position].demand().family)
            }) {
                self.prepare_one(
                    execution,
                    before,
                    pending.remove(position).demand(),
                    candidate,
                )?;
            } else {
                position += 1;
            }
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn prepare_one(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before_pin: &worth_relational::facade::runtime::RelationalPinnedRead,
        demand: &crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<synchronous_outputs::AppliedOutputParent>, CoupledSourceReadStop> {
        let principal = self.principal.ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "coupled producer authentication",
            )
        })?;
        let registry = &self.application.installed_producers;
        let source_entry = registry.declared_coupled_source(demand)?;
        let source = source_entry.coupled_executor().read_coupled_source(
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
            execution::CoupledSourceRead::Retiring => return Ok(None),
            execution::CoupledSourceRead::Prepared(index) => {
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
                return Ok(Some(synchronous_outputs::AppliedOutputParent::new(
                    candidate.pin_source(execution)?,
                    writer.meaning.source_scope,
                )));
            }
            execution::CoupledSourceRead::Verified(parent) => {
                return self
                    .retain_verified_parent(execution, parent)
                    .map(Some)
                    .map_err(Into::into);
            }
            execution::CoupledSourceRead::Source(source) => source,
        };
        let lifecycle = source.lifecycle;
        let producer = registry.declared_coupled_producer(demand, source.profile, lifecycle)?;
        self.rounds.borrow_mut().start(
            execution,
            source.scope,
            demand.family,
            source.partition,
            &producer.declaration.identity,
        )?;
        let pin = candidate.pin_source(execution)?;
        let prepared = producer.coupled_executor().prepare_coupled_writer(
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
        let position = self.applied_position() + 1;
        self.writers.borrow_mut().push(CoupledWriter {
            position,
            producer: std::sync::Arc::clone(producer),
            stage: Some(CoupledWriterStage::Inspected(writer)),
            meaning: meaning.clone(),
        });
        match self
            .application
            .runtime
            .primary_provider
            .application_attempt_operation_control
            .take_coupled_participant_corruption(self.writers.borrow().len())
        {
            fault @ (1..=9) => candidate.append_corrupt_participant_for_test(
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
        Ok(Some(synchronous_outputs::AppliedOutputParent::new(
            after_own,
            source_scope,
        )))
    }
}
