//! One tree progression for the actual ordinary or history Native custody.
use super::*;
impl<Schema, Program, Root>
    WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    RootDemand<Schema, Root>: Clone,
    Value<Schema, RootDemand<Schema, Root>>:
        WorthQueryApplicationProjection<Schema, Query<Schema, RootDemand<Schema, Root>>> + Clone,
    <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = Source<Schema, RootDemand<Schema, Root>>,
        >,
    <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Root::Dependents:
        ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
{
    /// Exact original read basis held by this continuation, distinct from later outputs.
    pub fn source_observation(&self) -> &crate::application_entry::WorthQueryApplicationReadObservation { &self.source_observation }

    pub fn notifications(
        &self,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryOutputDemandNotifications,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        self.root
            .as_ref()
            .ok_or(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed)?
            .notifications()
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)
    }

    pub fn settled_root_observation(
        &self,
    ) -> Option<&crate::application_entry::WorthQueryApplicationReadObservation> {
        self.root_settlement.as_ref().map(
            crate::application_entry::WorthQueryApplicationOutputDemandSettlement::observation,
        )
    }

    pub fn settle(
        &mut self,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            '_,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<
        WorthQueryApplicationProgramOutputProgress<
            Query<Schema, RootDemand<Schema, Root>>,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        application.runtime().with_application_advancement(request.scope, |phase| {
            let mut remaining = self.controls
                .resolve(application.runtime().output_demand_resource_profile())
                .settlement_attempts();
            loop {
                if remaining == Some(0) { break; }
                remaining = remaining.map(|steps| steps - 1);
                let progress = self.advance_in_advancement(&phase, application, request)?;
                match progress {
                    WorthQueryApplicationProgramOutputProgress::Settled(_)
                    | WorthQueryApplicationProgramOutputProgress::Pending => return Ok(progress),
                    WorthQueryApplicationProgramOutputProgress::Progressed => {
                        if remaining == Some(0) { return Ok(progress); }
                    }
                }
            }
            Ok(WorthQueryApplicationProgramOutputProgress::Pending)
        }).map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::advancement)?
    }

    pub fn advance(
        &mut self,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            '_,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<
        WorthQueryApplicationProgramOutputProgress<
            Query<Schema, RootDemand<Schema, Root>>,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        application.runtime().with_application_advancement(request.scope, |phase| self.advance_in_advancement(&phase, application, request))
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::advancement)?
    }

    pub(in crate::application_entry) fn advance_in_advancement(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryApplicationProgramOutputProgress<Query<Schema, RootDemand<Schema, Root>>>, crate::application_entry::WorthQueryRequiredOutputPreparationDenial> {
        self.advance_tree(phase, application, request, None)
    }

    pub(super) fn advance_tree(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            '_,
            '_,
            '_,
            Schema,
        >,
        history: Option<super::super::program_output_continuation::ProgramOutputHistoryFrame<'_, '_, '_>>,
    ) -> Result<
        WorthQueryApplicationProgramOutputProgress<
            Query<Schema, RootDemand<Schema, Root>>,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        if history.is_none()
            && let Some(root) = &self.root {
                root.require_ordinary().map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?;
            }
        if self.complete {
            return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed);
        }
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(crate::application_entry::WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch));
        }
        let access = worth_query_execution::publication_boundary::program_publication_access();
        if history.is_none() { match (&self.source_receipt, &self.source_preparation) {
            (Some(receipt), Some(prepared)) => match self.source_binding.as_ref() {
                Some(binding) => (binding.validate)(phase, application, prepared, receipt, &self.source_observation, request),
                None => return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery),
            },
            (None, None) => application.validate_initial_program_source::<Root>(phase, &access, &self.source_observation.retained, request.branch),
            _ => return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery),
        }.map_err(|denial| crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
            if denial.kind() == worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded {
                crate::application_entry::WorthQueryApplicationOutputDemandDenial::Superseded
            } else { crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial) }))?; }
        if self.root_settlement.is_none() {
            let root = self.root.as_mut().expect("an unsettled root retains its admitted demand");
            match root
                .advance_with_frame(phase, application, request, history)
                .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?
            {
                WorthQueryApplicationProgramDemandProgress::Progressed => {
                    return Ok(WorthQueryApplicationProgramOutputProgress::Progressed);
                }
                WorthQueryApplicationProgramDemandProgress::Pending => {
                    return Ok(WorthQueryApplicationProgramOutputProgress::Pending);
                }
                WorthQueryApplicationProgramDemandProgress::Settled {
                    settlement,
                    authority,
                } => {
                    self.root_settlement = Some(settlement);
                    self.pending_root_authority = Some(authority);
                    // Keep the actual required interest until every descendant
                    // has sealed its consumed-output claims. A native receipt
                    // alone does not keep its registry settlement posting live.
                    // Return custody before independent discovery needs fresh
                    // admission after potentially long native producer work.
                    return Ok(WorthQueryApplicationProgramOutputProgress::Progressed);
                }
            }
        }
        self.retain_root_fulfillment(phase, application)?;
        // A refused start keeps the original settled authority for retry.
        if let Some(authority) = self.pending_root_authority.as_ref() {
            let settlement = self
                .root_settlement
                .as_ref()
                .expect("a settled root retains its output settlement");
            self.continuation = Some(Root::Dependents::start(
                application,
                &self.root_demand,
                settlement,
                authority,
                &self.source_observation,
                request,
                self.controls,
            )?);
            if let Some((origin, index)) = &self.recording_recipes {
                self.continuation.as_mut().expect("constructed ordinary continuation").record_history_discovery(std::sync::Arc::clone(origin), *index);
            }
            self.pending_root_authority = None;
        }
        let continuation = self
            .continuation
            .as_mut()
            .expect("a settled root installs its typed output continuation");
        match continuation.advance(phase, application, request, history)? {
            ProgramOutputContinuationProgress::Progressed => {
                Ok(WorthQueryApplicationProgramOutputProgress::Progressed)
            }
            ProgramOutputContinuationProgress::Pending => {
                Ok(WorthQueryApplicationProgramOutputProgress::Pending)
            }
            ProgramOutputContinuationProgress::Settled { outputs, work, dependencies } => {
                if let Some(source) = &self.source_receipt {
                    application.complete_program_output_source(
                        phase,
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        source,
                    ).map_err(|cause| crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
                        crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(cause)))?;
                }
                let performed_source = match (&self.source_receipt, &self.source_preparation) {
                    (Some(receipt), Some(_)) => Some(
                        super::super::program_output_settlement::WorthQueryPerformedProgramOutputSource::capture(receipt.clone(), self.pending_root_fulfillment.take().expect("a performed root seals fulfillment before descendants"))),
                    (None, None) => None,
                    _ => return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery),
                };
                let original_root = self.root.as_ref().and_then(|demand| demand.original_root_result());
                self.complete = true;
                self.source_preparation = None;
                // A read-origin tree keeps its actual interests through explicit
                // close. Performed-source completion keeps its existing release.
                if performed_source.is_some() {
                    self.continuation = None;
                    self.root = None;
                }
                let root = self
                    .root_settlement
                    .take()
                    .expect("output progression retains its root settlement");
                let current_root = root;
                let root = original_root.unwrap_or_else(||
                    crate::application_entry::WorthQueryApplicationOutputDemandSettlement::new(
                        current_root.retain_settlement(), current_root.observed_source().clone(),
                    ));
                let program_work = super::super::program_output_work::WorthQueryApplicationProgramWork::from_settlements(
                    work,
                    (
                        current_root.application_commit_receipt(),
                        current_root.readiness_delivery(),
                    ),
                    outputs.iter().map(|output| (output.receipt(), output.readiness_delivery())),
                );
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(
                    WorthQueryApplicationProgramOutputSettlement {
                        performed_source,
                        root,
                        current_root,
                        outputs,
                        work: program_work,
                        dependencies,
                    },
                ))
            }
        }
    }
}
