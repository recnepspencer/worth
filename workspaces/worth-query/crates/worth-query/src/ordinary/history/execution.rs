use super::{
    WorthQueryHistoricalCompletion, WorthQueryHistoricalJourneyCounters,
    WorthQueryHistoricalNextAction, WorthQueryHistoricalOutcome, WorthQueryHistoricalPathKind,
    WorthQueryHistoricalRequest, WorthQueryHistoricalStop, WorthQueryHistoricalStopSource,
};
use crate::basis::{preflight_execution_basis, resolve_runtime_historical_snapshot_basis};
use crate::basis_lifecycle::basis_lifecycle;
use crate::historical::{
    admit_historical_evaluation_path, materialization_metadata_from_resolved,
    resolve_historical_materialization_path, HistoricalCapabilityDescriptor,
    HistoricalEvaluationRequest, HistoricalMaterializationDescriptor,
    HistoricalPathReuseDescriptor,
};
use crate::ordinary::read::{admit_read_context_declaration, current};
use crate::query_context::{
    admit_query_basis_context, QueryContextBindingSource, ScopedQueryBasisContext,
};
use crate::runtime::WorthQueryWorkspace;

impl WorthQueryHistoricalRequest {
    /// Execute one historical query. The in-memory runtime currently owns one
    /// real historical materialization: the exact retained snapshot captured
    /// by the context. Other paths remain explicit typed stops for store-backed
    /// implementations to satisfy later through the same request contract.
    pub fn run(self, workspace: &mut WorthQueryWorkspace) -> WorthQueryHistoricalOutcome {
        let counters = WorthQueryHistoricalJourneyCounters::begin();
        if self.declaration.path != WorthQueryHistoricalPathKind::RetainedSnapshot {
            return stopped(
                WorthQueryHistoricalStopSource::HistoryUnavailable,
                WorthQueryHistoricalNextAction::SupplyAvailableHistory,
                "the in-memory runtime has no replay or reconstruction materialization",
                counters,
            );
        }
        let counters = counters.admit_path();
        let snapshot = match workspace.snapshot_identity() {
            Ok(snapshot) => snapshot,
            Err(denial) => {
                return stopped(
                    WorthQueryHistoricalStopSource::Handle(denial),
                    WorthQueryHistoricalNextAction::ResolveAuthority,
                    denial.to_string(),
                    counters,
                )
            }
        };
        if self.context.workspace_name() != workspace.name()
            || !self.context.admits_snapshot(&snapshot)
        {
            return stopped(
                WorthQueryHistoricalStopSource::StaleContext,
                WorthQueryHistoricalNextAction::RefreshContext,
                "the captured retained snapshot does not belong to the current workspace truth basis",
                counters,
            );
        }

        let intent = self.declaration.read.into_declared_intent();
        let schema_basis = intent.schema_basis().clone();
        let counters = counters.admit_context();
        let admitted = match admit_read_context_declaration(&intent, current().into()) {
            Ok(admitted) => admitted,
            Err(error) => {
                return stopped(
                    WorthQueryHistoricalStopSource::ContextAdmission,
                    WorthQueryHistoricalNextAction::ResolveAuthority,
                    format!("historical query context was denied: {error:?}"),
                    counters,
                );
            }
        };
        let (_, planning_authority, context_receipt) = admitted.into_parts();
        let counters = counters.plan();
        let graph = match intent.plan_runtime_historical(planning_authority) {
            Ok(graph) => graph,
            Err(error) => {
                return stopped(
                    WorthQueryHistoricalStopSource::Planning,
                    WorthQueryHistoricalNextAction::ReviseDeclaration,
                    format!("historical query planning was denied: {error:?}"),
                    counters,
                );
            }
        };

        let basis = resolve_runtime_historical_snapshot_basis(
            self.context.snapshot_identity().evidence_identity(),
            schema_basis,
            self.context.workspace_name(),
        );
        let query_preflight =
            match preflight_execution_basis(graph.execution_plan().clone(), basis.clone()) {
                Ok(preflight) => preflight,
                Err(error) => {
                    return stopped(
                        WorthQueryHistoricalStopSource::BasisAdmission,
                        WorthQueryHistoricalNextAction::ResolveAuthority,
                        format!("historical basis preflight was denied: {error:?}"),
                        counters,
                    );
                }
            };
        let counters = counters.bind_basis();
        let (basis_context, materialization) = match retained_basis_context(
            &self.context.basis_label(),
            &basis,
            &query_preflight,
            self.declaration.replay_budget,
            self.declaration.reconstruction_budget,
        ) {
            Ok(parts) => parts,
            Err(reason) => {
                return stopped(
                    WorthQueryHistoricalStopSource::BasisAdmission,
                    WorthQueryHistoricalNextAction::ResolveAuthority,
                    reason,
                    counters,
                );
            }
        };

        let counters = counters.execute();
        match workspace.execute_declared_read_graph_in_basis_context(graph, &basis_context) {
            Ok(result) => {
                WorthQueryHistoricalOutcome::Completed(WorthQueryHistoricalCompletion::new(
                    result,
                    context_receipt,
                    materialization,
                    counters,
                ))
            }
            Err(error) => stopped(
                WorthQueryHistoricalStopSource::Runtime,
                WorthQueryHistoricalNextAction::RetryRuntime,
                format!("historical lower-runtime execution failed: {error:?}"),
                counters,
            ),
        }
    }
}

fn retained_basis_context(
    basis_label: &str,
    basis: &crate::basis::ResolvedSnapshotBasis,
    query_preflight: &crate::basis::ExecutionPreflightBundle,
    replay_budget: usize,
    reconstruction_budget: usize,
) -> Result<
    (
        ScopedQueryBasisContext,
        crate::historical::HistoricalMaterializationPathMetadata,
    ),
    String,
> {
    let reuse = HistoricalPathReuseDescriptor::retained_reuse();
    let request = HistoricalEvaluationRequest::retained_snapshot(
        basis.proof(),
        replay_budget,
        reconstruction_budget,
        reuse.clone(),
    );
    let capability = HistoricalCapabilityDescriptor::retained_snapshot(basis.proof(), reuse);
    let admission = admit_historical_evaluation_path(request, capability)
        .map_err(|error| format!("retained historical path was denied: {error:?}"))?;
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot(basis.proof()),
    )
    .map_err(|error| format!("retained historical materialization was denied: {error:?}"))?;
    let metadata = materialization_metadata_from_resolved(resolved);
    let declaration = basis_lifecycle().historical_snapshot(basis_label, true);
    let context = admit_query_basis_context(
        declaration,
        QueryContextBindingSource::Historical {
            query_preflight,
            admission: &admission,
            metadata: &metadata,
        },
    )
    .map_err(|error| format!("historical basis context was denied: {error:?}"))?;
    Ok((context, metadata))
}

fn stopped(
    source: WorthQueryHistoricalStopSource,
    next_action: WorthQueryHistoricalNextAction,
    reason: impl Into<String>,
    counters: WorthQueryHistoricalJourneyCounters,
) -> WorthQueryHistoricalOutcome {
    WorthQueryHistoricalOutcome::Stopped(WorthQueryHistoricalStop::new(
        source,
        next_action,
        reason,
        counters,
    ))
}
