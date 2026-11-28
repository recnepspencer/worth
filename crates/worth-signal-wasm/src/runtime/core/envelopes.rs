use std::collections::BTreeMap;
use std::sync::Arc;

use worth_signal::facade::adapters::{
    branch_state_proof_report, merge_plan_proof_report, merge_result_proof_report,
    replay_artifact_proof_report, replay_parity_proof_report, runtime_proof_report,
    BranchStateDenseGridProofBasis, BranchStateProofBasis, BranchStateProofReport,
    MergePlanProofReport, MergeResultProofReport, ReplayArtifactProofInput,
    ReplayArtifactProofReport, ReplayParityProofReport, RuntimeProofReport,
    BRANCH_STATE_PROOF_BASIS_VERSION,
};
use worth_signal::facade::adapters::{BranchMergePlan, BranchMergeResult};
use worth_signal::facade::history::RuntimeBranchId;

use crate::boundary::errors::WorthSignalJsError;
use crate::recipe::model::SourceSpec;
use crate::runtime::adapters::{
    HostCapabilityTransportArtifact, RuntimeDefinitionEnvelope, RuntimeEnvelope,
    UnavailableCallbackArtifact,
};
use crate::runtime::summaries::{
    public_callback_read_ids, RuntimeSnapshotEnvelope, RuntimeStoreSnapshot,
};

use super::state::{
    BranchRuntimeState, CallbackDiagnosticState, CatalogEntry, DenseGridFamily, RuntimeStore,
    StoredRecipeDefinition, WebRuntimeMetrics, WebSignalKind,
};
use super::RuntimeCore;

mod restore;

#[derive(Clone)]
/// In-memory exact state for the current root head.
///
/// Historical snapshots and sibling branches are owner-bound and are not
/// transferred to a newly constructed runtime by this artifact.
pub(crate) struct ExactRuntimeRestoreArtifact {
    snapshot: RuntimeSnapshotEnvelope,
    store: RuntimeStore,
    callback_diagnostics: BTreeMap<String, CallbackDiagnosticState>,
    catalog: BTreeMap<String, CatalogEntry>,
    web_signals: BTreeMap<String, WebSignalKind>,
    nodes_by_id: BTreeMap<worth_signal::facade::NodeId, String>,
    dense_grids: BTreeMap<String, Arc<DenseGridFamily>>,
    policy: crate::runtime::policy::RuntimePolicySpec,
    web_metrics: WebRuntimeMetrics,
}

const CALLBACK_UNAVAILABLE_FOR_PORTABLE_EXPORT: &str =
    "computeCallbackUnavailableForPortableExport";
#[cfg_attr(not(test), allow(dead_code))]
const CALLBACK_UNAVAILABLE_FOR_RUNTIME_ENVELOPE_IMPORT: &str =
    "computeCallbackUnavailableForRuntimeEnvelopeImport";

fn host_capability_transport_artifacts(
    host_capability_reads: &[crate::runtime::compute_callbacks::CapturedHostCapabilityRead],
) -> Vec<HostCapabilityTransportArtifact> {
    host_capability_reads
        .iter()
        .map(|read| {
            let (portable_import_outcome, portable_import_reason) =
                portable_import_outcome_for_compatibility(&read.compatibility);
            HostCapabilityTransportArtifact {
                family: read.family.clone(),
                registration_id: read.registration_id.clone(),
                compatibility: read.compatibility.clone(),
                exact_restore_outcome: "Live".to_owned(),
                portable_import_outcome: portable_import_outcome.to_owned(),
                portable_import_reason: portable_import_reason.to_owned(),
            }
        })
        .collect()
}

fn portable_import_outcome_for_compatibility(compatibility: &str) -> (&'static str, &'static str) {
    match compatibility {
        "LiveOnly" => (
            "Denied",
            "live-only host capabilities require the exact originating runtime and cannot cross portable runtime-envelope import",
        ),
        "Reattachable" => (
            "Unavailable",
            "equivalent host capability reattachment is required before portable import can resume live reevaluation",
        ),
        "SnapshotPortable" => (
            "Unavailable",
            "committed snapshot truth can travel, but the live host capability itself is not transported by portable runtime-envelope import",
        ),
        "ImportDenied" => (
            "Denied",
            "this host capability family explicitly denies portable import outside the originating runtime",
        ),
        _ => (
            "Incompatible",
            "host capability compatibility posture was not recognized by the portable import boundary",
        ),
    }
}

