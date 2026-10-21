//! Basis-bound admission of a prepared member by the required-set owner.
use super::super::WorthQueryOutputDemandRegistry;
use crate::domain_computation::primary_graph::application_contribution::OwnerPlan;
use crate::domain_computation::primary_graph::invariant_projection::PreparedCurrentInputs;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

/// No driver can mint this from a dependency name, level or readiness flag.
/// The pinned projection remains in owner completion while this token travels.
pub(in crate::domain_computation::primary_graph) struct PreparedRequiredInputs {
    runtime: u64,
    root: u64,
    owner: WorthQueryOutputDemandRegistry,
}
impl PreparedRequiredInputs {
    pub(in crate::domain_computation::primary_graph) fn accepts_basis(
        &self,
        basis: &PositionedRelationalSnapshot,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        Ok((self.runtime, self.root) == (basis.runtime_instance_id(), basis.root_id()))
    }
}
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit_prepared_inputs(
        &self,
        inputs: PreparedCurrentInputs<'_>,
    ) -> Result<PreparedRequiredInputs, WorthQueryOutputDemandDenial> {
        self.admit_current_outputs(inputs.native_basis(), inputs.consumed())
    }

    // Only this registry owner can interpret sealed current-output evidence.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn admit_current_outputs<
        'a,
    >(
        &self,
        (runtime, root): (u64, u64),
        consumed: impl Iterator<Item = &'a crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
    ) -> Result<PreparedRequiredInputs, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for consumed in consumed {
            if consumed.current_read_basis() != (runtime, root) {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSettlement,
                    "prepared upstream was read at another native basis",
                ));
            }
            let Some(key) = state
                .settlement_keys
                .get_exact_admitted(consumed.identity())?
            else {
                // A sealed current native read already names a committed output.
                // Recovery or row reclamation can leave it without an executable
                // delivery. Completion still owns its retained prerequisite claim.
                continue;
            };

            let Some(record) = state.records.get(key.as_ref()) else {
                return Err(foreign_denial());
            };

            if let Err(denied) = require_committed(record) {
                // Current native data can precede its managed Ready delivery.
                // Carry the exact funded read so the wave owner resumes that
                // checkpoint, then verifies again before dispatching compute.
                let requested = consumed.requested_read().map_err(requested_read_denial)?;
                return Err(denied.with_requested_output(requested));
            }
        }
        Ok(PreparedRequiredInputs {
            runtime,
            root,
            owner: self.clone(),
        })
    }
}

fn foreign_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ForeignSettlement,
        "required wave inputs do not belong to the selected basis",
    )
}

fn requested_read_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    use worth_relational::facade::mvcc::RelationalOperationInterruption;
    let kind = match stop {
        Stop::Execution(cause) => {
            crate::domain_computation::primary_graph::output_reuse::read_denial(cause).kind()
        }
        Stop::WorkExhausted { .. }
        | Stop::WorkExhaustedWithoutRequirement { .. }
        | Stop::WorkCounterOverflow => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        Stop::PreparationMemoryExhausted { .. }
        | Stop::PreparationMemoryCounterOverflow
        | Stop::RetainedCompanionCounterOverflow { .. }
        | Stop::RetainedCompanionCapacityExhausted { .. } => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
        Stop::Interrupted(event) => match event.interruption() {
            RelationalOperationInterruption::Cancelled => {
                WorthQueryOutputDemandDenialKind::Cancelled
            }
            RelationalOperationInterruption::TimedOut => WorthQueryOutputDemandDenialKind::TimedOut,
        },
        Stop::TopologyPending
        | Stop::SelectedSourceMismatch
        | Stop::SelectedPositionUnavailable { .. }
        | Stop::ForeignCell
        | Stop::RegistrationChanged => WorthQueryOutputDemandDenialKind::ForeignSettlement,
    };
    WorthQueryOutputDemandDenial::new(kind, "required wave requested-read custody refused")
}

/// The sole required-wave dispatch input. Its selected basis stays borrowed
/// until every compute and owner completion has finished or been discarded.
pub(in crate::domain_computation::primary_graph) struct AdmittedRequiredWave<'basis, Key, Payload> {
    basis: &'basis PositionedRelationalSnapshot,
    keys: worth_proof::CanonicalUniqueVec<Key>,
    payloads: Vec<Payload>,
}
impl<'basis, Key, Payload> AdmittedRequiredWave<'basis, Key, Payload> {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        &'basis PositionedRelationalSnapshot,
        worth_proof::CanonicalUniqueVec<Key>,
        Vec<Payload>,
    ) {
        let Self {
            basis,
            keys,
            payloads,
        } = self;
        (basis, keys, payloads)
    }
}
type PreparedWave<'basis, 'owner, Key, Value, Stop> =
    AdmittedRequiredWave<'basis, Key, OwnerPlan<'owner, Value, Stop>>;
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit_prepared_wave<
        'basis,
        'owner,
        Key,
        Value: 'owner,
        Stop: 'owner,
    >(
        &self,
        basis: &'basis PositionedRelationalSnapshot,
        keys: worth_proof::CanonicalUniqueVec<Key>,
        plans: Vec<OwnerPlan<'owner, Value, Stop>>,
    ) -> Result<PreparedWave<'basis, 'owner, Key, Value, Stop>, WorthQueryOutputDemandDenial> {
        if keys.as_slice().len() != plans.len() {
            return Err(foreign_denial());
        }
        for plan in &plans {
            match plan.prepared_inputs() {
                Some(inputs)
                    if {
                        std::sync::Arc::ptr_eq(&self.state, &inputs.owner.state)
                            && inputs.accepts_basis(basis)?
                    } => {}
                Some(_) => {
                    return Err(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSettlement,
                        "prepared plan belongs to another owner or basis",
                    ))
                }
                // Stable publication owns its verified cutoff; a refused
                // preparation owns a stop. Neither contains worker compute.
                None if !plan.has_compute() => {}
                None => return Err(foreign_denial()),
            }
        }
        Ok(AdmittedRequiredWave {
            basis,
            keys,
            payloads: plans,
        })
    }
}

fn require_committed(
    record: &super::super::DemandRecord,
) -> Result<(), WorthQueryOutputDemandDenial> {
    if record.has_cached_ready() {
        return Ok(());
    }
    Err(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::SchedulingDeferred,
        "prepared member awaits its committed upstream delivery",
    ))
}
#[cfg(test)]
mod tests;
