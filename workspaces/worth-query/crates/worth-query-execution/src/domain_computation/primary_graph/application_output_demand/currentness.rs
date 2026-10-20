mod consumed;
mod facts;
mod settlements;
use facts::{require_current_facts, require_witnessed_output};

use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::{
    output_lineage::{invalidation::InvalidationEditAdmission, RetainedOutputCurrentnessRead},
    output_reuse::{first_moved_fact, RetainedFactStop},
    provider::FactlessCurrentness,
    WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandSettlement,
    WorthQuerySelectedProductOperation,
};

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Checks retained output settlements, including outputs readmitted from an
    /// accepted checkpoint that intentionally have no fresh commit receipt.
    /// The phase witnesses request custody; lower Relational accounting follows in part two.
    pub fn require_current_output_settlements<'settlement>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        settlements: impl IntoIterator<Item = &'settlement WorthQueryOutputDemandSettlement>,
        maximum_work: Option<usize>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|cause| {
                denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    cause.into(),
                ))
            })?;
        let mut admission = self.request_admission(maximum_work);
        runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |relational| {
                self.require_current_settlements_in(
                    execution,
                    relational,
                    settlements,
                    &mut admission,
                )
            })
    }

    /// Checks a set of retained producer receipts against this exact product
    /// observation, including their observed source facts.
    /// The phase witnesses request custody; lower Relational accounting follows in part two.
    pub fn require_current_output_receipts<'receipt>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        receipts: impl IntoIterator<Item = &'receipt WorthQueryApplicationCommitReceipt>,
        maximum_work: Option<usize>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|cause| {
                denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    cause.into(),
                ))
            })?;
        let mut admission = self.request_admission(maximum_work);
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
                    let read = lineage_read(&mut admission, |admission| {
                        lineage.source_facts_for_receipt(
                            runtime.runtime.authority_identity().as_u64(),
                            &runtime.installed_schema.binding_identity(),
                            self.product().observation(),
                            receipt,
                            &execution,
                            admission,
                        )
                    })?;
                    drop(lineage);
                    let Some(read) = read else {
                        require_own_publication(Some(receipt), self.product().observation())?;
                        continue;
                    };
                    self.require_current_read(
                        &execution,
                        relational,
                        &read,
                        Some(receipt),
                        &mut admission,
                    )?;
                }
                Ok(())
            })
    }

    /// The request's meter, within the caller's declared maximum.
    pub(in crate::domain_computation::primary_graph) fn request_admission(
        &self,
        maximum_work: Option<usize>,
    ) -> InvalidationEditAdmission {
        self.application()
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .edit_admission_with_work_limit(maximum_work)
    }

    /// Compares the original evidence and consumed lineage at this observation.
    /// Only nonempty comparable facts may re-establish the row's marks. That
    /// upkeep uses this request's meter and changes no currentness answer.
    pub(in crate::domain_computation::primary_graph) fn require_current_read(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        read: &RetainedOutputCurrentnessRead,
        receipt: Option<&WorthQueryApplicationCommitReceipt>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(facts) = read.facts.as_ref().filter(|facts| !facts.is_empty()) else {
            // No computation comparison can replace the original factless
            // publication decision, including a rebase that could not compare.
            require_own_publication(receipt, self.product().observation())?;
            return self.require_current_consumed(execution, relational, read, admission);
        };
        let owner = &self
            .application()
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner;
        let snapshot = self.application_basis().snapshot_handle();
        require_witnessed_output(
            relational,
            snapshot,
            read.native_output_witness.as_ref(),
            admission,
        )?;
        require_current_facts(execution, relational, snapshot, facts, admission).map_err(
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
        self.require_current_consumed(execution, relational, read, admission)?;
        if let Ok(selected) = relational.read_truth().positioned_snapshot(snapshot) {
            let witness = read.native_output_witness.as_ref();
            if let Some(witness) = witness
                .and_then(|witness| witness.get())
                .filter(|_| read.consumed_nothing)
            {
                // A restored output gets its row from this comparison.
                let _ = owner.establish_verified_root(
                    &selected,
                    &read.identity,
                    facts,
                    witness,
                    admission,
                );
            }
            // A row that cannot be re-established keeps requiring verification.
            let _ = owner.reestablish_verified(
                relational,
                snapshot,
                &selected,
                &read.identity,
                facts,
                admission,
            );
        }
        Ok(())
    }
}

/// Every visited native prefix charges the same admission, including refusal.
fn lineage_read(
    admission: &mut InvalidationEditAdmission,
    read: impl FnOnce(
        &mut InvalidationEditAdmission,
    ) -> Result<
        Option<RetainedOutputCurrentnessRead>,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    >,
) -> Result<Option<RetainedOutputCurrentnessRead>, WorthQueryOutputDemandDenial> {
    read(admission).map_err(|stop| {
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
