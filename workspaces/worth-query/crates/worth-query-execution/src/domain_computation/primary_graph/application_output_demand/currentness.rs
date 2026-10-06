use std::num::NonZeroUsize;

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
    pub fn require_current_output_settlements<'settlement>(
        &self,
        settlements: impl IntoIterator<Item = &'settlement WorthQueryOutputDemandSettlement>,
        maximum_work: NonZeroUsize,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        let mut admission = self.request_admission(maximum_work);
        runtime.primary_provider.graph.with_runtime(|relational| {
            for settlement in settlements {
                if !settlement.belongs_to(runtime) {
                    return Err(denial(WorthQueryOutputDemandDenialKind::ForeignSettlement));
                }
                let lineage = runtime
                    .primary_provider
                    .graph
                    .output_lineage
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let read = lineage_read(&mut admission, |maximum_work| {
                    if let Some(receipt) = settlement.application_commit_receipt() {
                        lineage.source_facts_for_receipt(
                            runtime.runtime.authority_identity().as_u64(),
                            &runtime.installed_schema.binding_identity(),
                            self.product().observation(),
                            receipt,
                            maximum_work,
                        )
                    } else if let Some(stable) = settlement.stable.as_ref() {
                        lineage.source_facts_for_stable_output(
                            runtime.runtime.authority_identity().as_u64(),
                            &runtime.installed_schema.binding_identity(),
                            self.product().observation(),
                            stable,
                            maximum_work,
                        )
                    } else {
                        lineage.source_facts_for_restored_output(
                            runtime.runtime.authority_identity().as_u64(),
                            &runtime.installed_schema.binding_identity(),
                            self.product().observation(),
                            settlement,
                            maximum_work,
                        )
                    }
                })?;
                drop(lineage);
                let Some(read) = read else {
                    let receipt = settlement.application_commit_receipt();
                    require_own_publication(receipt, self.product().observation())?;
                    continue;
                };
                self.require_current_read(relational, &read, &mut admission)?;
            }
            Ok(())
        })
    }

    /// Checks a set of retained producer receipts against this exact product
    /// observation, including their observed source facts.
    pub fn require_current_output_receipts<'receipt>(
        &self,
        receipts: impl IntoIterator<Item = &'receipt WorthQueryApplicationCommitReceipt>,
        maximum_work: NonZeroUsize,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        let mut admission = self.request_admission(maximum_work);
        runtime.primary_provider.graph.with_runtime(|relational| {
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
                let read = lineage_read(&mut admission, |maximum_work| {
                    lineage.source_facts_for_receipt(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        receipt,
                        maximum_work,
                    )
                })?;
                drop(lineage);
                let Some(read) = read else {
                    require_own_publication(Some(receipt), self.product().observation())?;
                    continue;
                };
                self.require_current_read(relational, &read, &mut admission)?;
            }
            Ok(())
        })
    }

    /// The request's meter, within the caller's declared maximum.
    fn request_admission(&self, maximum_work: NonZeroUsize) -> InvalidationEditAdmission {
        self.application()
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .edit_admission_within(maximum_work)
    }

    /// Compares the output witness and every source fact at this observation.
    /// That full verification re-establishes the row's marks, so the next
    /// demand at this image reads them instead of verifying again. The
    /// request pays for re-establishing them; their outcome changes no answer.
    pub(in crate::domain_computation::primary_graph) fn require_current_read(
        &self,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        read: &RetainedOutputCurrentnessRead,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
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
        require_current_facts(relational, snapshot, read, admission)?;
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
                    &read.facts,
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
                &read.facts,
                admission,
            );
        }
        Ok(())
    }
}

/// One lineage read on the request's meter: its remaining work is reserved
/// before the read and settled at what the read spent. A meter that cannot
/// reserve, or a read its reservation stops, is the request's work answer.
fn lineage_read(
    admission: &mut InvalidationEditAdmission,
    read: impl FnOnce(usize) -> Result<Option<RetainedOutputCurrentnessRead>, ()>,
) -> Result<Option<RetainedOutputCurrentnessRead>, WorthQueryOutputDemandDenial> {
    admission
        .reserved_read(|maximum_work| {
            read(maximum_work).map(|read| {
                let work = read.as_ref().map_or(0, |read| read.work);
                (read, work)
            })
        })
        .ok()
        .and_then(Result::ok)
        .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))
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

fn require_witnessed_output(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    witness: Option<
        &std::sync::Arc<
            std::sync::OnceLock<
                crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness,
            >,
        >,
    >,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    let witness = witness
        .and_then(|witness| witness.get())
        .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable))?;
    match witness.unchanged_in(relational, snapshot, admission) {
        Ok(true) => Ok(()),
        Ok(false) => Err(denial(WorthQueryOutputDemandDenialKind::Superseded)),
        Err(
            CompanionPreflightStop::WorkExhausted { .. }
            | CompanionPreflightStop::WorkCounterOverflow,
        ) => Err(denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded)),
        Err(_) => Err(denial(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        )),
    }
}

fn require_current_facts(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    read: &RetainedOutputCurrentnessRead,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    match first_moved_fact(relational, snapshot, read.facts.iter(), admission) {
        Ok(None) => Ok(()),
        Ok(Some(moved)) => Err(denial_subject(
            WorthQueryOutputDemandDenialKind::Superseded,
            moved.locator_identity(),
        )),
        Err(RetainedFactStop::Work) => {
            Err(denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))
        }
        Err(RetainedFactStop::Unavailable(fact)) => Err(denial_subject(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            fact.locator_identity(),
        )),
    }
}

fn denial_subject(
    kind: WorthQueryOutputDemandDenialKind,
    fact: String,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, format!("settled output fact is not current: {fact}"))
}

fn denial(kind: WorthQueryOutputDemandDenialKind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        kind,
        "settled output is not current at the selected observation",
    )
}
