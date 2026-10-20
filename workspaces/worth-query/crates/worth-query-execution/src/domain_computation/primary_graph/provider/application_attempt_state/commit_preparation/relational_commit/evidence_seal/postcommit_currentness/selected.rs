//! Selected native revision resolution against a distinct normalized body.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    application_checkpoint::facts::NativeFactPatchBuilder,
    output_lineage::invalidation::{InvalidationEditAdmission, PostconditionCarry},
};
use worth_execution::{
    ChargedBytes, ExecutionArray, ExecutionArrayBuilder, ExecutionMemoryReservation,
};
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

type Endpoints =
    crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints;
pub(super) struct SelectedRebase {
    facts: FactPayload,
    posting_basis: Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::PostingRebaseBasis,
    >,
    ordinals: im::OrdSet<usize>,
    moved: Arc<[usize]>,
    endpoints: Option<ExecutionArray<(usize, Option<Endpoints>)>>,
    payload_bytes: u64,
    actions: ExecutionArrayBuilder<resolution::PreparedFactRebase>,
    superseded: ExecutionArrayBuilder<Option<usize>>,
    patch: NativeFactPatchBuilder,
    _selection_memory: Option<ExecutionMemoryReservation>,
    _resolution_memory: Option<ExecutionMemoryReservation>,
}
impl SelectedRebase {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare(
        facts: FactPayload,
        moved: Arc<[usize]>,
        carry: PostconditionCarry,
        ordinals: im::OrdSet<usize>,
        selection_memory: Option<ExecutionMemoryReservation>,
        posting_basis: crate::domain_computation::primary_graph::output_lineage::invalidation::PostingRebaseBasis,
        admission: &mut InvalidationEditAdmission,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        control.check_live()?;
        let payload_bytes = carry
            .unchanged_payload_bytes(&ordinals, admission)
            .map_err(PreparedRebaseDenial::Admission)?;
        let suffix = facts.slice(carry.prefix()..facts.len());
        let base = carry
            .baseline()
            .slice(0..carry.prefix())
            .append_completed(suffix, control)?;
        let mut prepared = Self::prepare_body(
            facts,
            moved,
            base,
            payload_bytes,
            ordinals,
            selection_memory,
            control,
        )?;
        prepared.posting_basis = Some(posting_basis);
        Ok(prepared)
    }
    pub(super) fn prepare_full(
        facts: FactPayload,
        moved: Arc<[usize]>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        control.check_live()?;
        let quote = crate::domain_computation::primary_graph::output_lineage::invalidation::insertion_sequence_bytes::<usize, ()>(facts.len()).ok_or(StoreDenial::Representability)?;
        let memory = control.policy().reserve_memory(quote);
        control.check_live()?;
        let memory = memory?;
        let mut ordinals = im::OrdSet::new();
        for ordinal in 0..facts.len() {
            control.check_live()?;
            ordinals.insert(ordinal);
        }
        Self::prepare_body(facts.clone(), moved, facts, 0, ordinals, memory, control)
    }
    #[allow(clippy::too_many_arguments)]
    fn prepare_body(
        facts: FactPayload,
        moved: Arc<[usize]>,
        base: FactPayload,
        payload_bytes: u64,
        ordinals: im::OrdSet<usize>,
        selection_memory: Option<ExecutionMemoryReservation>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        let patch = base.prepare_native_patch(&ordinals, control)?;
        let mut bytes = 0u64;
        for &ordinal in &ordinals {
            control.check_live()?;
            fact_verification_observation::rebase_preparation();
            bytes = bytes
                .checked_add(
                    facts[ordinal]
                        .additional_charged_bytes()
                        .checked_mul(2)
                        .ok_or(StoreDenial::Representability)?,
                )
                .ok_or(StoreDenial::Representability)?;
        }
        let memory = control.policy().reserve_memory(bytes);
        control.check_live()?;
        let memory = memory?;
        let mut endpoints = ExecutionArrayBuilder::allocate(ordinals.len(), control.policy())?;
        for &ordinal in &ordinals {
            control.check_live()?;
            fact_verification_observation::rebase_preparation();
            endpoints.push((
                ordinal,
                adjacency::prepare_endpoints(&facts[ordinal], control)?,
            ))?;
        }
        let actions = ExecutionArrayBuilder::allocate(ordinals.len(), control.policy())?;
        let superseded = ExecutionArrayBuilder::allocate(ordinals.len(), control.policy())?;
        Ok(Self {
            facts,
            posting_basis: None,
            ordinals,
            moved,
            endpoints: Some(endpoints.seal()?),
            payload_bytes,
            actions,
            superseded,
            patch,
            _selection_memory: selection_memory,
            _resolution_memory: memory,
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn rebase(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        retired: &BTreeSet<EntityId>,
        producer: bool,
        maximum: Option<usize>,
        admission: &mut InvalidationEditAdmission,
        control: StorageControl<'_, '_>,
    ) -> RebasedSourceFacts {
        let mut width = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: maximum
                    .map(|maximum| u64::try_from(maximum).unwrap_or(u64::MAX)),
                maximum_preparation_bytes: Some(0),
            },
        );
        let mut failure = None;
        let mut own = OwnEffectOnReads::NONE_ASKED;
        if !self.moved.is_empty() {
            own = own.join(OwnEffect::Moved);
        }
        let endpoints = self
            .endpoints
            .take()
            .expect("prepared selected endpoints")
            .into_iter();
        for (ordinal, endpoints) in endpoints {
            fact_verification_observation::rebase_resolution();
            let fact = &self.facts[ordinal];
            if failure.is_some() && !resolution::reads_source(fact) {
                continue;
            }
            let prepared = resolution::PreparedFactRebase::prepare(
                runtime, execution, snapshot, fact, retired, producer, admission, &mut width,
                endpoints,
            );
            match prepared {
                Err(stop) => {
                    return RebasedSourceFacts::VerificationRequired {
                        reason: failure.unwrap_or(RebaseVerificationReason::AdmissionDenied(stop)),
                        own_effect: self.unanswered(ordinal, producer, own, OwnEffect::Moved),
                    }
                }
                Ok(Err(RebaseVerificationReason::Execution(cause))) => {
                    return RebasedSourceFacts::VerificationRequired {
                        reason: RebaseVerificationReason::Execution(cause),
                        own_effect: self.unanswered(ordinal, producer, own, OwnEffect::Undecidable),
                    }
                }
                Ok(Err(reason)) => {
                    let reason = match reason {
                        RebaseVerificationReason::NativeRevisionUnavailable => {
                            RebaseVerificationReason::NativeFactRevisionUnavailable(ordinal)
                        }
                        RebaseVerificationReason::IndexedSelectionDenied(denial) => {
                            RebaseVerificationReason::IndexedSelectionFactDenied(ordinal, denial)
                        }
                        reason => reason,
                    };
                    if producer && resolution::reads_source(fact) {
                        own = own.join(OwnEffect::Undecidable);
                    }
                    failure.get_or_insert(reason);
                }
                Ok(Ok(action)) => {
                    if matches!(action, resolution::PreparedFactRebase::KeepSuperseded) {
                        own = own.join(OwnEffect::Moved);
                    }
                    if failure.is_none()
                        && let Err(cause) = self.actions.push(action)
                    {
                        // A stopped funded action slot can precede later source
                        // questions. Never classify those unasked reads Current.
                        let own =
                            self.unanswered(ordinal + 1, producer, own, OwnEffect::Undecidable);
                        return rebase_walk::storage_stopped(cause, own);
                    }
                }
            }
        }
        if let Some(reason) = failure {
            return RebasedSourceFacts::VerificationRequired {
                reason,
                own_effect: own,
            };
        }
        let actions = match self.actions.seal() {
            Ok(actions) => actions,
            Err(cause) => return rebase_walk::storage_stopped(cause, own),
        };
        let mut superseded_count = 0usize;
        for (&ordinal, action) in self.ordinals.iter().zip(actions) {
            let moved = matches!(action, resolution::PreparedFactRebase::KeepSuperseded)
                || self.moved.binary_search(&ordinal).is_ok();
            superseded_count += usize::from(moved);
            if let Err(cause) = self.superseded.push(moved.then_some(ordinal)) {
                return rebase_walk::storage_stopped(cause, own);
            }
            if let Err(cause) = control.check_live() {
                return storage_stop(cause, own);
            }
            // Temporary deep clone is covered by pre-effect resolution memory;
            // patch replacement separately holds its surviving deep grant.
            let replacement = action.apply(self.facts[ordinal].clone());
            let added = match crate::domain_computation::primary_graph::output_lineage::invalidation::retained_fact_payload_bytes(&replacement, admission) {
                Ok(bytes) => bytes,
                Err(stop) => return RebasedSourceFacts::VerificationRequired { reason: RebaseVerificationReason::AdmissionDenied(stop), own_effect: own },
            };
            self.payload_bytes = match self.payload_bytes.checked_add(added) {
                Some(bytes) => bytes,
                None => return storage_stop(StoreDenial::Representability, own),
            };
            if let Err(cause) = self.patch.replace(ordinal, &replacement, control) {
                return storage_stop(cause, own);
            }
        }
        let facts = match self.patch.seal(control) {
            Ok(facts) => facts,
            Err(cause) => return storage_stop(cause, own),
        };
        let ordinals = match self.superseded.seal() {
            Ok(ordinals) => Arc::new(ordinals),
            Err(cause) => return rebase_walk::storage_stopped(cause, own),
        };
        let posting_continuation = match self.posting_basis {
            None => None,
            Some(basis) => {
                let after = match runtime.read_truth().positioned_snapshot_admitted(
                    snapshot,
                    |work, bytes| {
                        admission.charge_external_work(work)?;
                        admission.admit_read_scratch(bytes)
                    },
                ) {
                    Ok(after) => after,
                    Err(stop) => {
                        if let Err(interrupted) = control.check_live() {
                            return storage_stop(interrupted, own);
                        }
                        use worth_relational::facade::runtime::RelationalSnapshotPositionAdmissionStop as Stop;
                        let reason = match stop {
                            Stop::Admission(stop) => {
                                RebaseVerificationReason::AdmissionDenied(stop)
                            }
                            Stop::AccountingOverflow => RebaseVerificationReason::AdmissionDenied(
                                CompanionPreflightStop::WorkCounterOverflow,
                            ),
                            Stop::Position(_) => {
                                RebaseVerificationReason::NativeRevisionUnavailable
                            }
                        };
                        return RebasedSourceFacts::VerificationRequired {
                            reason,
                            own_effect: own,
                        };
                    }
                };
                match basis.seal(facts.clone(), after, self._selection_memory, control) {
                    Ok(completed) => Some(completed),
                    Err(stop) => return storage_stop(stop, own),
                }
            }
        };
        RebasedSourceFacts::Cached {
            posting_continuation,
            facts,
            payload_bytes: self.payload_bytes,
            ordinals,
            superseded_count,
        }
    }
    fn unanswered(
        &self,
        first: usize,
        producer: bool,
        own: OwnEffectOnReads,
        effect: OwnEffect,
    ) -> OwnEffectOnReads {
        if producer
            && self.ordinals.range(first..).any(|ordinal| {
                fact_verification_observation::rebase_unanswered();
                resolution::reads_source(&self.facts[*ordinal])
            })
        {
            own.join(effect)
        } else {
            own
        }
    }
}
fn storage_stop(cause: StoreDenial, own_effect: OwnEffectOnReads) -> RebasedSourceFacts {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Stop;
    let reason = match cause {
        StoreDenial::Allocation(cause) => return rebase_walk::storage_stopped(cause, own_effect),
        StoreDenial::RequestInterruption(stop) => RebaseVerificationReason::Execution(match stop {
            Stop::Cancelled => worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
            Stop::DeadlineExceeded => worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed,
        }),
        _ => RebaseVerificationReason::NativeRevisionUnavailable,
    };
    RebasedSourceFacts::VerificationRequired { reason, own_effect }
}
