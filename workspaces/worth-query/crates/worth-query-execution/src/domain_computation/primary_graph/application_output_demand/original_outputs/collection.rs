//! Complete original trees share one receipt-derived endpoint and currentness owner.
use super::*;
use crate::domain_computation::primary_graph::{
    structural_walk::memory, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

type Settlement = WorthQueryOutputDemandSettlement;
type Read = WorthQueryApplicationReadObservation;
type Denial = WorthQueryOriginalReadOutputDenial;

/// Retained complete SDK tree payload. It is neither a settlement nor a grant.
/// The history root-completion owner authenticates its root before capture.
pub(in crate::domain_computation::primary_graph) struct OriginalOutputTree {
    source: Arc<Read>,
    root: Arc<Settlement>,
    dependents: Vec<Arc<Settlement>>,
    dependencies: Option<WorthQueryRetainedQueryDependencies>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl OriginalOutputTree {
    pub(in crate::domain_computation::primary_graph) fn retain(
        execution: &ExecutionRequest<'_, '_>,
        source: &Arc<Read>,
        root: Arc<Settlement>,
        dependents: impl ExactSizeIterator<Item = Arc<Settlement>>,
        dependencies: &mut Option<WorthQueryRetainedQueryDependencies>,
        header_bytes: usize,
    ) -> Result<Self, Denial> {
        let count = dependents.len();
        let bytes = count
            .checked_mul(std::mem::size_of::<Arc<Settlement>>())
            .and_then(|bytes| bytes.checked_add(header_bytes))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| resource(Cause::ChargedBytesOverflow))?;
        let memory =
            memory::reserve(execution, execution.has_accounting(), bytes).map_err(resource)?;
        let mut retained = Vec::with_capacity(count);
        for dependent in dependents {
            checkpoint(execution)?;
            if retained.len() == count {
                return Err(foreign("original output dependent count"));
            }
            retained.push(dependent);
        }
        if retained.len() != count {
            return Err(foreign("original output dependent count"));
        }
        Ok(Self {
            source: Arc::clone(source),
            root,
            dependents: retained,
            dependencies: dependencies.take(),
            _memory: memory,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn require_current_at<S: ApplicationSchema>(
        &self,
        selected: crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            S,
        >,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        scope: &WorthQueryRequestScope,
    ) -> Result<(), Denial> {
        selected
            .qualify_completed_output_set(
                phase,
                access,
                scope,
                &self.root,
                self.dependents.iter().map(Arc::as_ref),
                self.dependencies.as_ref(),
                0,
            )
            .map(|_| ())
            .map_err(Denial::Currentness)
    }

    pub(in crate::domain_computation::primary_graph) fn settlements(
        &self,
    ) -> impl Iterator<Item = &Settlement> + Clone {
        std::iter::once(self.root.as_ref()).chain(self.dependents.iter().map(Arc::as_ref))
    }
}

/// Whole collection verification is performed only after deriving the endpoint
/// from all original publication movements. No intermediate sibling qualifies E.
pub(in crate::domain_computation::primary_graph) fn qualify_output_collection<
    'a,
    S: ApplicationSchema,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    access: &WorthQueryProgramPublicationAccess,
    scope: &WorthQueryRequestScope,
    trees: impl Iterator<Item = &'a OriginalOutputTree> + Clone,
) -> Result<WorthQueryCompletedOriginalReadOutputs, Denial> {
    let execution = phase
        .checked_execution_request_for(&runtime.runtime.product_runtime)
        .map_err(|cause| {
            Denial::Output(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
                "original output forest",
            ))
        })?;
    let first = trees
        .clone()
        .next()
        .ok_or_else(|| foreign("empty original output forest"))?;
    for tree in trees.clone() {
        checkpoint(&execution)?;
        if !Arc::ptr_eq(&first.source, &tree.source) {
            return Err(foreign("original output forest source"));
        }
    }
    let settlements = trees.clone().flat_map(OriginalOutputTree::settlements);
    let endpoint = super::super::original_endpoint::derive(
        &execution,
        &first.source,
        &mut settlements.clone(),
        &mut settlements.clone(),
    )
    .map_err(Denial::Output)?;
    let memory = memory::reserve(
        &execution,
        execution.has_accounting(),
        std::mem::size_of::<WorthQueryCompletedOriginalReadOutputs>() as u64,
    )
    .map_err(resource)?;
    let mut output = Arc::clone(&endpoint);
    for tree in trees {
        checkpoint(&execution)?;
        let selected = runtime
            .select_application_read_observation(phase, &endpoint)
            .map_err(Denial::Observation)?;
        let (verified, _zero_header) = selected
            .qualify_completed_output_set(
                phase,
                access,
                scope,
                &tree.root,
                tree.dependents.iter().map(Arc::as_ref),
                tree.dependencies.as_ref(),
                0,
            )
            .map_err(Denial::Currentness)?;
        output = verified;
    }
    Ok(WorthQueryCompletedOriginalReadOutputs {
        source: Arc::clone(&first.source),
        root: Arc::clone(first.root.retained_read_ref()),
        posture: first.root.posture(),
        output,
        _memory: memory,
    })
}

/// Recording may verify healthy or removed roots without ever opening an SDK
/// root. Only the original trees that ran contribute publication movements.
pub(in crate::domain_computation::primary_graph) fn qualify_recording_output_collection<
    'a,
    S: ApplicationSchema,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    access: &WorthQueryProgramPublicationAccess,
    scope: &WorthQueryRequestScope,
    source: &Arc<Read>,
    trees: impl Iterator<Item = &'a OriginalOutputTree> + Clone,
) -> Result<Arc<Read>, Denial> {
    let execution = phase
        .checked_execution_request_for(&runtime.runtime.product_runtime)
        .map_err(|cause| {
            Denial::Output(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
                "recording output forest",
            ))
        })?;
    for tree in trees.clone() {
        checkpoint(&execution)?;
        if !Arc::ptr_eq(source, &tree.source) {
            return Err(foreign("recording output forest source"));
        }
    }
    let settlements = trees.clone().flat_map(OriginalOutputTree::settlements);
    let endpoint = super::super::original_endpoint::derive(
        &execution,
        source,
        &mut settlements.clone(),
        &mut settlements.clone(),
    )
    .map_err(Denial::Output)?;
    for tree in trees {
        checkpoint(&execution)?;
        tree.require_current_at(
            runtime
                .select_application_read_observation(phase, &endpoint)
                .map_err(Denial::Observation)?,
            phase,
            access,
            scope,
        )?;
    }
    Ok(endpoint)
}

fn foreign(subject: &'static str) -> Denial {
    Denial::Output(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ForeignSettlement,
        subject,
    ))
}
fn resource(cause: Cause) -> Denial {
    Denial::Output(crate::domain_computation::primary_graph::output_reuse::read_denial(cause))
}
fn checkpoint(execution: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)
        })
        .map_err(|cause| resource(cause.into()))?
        .0
        .map_err(|cause| resource(worth_execution::WorkCeilingDenial::Stopped(cause).into()))
}
