mod consumed;
mod facts;
mod settlements;
use facts::require_current_facts;
use facts::require_witnessed_output;

use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::output_lineage::RetainedOutputCurrentnessRead;
use crate::domain_computation::primary_graph::output_reuse::first_moved_fact;
use crate::domain_computation::primary_graph::output_reuse::RetainedFactStop;
use crate::domain_computation::primary_graph::provider::FactlessCurrentness;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandSettlement;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Checks retained output settlements, including outputs readmitted from an
    /// accepted checkpoint that intentionally have no fresh commit receipt.
    /// The phase witnesses request custody; lower Relational accounting follows in part two.
    pub fn require_current_output_settlements<'settlement>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        settlements: impl IntoIterator<Item = &'settlement WorthQueryOutputDemandSettlement>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|cause| {
                denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    cause.into(),
                ))
            })?;

        runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |relational| {
                self.require_current_settlements_in(execution, relational, settlements)
            })
    }

    /// Checks a set of retained producer receipts against this exact product
    /// observation, including their observed source facts.
    /// The phase witnesses request custody; lower Relational accounting follows in part two.
    pub fn require_current_output_receipts<'receipt>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        receipts: impl IntoIterator<Item = &'receipt WorthQueryApplicationCommitReceipt>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|cause| {
                denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    cause.into(),
                ))
            })?;

        runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |relational| {
                for receipt in receipts {
                    if receipt.runtime_authority() != runtime.runtime.authority_identity()
                        || receipt.principal_scope().binding_identity()
                            != &runtime.installed_schema.binding_identity()
                    {
                        return Err(denial(WorthQueryOutputDemandDenialKind::ForeignSettlement));
                    }
                    let lineage = runtime
                        .primary_provider
                        .graph
                        .output_lineage
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let read = lineage_read(|| {
                        lineage.source_facts_for_receipt(
                            runtime.runtime.authority_identity().as_u64(),
                            &runtime.installed_schema.binding_identity(),
                            self.product().observation(),
                            receipt,
                            &execution,
                        )
                    })?;
                    drop(lineage);
                    let Some(read) = read else {
                        require_own_publication(Some(receipt), self.product().observation())?;
                        continue;
                    };
                    self.require_current_read(&execution, relational, &read, Some(receipt))?;
                }
                Ok(())
            })
    }

    /// Checks the output witness and exact selected-image native marks. Missing
    /// or unqualified marks use full source verification and re-establishment;
    /// neither path borrows authority from content equality or another image.
    pub(in crate::domain_computation::primary_graph) fn require_current_read(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        read: &RetainedOutputCurrentnessRead,
        receipt: Option<&WorthQueryApplicationCommitReceipt>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(facts) = read.facts.as_ref().filter(|facts| !facts.is_empty()) else {
            // No computation comparison can replace the original factless
            // publication decision, including a rebase that could not compare.
            require_own_publication(receipt, self.product().observation())?;
            return self.require_current_consumed(execution, relational, read);
        };
        let owner = &self
            .application()
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner;
        let snapshot = self.application_basis().snapshot_handle();
        require_witnessed_output(relational, snapshot, read.native_output_witness.as_ref())?;
        // Exact lineage selection and the output witness precede the native mark.
        // Missing/dirty/discontinuous rows retain the original funded comparison.
        if let Some(selected) = currentness_basis(relational, snapshot)? {
            use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;
            currentness_live(execution)?;
            let current = crate::domain_computation::primary_graph::output_reuse::owner_read(
                owner.currentness(&selected, &read.identity),
                "settled source marks unavailable",
            )?;
            currentness_live(execution)?;
            if matches!(current, SourceSettlementCurrentness::Clean) {
                owner.carry_read_basis(relational, snapshot, &selected, std::slice::from_ref(&read.identity))
                    .map_err(|stop| crate::domain_computation::primary_graph::output_reuse::verification_denial(
                        crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_verification_stop(stop),
                        "settled output basis carriage",
                    ))?;
                currentness_live(execution)?;
                return Ok(());
            }
        }
        require_current_facts(execution, relational, snapshot, facts).map_err(
            |error| match read.verification_requirement {
                Some(crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason::NativeFactRevisionUnavailable(ordinal)) => WorthQueryOutputDemandDenial::new(
                    error.kind(),
                    format!("{}; native revision unavailable for: {}", error.subject(),
                        facts.get(ordinal).map_or_else(|| "unavailable retained fact".into(), |fact| fact.locator_identity())),
                ),
                Some(crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason::IndexedSelectionFactDenied(ordinal, denial)) => WorthQueryOutputDemandDenial::new(
                    error.kind(),
                    format!("{}; indexed selection reobservation denied: {denial:?}; fact: {}", error.subject(),
                        facts.get(ordinal).map_or_else(|| "unavailable retained fact".into(), |fact| fact.locator_identity())),
                ),
                Some(reason) => WorthQueryOutputDemandDenial::new(
                    error.kind(),
                    format!(
                        "{}; output requires full verification: {reason:?}",
                        error.subject()
                    ),
                ),
                None => error,
            },
        )?;
        self.require_current_consumed(execution, relational, read)?;
        if let Ok(selected) = relational.read_truth().positioned_snapshot(snapshot) {
            let witness = read.native_output_witness.as_ref();
            if let Some(witness) = witness
                .and_then(|witness| witness.get())
                .filter(|_| read.consumed_nothing)
            {
                // A restored output gets its row from this comparison.
                let _ = owner.establish_verified_root(&selected, &read.identity, facts, witness);
            }
            // A row that cannot be re-established keeps requiring verification.
            let _ =
                owner.reestablish_verified(relational, snapshot, &selected, &read.identity, facts);
        }
        Ok(())
    }
}