impl RuntimeCore {
    pub fn export_definitions(&mut self) -> Result<RuntimeDefinitionEnvelope, WorthSignalJsError> {
        let store = self.lock_store()?;
        let sources = store
            .sources
            .iter()
            .map(|(id, source)| SourceSpec {
                id: id.clone(),
                initial: source.value.clone(),
                produces_aspects: self.catalog.get(id).map(|entry| {
                    entry
                        .produced_aspects
                        .iter()
                        .map(|aspect| aspect.id())
                        .collect()
                }),
            })
            .collect();
        let recipes = store
            .recipes
            .values()
            .filter_map(|recipe| recipe.definition.exportable_spec().cloned())
            .collect();
        let source_families = store
            .source_families
            .values()
            .map(|family| family.spec.clone())
            .collect();
        let recipe_families = store
            .recipe_families
            .values()
            .map(|family| family.spec.clone())
            .collect();
        let unavailable_callbacks: Vec<UnavailableCallbackArtifact> = store
            .recipes
            .iter()
            .filter_map(|(id, recipe)| match &recipe.definition {
                StoredRecipeDefinition::Expr(_) => None,
                StoredRecipeDefinition::Callback(callback) => {
                    let current_reads = callback
                        .reads
                        .iter()
                        .map(|read| read.id().to_owned())
                        .collect::<Vec<_>>();
                    Some(UnavailableCallbackArtifact {
                        id: id.clone(),
                        signal_kind: self
                            .web_signals
                            .get(id)
                            .map(|kind| match kind {
                                super::state::WebSignalKind::Input => "input".to_owned(),
                                super::state::WebSignalKind::Computed => "computed".to_owned(),
                                super::state::WebSignalKind::Output => "output".to_owned(),
                            })
                            .unwrap_or_else(|| "computed".to_owned()),
                        reason: CALLBACK_UNAVAILABLE_FOR_PORTABLE_EXPORT.to_owned(),
                        current_reads: public_callback_read_ids(&current_reads),
                        host_capability_reads: callback.host_capability_reads.clone(),
                        host_capability_transports: host_capability_transport_artifacts(
                            &callback.host_capability_reads,
                        ),
                    })
                }
            })
            .collect();
        drop(store);
        self.web_metrics
            .compute_callback_missing_unavailability_count = self
            .web_metrics
            .compute_callback_missing_unavailability_count
            .saturating_add(unavailable_callbacks.len() as u64);
        let worker_public_output_ids = self
            .web_signals
            .iter()
            .filter_map(|(id, kind)| {
                if matches!(kind, super::state::WebSignalKind::Output) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(RuntimeDefinitionEnvelope {
            policy: self.policy.clone(),
            sources,
            recipes,
            source_families,
            recipe_families,
            worker_public_output_ids,
            unavailable_callbacks,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn export_runtime_envelope(&mut self) -> Result<RuntimeEnvelope, WorthSignalJsError> {
        self.preflight_runtime_envelope_export()?;
        Ok(RuntimeEnvelope {
            definitions: self.export_definitions()?,
            snapshot: self.snapshot()?,
        })
    }

    pub(crate) fn export_exact_runtime_restore_artifact(
        &mut self,
    ) -> Result<ExactRuntimeRestoreArtifact, WorthSignalJsError> {
        self.preflight_runtime_envelope_export()?;
        let snapshot = self.snapshot()?;
        let store = self.lock_store()?.clone();
        let callback_diagnostics = self.lock_callback_diagnostics()?.clone();
        Ok(ExactRuntimeRestoreArtifact {
            snapshot,
            store,
            callback_diagnostics,
            catalog: self.catalog.clone(),
            web_signals: self.web_signals.clone(),
            nodes_by_id: self.nodes_by_id.clone(),
            dense_grids: self.dense_grids.clone(),
            policy: self.policy.clone(),
            web_metrics: self.web_metrics.clone(),
        })
    }

    pub(crate) fn preflight_runtime_envelope_export(&self) -> Result<(), WorthSignalJsError> {
        if self.runtime.current_branch().parent_branch_id.is_some() {
            return Err(WorthSignalJsError::invalid_input(
                "exact runtime restore artifacts require the root branch to be active",
            ));
        }
        Ok(())
    }

    pub fn runtime_proof_report(&self) -> RuntimeProofReport {
        runtime_proof_report(
            self.runtime.schema_registry().registry_digest(),
            self.runtime.merge_strategy_registry().registry_digest(),
            self.runtime
                .merge_base_strategy_registry()
                .registry_digest(),
            self.runtime
                .aspect_merge_policy_registry()
                .registry_digest(),
            self.runtime.conflict_isolation_registry().registry_digest(),
            self.runtime.conflict_policy_registry().registry_digest(),
            self.runtime.identity_matcher_registry().registry_digest(),
            self.runtime.source_only_policy_registry().registry_digest(),
            self.runtime.deletion_policy_registry().registry_digest(),
        )
    }

    pub fn branch_state_proof(
        &self,
        branch_id: u64,
    ) -> Result<BranchStateProofReport, WorthSignalJsError> {
        let branch = self
            .runtime
            .branch_handle(RuntimeBranchId(branch_id))
            .ok_or_else(|| {
                WorthSignalJsError::invalid_input(format!("unknown branch `{branch_id}`"))
            })?;
        let state = self.state_for_branch(branch_id);
        Ok(branch_state_proof_report(
            branch_id,
            branch.name,
            branch.head_snapshot_id.map(|id| id.0),
            BRANCH_STATE_PROOF_BASIS_VERSION,
            &build_branch_state_proof_basis(&state),
        ))
    }

    pub fn replay_parity_proof(
        &self,
        expected_branch_id: u64,
        replayed_branch_id: u64,
    ) -> Result<ReplayParityProofReport, WorthSignalJsError> {
        let expected = self.branch_state_proof(expected_branch_id)?;
        let replayed = self.branch_state_proof(replayed_branch_id)?;
        Ok(replay_parity_proof_report(
            expected.branch_id,
            expected.branch_name,
            expected.snapshot_id,
            expected.state_digest,
            replayed.branch_id,
            replayed.branch_name,
            replayed.snapshot_id,
            replayed.state_digest,
        ))
    }

    pub fn replay_artifact_proof(
        &self,
        expected: ReplayArtifactProofInput,
        replayed_branch_id: u64,
    ) -> Result<ReplayArtifactProofReport, WorthSignalJsError> {
        let replayed_state = self.branch_state_proof(replayed_branch_id)?;
        let runtime_proof = self.runtime_proof_report();
        Ok(replay_artifact_proof_report(
            expected,
            ReplayArtifactProofInput {
                proof_schema_version: runtime_proof.proof_schema_version.clone(),
                registry_bundle_digest: Some(runtime_proof.registry_bundle_digest),
                lowered_strategy_bundle_digest: None,
                merge_plan_digest: None,
                merge_result_digest: None,
                lineage_digest: None,
                strategy_witness: None,
                compatibility_witness: None,
                scoped_merge_proof: None,
                branch_state_digest: replayed_state.state_digest,
            },
        ))
    }

    pub(super) fn merge_plan_proof_report(
        &self,
        plan: &BranchMergePlan,
    ) -> Result<MergePlanProofReport, WorthSignalJsError> {
        Ok(merge_plan_proof_report(
            plan,
            &self.runtime_proof_report().registry_bundle_digest,
        ))
    }

    pub(super) fn merge_result_proof_report(
        &self,
        result: &BranchMergeResult,
    ) -> Result<MergeResultProofReport, WorthSignalJsError> {
        Ok(merge_result_proof_report(result))
    }
}

fn build_branch_state_proof_basis(
    state: &BranchRuntimeState,
) -> BranchStateProofBasis<RuntimeStoreSnapshot> {
    let mut catalog_ids = state.metadata.catalog.keys().cloned().collect::<Vec<_>>();
    catalog_ids.sort();
    let mut dense_grids = state
        .metadata
        .dense_grids
        .iter()
        .map(|(family_id, grid)| BranchStateDenseGridProofBasis {
            family_id: family_id.clone(),
            width: grid.width,
            height: grid.height,
            key_count: grid.key_to_index.len(),
            ids: grid.ids.clone(),
        })
        .collect::<Vec<_>>();
    dense_grids.sort_by(|left, right| left.family_id.cmp(&right.family_id));
    for grid in &mut dense_grids {
        grid.ids.sort();
    }
    BranchStateProofBasis {
        proof_schema_version: BRANCH_STATE_PROOF_BASIS_VERSION.to_owned(),
        catalog_ids,
        dense_grids,
        store: state.store.clone(),
    }
}
