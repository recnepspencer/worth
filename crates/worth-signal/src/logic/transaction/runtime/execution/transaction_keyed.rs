use std::sync::Mutex;

use crate::clock::RuntimeInstant;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::output::ComputationKey;
use crate::data::output::KeyedComputation;
use crate::data::proof::PartitionScopeSet;
use crate::data::reuse::PersistentCorrespondenceEvidence;
use crate::diagnostics::ExecutionFailurePhase;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::{EvaluationRequestMode, IntoEvaluationOutput};
use crate::logic::planner::precompute::callback::LegacyPrecompute;
use crate::logic::prepared::{
    PreparedEvaluationOrigin, PreparedKeyedContext, PreparedMemoDecision,
};

use super::super::transaction::SignalTransaction;
use super::shared::{
    absorb_execution_report_telemetry, execute_targets_with_prepared_runtime_config_detailed,
};

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub fn evaluate_keyed<F, O>(
        &mut self,
        node: NodeId,
        computation: &KeyedComputation,
        evaluator: &F,
    ) -> Result<(), SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.evaluate_keyed_with_mode(node, computation, evaluator, EvaluationRequestMode::Default)
    }

    pub fn evaluate_keyed_with_mode<F, O>(
        &mut self,
        node: NodeId,
        computation: &KeyedComputation,
        evaluator: &F,
        request_mode: EvaluationRequestMode,
    ) -> Result<(), SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.evaluate_keyed_internal(
            node,
            computation,
            evaluator,
            request_mode,
            KeyedReuseRequest::None,
        )
    }

    pub fn evaluate_keyed_cross_identity<F, O>(
        &mut self,
        node: NodeId,
        computation: &KeyedComputation,
        evaluator: &F,
        source_key: ComputationKey,
        correspondence: PersistentCorrespondenceEvidence,
    ) -> Result<(), SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.evaluate_keyed_internal(
            node,
            computation,
            evaluator,
            EvaluationRequestMode::Default,
            KeyedReuseRequest::CrossIdentity {
                source_key,
                correspondence,
            },
        )
    }

    pub fn evaluate_keyed_partial_splice<F, O>(
        &mut self,
        node: NodeId,
        computation: &KeyedComputation,
        evaluator: &F,
        composition_regions: PartitionScopeSet,
    ) -> Result<(), SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.evaluate_keyed_internal(
            node,
            computation,
            evaluator,
            EvaluationRequestMode::Default,
            KeyedReuseRequest::PartialSplice {
                composition_regions,
            },
        )
    }

    fn evaluate_keyed_internal<F, O>(
        &mut self,
        node: NodeId,
        computation: &KeyedComputation,
        evaluator: &F,
        request_mode: EvaluationRequestMode,
        reuse_request: KeyedReuseRequest,
    ) -> Result<(), SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.with_telemetry(|telemetry| telemetry.invalidation.keyed_evaluation_count += 1);
        self.stage_evaluate_candidate_batch(std::slice::from_ref(&node))?;
        self.rollback_packets
            .capture_config_baseline_if_needed(self.config);
        self.admit_temporal_wakes_for_nodes(&[node])?;
        self.promote_due_temporal_wakes_ready()?;
        let temporal_lowering = self.temporal_lowering_context_for_nodes(&[node]);
        let family_id = self.config.key_registry.intern_family(&computation.family);
        let key_id = self.config.key_registry.intern_key(&computation.key);
        let memo_key_id = computation
            .memo_key
            .as_ref()
            .map(|memo_key| self.config.key_registry.intern_memo_key(memo_key));
        let resolve_memo_key_id = || {
            memo_key_id.ok_or_else(|| {
                SignalError::internal("memoized keyed execution is missing an interned memo key id")
            })
        };
        let base_keyed_context = PreparedKeyedContext {
            family: Some(computation.family.clone()),
            key: Some(computation.key.clone()),
            memo_key: computation.memo_key.clone(),
            memoized_origin: crate::data::output::MemoizedResultOrigin::DirectCompute,
            persistent_correspondence: reuse_request.persistent_correspondence().cloned(),
            composition_regions: reuse_request
                .composition_regions()
                .cloned()
                .unwrap_or_default(),
        };
        if let Some(memo_key) = computation.memo_key.as_ref() {
            let cached = self
                .scratch
                .staged_memo_writes
                .get(&(family_id, key_id, resolve_memo_key_id()?))
                .cloned()
                .or_else(|| {
                    self.config.lookup_memoized_result(
                        &computation.family,
                        &computation.key,
                        memo_key,
                    )
                });
            let cached = cached.or_else(|| {
                reuse_request.lookup_cached_result(
                    self.config,
                    &self.scratch,
                    &computation.family,
                    memo_key,
                )
            });
            if let Some(cached) = cached {
                self.with_telemetry(|telemetry| telemetry.evaluation.memoization_hits += 1);
                let cached_result = cached;
                self.scratch.staged_memo_writes.insert(
                    (family_id, key_id, resolve_memo_key_id()?),
                    cached_result.clone(),
                );
                let execution_start = RuntimeInstant::now();
                let report = match execute_targets_with_prepared_runtime_config_detailed(
                    self.graph,
                    self.config,
                    temporal_lowering.clone(),
                    &[node],
                    request_mode,
                    &LegacyPrecompute::new(
                        |_current, _view: &crate::logic::prepared::ExecutionReadView<'_>| {
                            Ok(crate::logic::prepared::PreparedEvaluation::from_result(
                                cached_result.clone(),
                            )
                            .with_origin(reuse_request.prepared_origin())
                            .with_memo_decision(PreparedMemoDecision::Hit)
                            .with_keyed(PreparedKeyedContext {
                                memoized_origin:
                                    crate::data::output::MemoizedResultOrigin::MemoizedFromCache,
                                ..base_keyed_context.clone()
                            }))
                        },
                    ),
                    self.execution,
                ) {
                    Ok(report) => report,
                    Err(failure) => {
                        let err = failure.error;
                        self.record_failure_from_error(
                            ExecutionFailurePhase::Apply,
                            &err,
                            Some(failure.plan_summary),
                        );
                        return Err(err);
                    }
                };
                self.execution_state
                    .record_report(&report, execution_start.elapsed().as_nanos());
                self.scratch.temporal.absorb_report(&report);
                self.lower_observation_classifications_from_report(&report)?;
                self.with_telemetry(|telemetry| {
                    absorb_execution_report_telemetry(telemetry, &report)
                });
                self.retire_consumed_temporal_wakes_from_report(&report)?;
                return self.apply_result(Ok(()));
            }
            self.with_telemetry(|telemetry| telemetry.evaluation.memoization_misses += 1);
        }

        let last_result = Mutex::new(None);
        let execution_start = RuntimeInstant::now();
        let result = match execute_targets_with_prepared_runtime_config_detailed(
            self.graph,
            self.config,
            temporal_lowering,
            &[node],
            request_mode,
            &LegacyPrecompute::new(
                |current, view: &crate::logic::prepared::ExecutionReadView<'_>| {
                    let mut ctx = EvaluationContext::new(view.graph(), current, &*self.runtime_ctx);
                    let output = evaluator(&mut ctx)?;
                    let prepared = ctx
                        .into_prepared(output)
                        .with_origin(reuse_request.compute_origin())
                        .with_memo_decision(PreparedMemoDecision::Miss)
                        .with_keyed(base_keyed_context.clone());
                    if current == node {
                        let mut guard = last_result
                            .lock()
                            .map_err(|_| SignalError::internal("memo capture mutex poisoned"))?;
                        *guard = Some(prepared.result.clone());
                    }
                    Ok(prepared)
                },
            ),
            self.execution,
        ) {
            Ok(report) => Ok(report),
            Err(failure) => {
                let err = failure.error;
                self.record_failure_from_error(
                    ExecutionFailurePhase::Apply,
                    &err,
                    Some(failure.plan_summary),
                );
                Err(err)
            }
        };
        let result = match result {
            Ok(report) => {
                self.execution_state
                    .record_report(&report, execution_start.elapsed().as_nanos());
                self.scratch.temporal.absorb_report(&report);
                self.lower_observation_classifications_from_report(&report)?;
                self.with_telemetry(|telemetry| {
                    absorb_execution_report_telemetry(telemetry, &report)
                });
                self.retire_consumed_temporal_wakes_from_report(&report)?;
                self.apply_result(Ok(()))
            }
            Err(err) => self.apply_result(Err(err)),
        };
        if result.is_ok() {
            if let Ok(mut guard) = last_result.lock() {
                if let Some(last_result) = guard.take() {
                    self.scratch
                        .staged_memo_writes
                        .insert((family_id, key_id, resolve_memo_key_id()?), last_result);
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
enum KeyedReuseRequest {
    None,
    CrossIdentity {
        source_key: ComputationKey,
        correspondence: PersistentCorrespondenceEvidence,
    },
    PartialSplice {
        composition_regions: PartitionScopeSet,
    },
}

impl KeyedReuseRequest {
    fn prepared_origin(&self) -> PreparedEvaluationOrigin {
        match self {
            Self::None => PreparedEvaluationOrigin::MemoizedReuse,
            Self::CrossIdentity { .. } => PreparedEvaluationOrigin::CrossIdentityPersistentReuse,
            Self::PartialSplice { .. } => PreparedEvaluationOrigin::PartialArtifactSplice,
        }
    }

    fn compute_origin(&self) -> PreparedEvaluationOrigin {
        match self {
            Self::PartialSplice {
                composition_regions,
            } if !composition_regions.is_empty() => PreparedEvaluationOrigin::PartialArtifactSplice,
            _ => PreparedEvaluationOrigin::DirectPrecompute,
        }
    }

    fn persistent_correspondence(&self) -> Option<&PersistentCorrespondenceEvidence> {
        match self {
            Self::CrossIdentity { correspondence, .. } => Some(correspondence),
            _ => None,
        }
    }

    fn composition_regions(&self) -> Option<&PartitionScopeSet> {
        match self {
            Self::PartialSplice {
                composition_regions,
            } => Some(composition_regions),
            _ => None,
        }
    }

    fn lookup_cached_result<T: Copy + Ord, D, I, E>(
        &self,
        config: &super::super::config::SignalRuntimeConfig<T>,
        scratch: &crate::logic::transaction::runtime::transaction::TransactionScratch<D, I, E>,
        family: &crate::data::output::ComputationFamily,
        memo_key: &crate::data::output::StructuralMemoKey,
    ) -> Option<crate::data::output::NodeEvaluationResult>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
    {
        match self {
            Self::CrossIdentity { source_key, .. } => {
                let family_id = config.key_registry.family_lookup.get(family).copied()?;
                let source_key_id = config.key_registry.key_lookup.get(source_key).copied()?;
                let memo_key_id = config.key_registry.memo_key_lookup.get(memo_key).copied()?;
                scratch
                    .staged_memo_writes
                    .get(&(family_id, source_key_id, memo_key_id))
                    .cloned()
                    .or_else(|| config.lookup_memoized_result(family, source_key, memo_key))
            }
            _ => None,
        }
    }
}
