//! `SourceQueryReaderPort` implementation for `SourceQueryReaderAccess`.

use std::any::Any;
use std::sync::Arc;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;

use worth_query_execution_lineage::facade::ports::{
    AppliedOutputsSelection, SourceQueryReaderPort,
};
use worth_query_installation::facade::{ApplicationSchema, InstalledOperationView};
use worth_relational::facade::{
    identity::EntityId,
    mvcc::{RelationalPredecessorInput, RelationalRequiredOutputPublication},
    runtime::{
        PositionedRelationalSnapshot, RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
        RelationalRuntime,
    },
};

use crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization;
use crate::domain_computation::primary_graph::application_contribution::{
    producer::observation_reads::{
        WorthQueryHistoricalProducerRead, WorthQueryProducerObservationReadDenial as Denial,
    },
    QualifiedOutputSource,
};
use crate::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryReaderAccess;
use crate::domain_computation::primary_graph::WorthQueryCurrentOutputDenial;

impl<S: ApplicationSchema + 'static> SourceQueryReaderPort for SourceQueryReaderAccess<'_, S> {
    fn applied_outputs(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        family: &'static str,
        producer: EntityId,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<AppliedOutputsSelection, WorthQueryCurrentOutputDenial> {
        let Some(participants) = self.applied_participants else {
            return Ok(AppliedOutputsSelection {
                replaced: Vec::new(),
                outputs: Vec::new(),
            });
        };
        let found = self.application.runtime.primary_provider.applied_outputs(
            execution,
            binding_identity,
            family,
            producer,
            participants,
            pin,
        )?;
        Ok(AppliedOutputsSelection {
            replaced: found.replaced,
            outputs: found.outputs,
        })
    }

    fn with_committed_runtime_dyn(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        visit: &mut dyn FnMut(&RelationalRuntime),
    ) {
        self.application
            .runtime
            .primary_provider
            .graph_ref()
            .with_runtime(execution, |native| visit(native));
    }

    fn applied_source_publication(
        &self,
    ) -> Option<(&QualifiedOutputSource, &PositionedRelationalSnapshot)> {
        self.applied_source_publication
            .as_ref()
            .map(|loan| (&loan.source, &loan.selected))
    }

    fn observation_control(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(), Denial> {
        SourceQueryReaderAccess::observation_control(self, execution, native)
    }

    fn record_observation_failure(&self, cause: &Denial) {
        SourceQueryReaderAccess::record_observation_failure(self, cause);
    }

    fn record_predecessor_read(&self, read: WorthQueryHistoricalProducerRead) {
        SourceQueryReaderAccess::record_predecessor_read(self, read);
    }

    fn authorize_observation_view(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        installed_view: InstalledOperationView<'_>,
    ) -> Result<Arc<SourceReadAuthorization>, Denial> {
        self.authorize_observation_view(execution, native, installed_view)
    }

    fn select_predecessor_input(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        scope: EntityId,
        partition: [u8; 32],
    ) -> Result<
        (
            RelationalRequiredOutputPublication,
            RelationalPredecessorInput,
        ),
        Denial,
    > {
        self.select_predecessor_input(execution, native, scope, partition)
    }

    fn observation_components_any(
        &self,
    ) -> (
        &dyn Any,
        &dyn Any,
        &dyn Any,
        Option<&dyn Any>,
        Option<&dyn Any>,
    ) {
        self.observation_components_any()
    }
}