/// Every visited native prefix charges the same admission, including refusal.
fn lineage_read(
    read: impl FnOnce() -> Result<
        Option<RetainedOutputCurrentnessRead>,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    >,
) -> Result<Option<RetainedOutputCurrentnessRead>, WorthQueryOutputDemandDenial> {
    read().map_err(|stop| {
        crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
            crate::domain_computation::primary_graph::output_reuse::preflight_stop(stop),
            "native lineage read unavailable",
        )
    })
}

/// A settlement that retains no fact has nothing to compare: the commit of a
/// failed rebase answers for itself. One whose rebase could not compare a read
/// it asked about is denied, never superseded: a recompute's own commit would
/// meet the same comparison. A rebase its meter stopped counted the reads it
/// left as moved, so that commit is superseded and refreshes. A settlement
/// with neither facts nor such a commit is superseded.
fn require_own_publication(
    receipt: Option<&WorthQueryApplicationCommitReceipt>,
    observation: &worth_runtime_world::facade::ProductBranchObservation,
) -> Result<(), WorthQueryOutputDemandDenial> {
    match receipt.and_then(|receipt| receipt.currentness_without_facts_at(observation)) {
        Some(FactlessCurrentness::Current) => Ok(()),
        Some(FactlessCurrentness::Undecidable) => Err(denial(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        )),
        Some(FactlessCurrentness::Superseded) | None => {
            Err(denial(WorthQueryOutputDemandDenialKind::Superseded))
        }
    }
}

fn denial(kind: WorthQueryOutputDemandDenialKind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        kind,
        "settled output is not current at the selected observation",
    )
}

fn currentness_live(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    execution.allocation_policy().check_live().map_err(|cause| {
        use worth_execution::ExecutionAllocationDenialClass as Class;
        use worth_execution::MapKernelStop;
        use worth_execution::WorkCeilingDenial;
        let kind = match cause.classify() {
            Class::Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)) => {
                WorthQueryOutputDemandDenialKind::Cancelled
            }
            Class::Request(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed)) => {
                WorthQueryOutputDemandDenialKind::TimedOut
            }
            Class::Request(
                WorkCeilingDenial::Admission(_)
                | WorkCeilingDenial::Panicked
                | WorkCeilingDenial::Stopped(
                    MapKernelStop::WorkCounterOverflow
                    | MapKernelStop::AccountingUnavailable
                    | MapKernelStop::WorkCeiling
                    | MapKernelStop::NestedStopped,
                ),
            )
            | Class::Physical(_) => WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        };
        WorthQueryOutputDemandDenial::new(kind, "settled output request control")
    })
}

fn currentness_basis(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<
    Option<worth_relational::facade::runtime::PositionedRelationalSnapshot>,
    WorthQueryOutputDemandDenial,
> {
    Ok(relational.read_truth().positioned_snapshot(snapshot).ok())
}
