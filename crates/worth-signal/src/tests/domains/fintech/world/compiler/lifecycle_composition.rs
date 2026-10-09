use std::collections::{BTreeMap, BTreeSet};

use crate::data::aspect::Aspect;
use crate::data::async_node::{
    AsyncNodeAdmissionClass, AsyncNodeCapabilityDeclaration, AsyncNodeConditionBlockClass,
    AsyncNodePayloadContract, AsyncNodePayloadContractId, AsyncNodeRequestIntent,
};
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use crate::diagnostics::{replay_slices_equivalent, ReplayEventKind};
use crate::facade::{DiagnosticsTier, NodeState, SignalRuntimePolicy};

use super::super::{
    FinancialEconomicSnapshot, FinancialSemanticProjection, FinancialWorldDefinition, InstrumentId,
    MarketFactorKey, SemanticOutputKey,
};
use super::evaluation::FinancialEvaluationProgram;
use super::runtime_finance::runtime_financial_snapshot;
use super::CompiledFinancialWorld;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct FinancialCauseFingerprint {
    consumer: NodeId,
    dependency_revision: u64,
    producer: NodeId,
    aspect: Aspect,
    edge_scope: Option<PartitionSubscription>,
    cached_version: u64,
    output_commit_ordinal: u64,
    committed_version: u64,
    changed_scopes: Vec<PartitionSubscription>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct FinancialBranchLifecycleEvidence {
    pub(in crate::tests::domains::fintech) diagnostics_tier: DiagnosticsTier,
    pub(in crate::tests::domains::fintech) diagnostics_profile_observed: DiagnosticsTier,
    pub(in crate::tests::domains::fintech) active_nodes_observed: u32,
    pub(in crate::tests::domains::fintech) analysis_causes_before_capture:
        Vec<FinancialCauseFingerprint>,
    pub(in crate::tests::domains::fintech) analysis_causes_after_restore:
        Vec<FinancialCauseFingerprint>,
    pub(in crate::tests::domains::fintech) main_pending_isolated: bool,
    pub(in crate::tests::domains::fintech) async_dependency_blocked: bool,
    async_after_restore_blocked: bool,
    async_after_settlement_admitted: bool,
    pub(in crate::tests::domains::fintech) replay_branch_local: bool,
    pub(in crate::tests::domains::fintech) replay_has_restore: bool,
    pub(in crate::tests::domains::fintech) final_snapshot: FinancialEconomicSnapshot,
    pub(in crate::tests::domains::fintech) observed_work: BTreeSet<SemanticOutputKey>,
    committed_values: BTreeMap<SemanticOutputKey, i64>,
    replay: crate::diagnostics::replay::ReplaySlice,
}

#[derive(Clone)]
pub(in crate::tests::domains::fintech) struct FinancialBranchLifecycleCompletion {
    _seal: BranchLifecycleSeal,
}

#[derive(Clone)]
struct BranchLifecycleSeal;

impl FinancialBranchLifecycleEvidence {
    pub(in crate::tests::domains::fintech) fn verifies_lifecycle(&self) -> bool {
        self.diagnostics_profile_observed == self.diagnostics_tier
            && self.active_nodes_observed > 0
            && !self.analysis_causes_before_capture.is_empty()
            && self.analysis_causes_before_capture == self.analysis_causes_after_restore
            && self.main_pending_isolated
            && self.async_dependency_blocked
            && self.async_after_restore_blocked
            && self.async_after_settlement_admitted
            && self.replay_branch_local
            && self.replay_has_restore
    }

    pub(in crate::tests::domains::fintech) fn operationally_matches(&self, other: &Self) -> bool {
        self.diagnostics_tier != other.diagnostics_tier
            && self.analysis_causes_before_capture == other.analysis_causes_before_capture
            && self.analysis_causes_after_restore == other.analysis_causes_after_restore
            && self.main_pending_isolated == other.main_pending_isolated
            && self.async_dependency_blocked == other.async_dependency_blocked
            && self.async_after_restore_blocked == other.async_after_restore_blocked
            && self.async_after_settlement_admitted == other.async_after_settlement_admitted
            && self.replay_branch_local == other.replay_branch_local
            && self.replay_has_restore == other.replay_has_restore
            && self.final_snapshot == other.final_snapshot
            && self.observed_work == other.observed_work
            && self.committed_values == other.committed_values
            && replay_slices_equivalent(&self.replay, &other.replay)
    }

    pub(in crate::tests::domains::fintech) fn certify_tier_pair(
        self,
        other: Self,
    ) -> Result<FinancialBranchLifecycleCompletion, SignalError> {
        if !self.verifies_lifecycle()
            || !other.verifies_lifecycle()
            || !self.operationally_matches(&other)
        {
            return Err(SignalError::internal(
                "financial branch lifecycle tiers did not preserve identical causal truth",
            ));
        }
        Ok(FinancialBranchLifecycleCompletion {
            _seal: BranchLifecycleSeal,
        })
    }
}

impl CompiledFinancialWorld {
    pub(in crate::tests::domains::fintech) fn exercise_branch_restore_replay(
        &mut self,
        base: FinancialWorldDefinition,
        analysis_definition: FinancialWorldDefinition,
        analysis_factor: MarketFactorKey,
        main_definition: FinancialWorldDefinition,
        main_factor: MarketFactorKey,
        instrument: InstrumentId,
        diagnostics_tier: DiagnosticsTier,
    ) -> Result<FinancialBranchLifecycleEvidence, SignalError> {
        self.runtime.set_runtime_policy(
            SignalRuntimePolicy::for_tier(diagnostics_tier)
                .with_history_limit(8)
                .with_detail_limit(4)
                .with_observation_activation(
                    worth_foundational::ObservationActivationProfile::Continuous,
                ),
        );
        let risk = self.handles.position(instrument).risk;
        self.runtime
            .declare_async_node_capability(AsyncNodeCapabilityDeclaration::new(
                risk,
                AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(12)),
            ))?;
        let main = self.runtime.current_branch();
        self.runtime.capture_snapshot()?;
        let analysis = self.runtime.create_branch("m12-causal-analysis")?;
        self.runtime.switch_branch(analysis.clone())?;
        self.stage_factor_change(analysis_definition.clone(), analysis_factor)?;
        let valuation = self.handles.position(instrument).valuation;
        self.settle_valuation_wave()?;
        let analysis_causes_before_capture =
            cause_fingerprints(self.runtime.graph().pending_causes(risk)?);
        let async_report = self.runtime.admit_async_node_request(
            AsyncNodeRequestIntent::new(risk).require_clean_dependencies(),
        )?;
        let async_dependency_blocked = async_report.classification().class()
            == AsyncNodeAdmissionClass::BlockedByCondition
            && async_report.classification().condition_block_class()
                == Some(AsyncNodeConditionBlockClass::DependencyNotReady)
            && async_report.resource_admission().is_none();
        let analysis_work_before_capture = self.ledger.observed_work();
        let analysis_snapshot = self.runtime.capture_snapshot()?;
        self.settle_current_definition()?;

        self.runtime.switch_branch(main.clone())?;
        self.install_definition_state(&base, base.clone());
        self.stage_factor_change(main_definition, main_factor)?;
        self.settle_valuation_wave()?;
        let main_causes = cause_fingerprints(self.runtime.graph().pending_causes(risk)?);
        let main_pending_isolated =
            !main_causes.is_empty() && main_causes.iter().all(|cause| cause.producer == valuation);
        self.runtime.capture_snapshot()?;

        self.runtime
            .restore_branch_snapshot(analysis.clone(), &analysis_snapshot)?;
        self.runtime.switch_branch(analysis.clone())?;
        self.install_definition_state(&base, analysis_definition);
        let analysis_causes_after_restore =
            cause_fingerprints(self.runtime.graph().pending_causes(risk)?);
        let post_restore_async = self.runtime.admit_async_node_request(
            AsyncNodeRequestIntent::new(risk).require_clean_dependencies(),
        )?;
        let async_after_restore_blocked = dependency_not_ready(&post_restore_async);
        self.ledger.clear();
        for key in analysis_work_before_capture {
            self.ledger.record(key);
        }
        self.settle_current_definition()?;
        let settled_async = self.runtime.admit_async_node_request(
            AsyncNodeRequestIntent::new(risk).require_clean_dependencies(),
        )?;
        let async_after_settlement_admitted = settled_async.resource_admission().is_some()
            && settled_async.classification().class()
                != AsyncNodeAdmissionClass::BlockedByCondition;
        let observed_work = self.ledger.observed_work();
        let replay = self.runtime.replay_for_branch(analysis.id);
        let replay_branch_local = replay
            .frames
            .iter()
            .all(|frame| frame.branch_id == analysis.id);
        let replay_has_restore = replay
            .frames
            .iter()
            .any(|frame| frame.kind == ReplayEventKind::SnapshotRestored);
        self.verify_committed_financial_truth(&observed_work)?;
        let committed_values = self.committed_financial_values()?;
        let diagnostics_profile_observed = self.runtime.observe().diagnostics_profile();
        let summary = self.runtime.observe().diagnostics_summary(diagnostics_tier);
        let expected_producer = valuation;
        let expected_aspect = super::super::super::aspects::PRICE;
        if analysis_causes_before_capture.is_empty()
            || analysis_causes_before_capture != analysis_causes_after_restore
            || analysis_causes_before_capture.iter().any(|cause| {
                cause.consumer != risk
                    || cause.producer != expected_producer
                    || cause.aspect != expected_aspect
            })
        {
            return Err(SignalError::internal(
                "branch restore did not preserve the exact financial dependency causes",
            ));
        }

        Ok(FinancialBranchLifecycleEvidence {
            diagnostics_tier,
            diagnostics_profile_observed,
            active_nodes_observed: summary.active_node_count,
            analysis_causes_before_capture,
            analysis_causes_after_restore,
            main_pending_isolated,
            async_dependency_blocked,
            async_after_restore_blocked,
            async_after_settlement_admitted,
            replay_branch_local,
            replay_has_restore,
            final_snapshot: self.economic_snapshot.clone(),
            observed_work,
            committed_values,
            replay,
        })
    }

    fn install_definition_state(
        &mut self,
        base: &FinancialWorldDefinition,
        definition: FinancialWorldDefinition,
    ) {
        let base_snapshot = runtime_financial_snapshot(base);
        let snapshot = runtime_financial_snapshot(&definition);
        self.projection = FinancialSemanticProjection::initial(&base_snapshot).advance(&snapshot);
        self.economic_snapshot = snapshot;
        self.definition = definition;
    }

    fn settle_current_definition(&mut self) -> Result<(), SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        let program = FinancialEvaluationProgram::new(
            self.definition.clone(),
            self.projection.clone(),
            self.handles.clone(),
            self.ledger.clone(),
        );
        let evaluator = program.evaluator();
        let valuation_wave = self
            .definition
            .positions()
            .iter()
            .map(|position| self.handles.position(position.instrument).valuation)
            .collect::<Vec<_>>();
        let risk_wave = self
            .definition
            .positions()
            .iter()
            .map(|position| self.handles.position(position.instrument).risk)
            .collect::<Vec<_>>();
        let consumer_wave = self
            .handles
            .consumers
            .values()
            .map(|handle| handle.0)
            .collect::<Vec<_>>();
        for wave in [valuation_wave, risk_wave, consumer_wave] {
            let dirty = wave
                .into_iter()
                .filter(|node| {
                    self.runtime
                        .graph()
                        .get_state(*node)
                        .is_ok_and(|state| !matches!(state, NodeState::Clean))
                })
                .collect::<Vec<_>>();
            self.runtime.transaction(request_execution, &mut (), |tx| {
                for node in &dirty {
                    tx.read(*node, &evaluator)?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }

    fn settle_valuation_wave(&mut self) -> Result<(), SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        let program = FinancialEvaluationProgram::new(
            self.definition.clone(),
            self.projection.clone(),
            self.handles.clone(),
            self.ledger.clone(),
        );
        let evaluator = program.evaluator();
        let valuations = self
            .definition
            .positions()
            .iter()
            .map(|position| self.handles.position(position.instrument).valuation)
            .filter(|node| {
                self.runtime
                    .graph()
                    .get_state(*node)
                    .is_ok_and(|state| !matches!(state, NodeState::Clean))
            })
            .collect::<Vec<_>>();
        self.runtime.transaction(request_execution, &mut (), |tx| {
            for valuation in &valuations {
                tx.read(*valuation, &evaluator)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

fn dependency_not_ready(report: &crate::data::async_node::AsyncNodeRequestAdmissionReport) -> bool {
    report.classification().class() == AsyncNodeAdmissionClass::BlockedByCondition
        && report.classification().condition_block_class()
            == Some(AsyncNodeConditionBlockClass::DependencyNotReady)
        && report.resource_admission().is_none()
}

fn cause_fingerprints(
    causes: &[crate::data::proof::invalidation::binding::ResolvedDependencyCause],
) -> Vec<FinancialCauseFingerprint> {
    causes
        .iter()
        .map(|cause| FinancialCauseFingerprint {
            consumer: cause.binding_axes.consumer,
            dependency_revision: cause.binding_axes.dependency_revision.0,
            producer: cause.binding_axes.producer,
            aspect: cause.binding_axes.aspect,
            edge_scope: cause.binding_axes.edge_scope.clone(),
            cached_version: cause.binding_axes.cached_version,
            output_commit_ordinal: cause.binding_axes.output_commit_ordinal.0,
            committed_version: cause.binding_axes.committed_version,
            changed_scopes: cause.changed_scopes.as_slice().to_vec(),
        })
        .collect()
}
