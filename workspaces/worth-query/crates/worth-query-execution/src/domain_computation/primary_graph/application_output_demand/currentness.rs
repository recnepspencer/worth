use std::num::NonZeroUsize;

use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::{
    application_attempt::WorthQuerySourceCurrentnessFailure, WorthQueryApplicationCommitReceipt,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandSettlement, WorthQuerySelectedProductOperation,
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
        runtime.primary_provider.graph.with_runtime(|relational| {
            let mut remaining_work = maximum_work.get();
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
                let read = if let Some(receipt) = settlement.application_commit_receipt() {
                    lineage.source_facts_for_receipt(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        receipt,
                        remaining_work,
                    )
                } else if let Some(stable) = settlement.stable.as_ref() {
                    lineage.source_facts_for_stable_output(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        stable,
                        remaining_work,
                    )
                } else {
                    lineage.source_facts_for_restored_output(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        settlement,
                        remaining_work,
                    )
                }
                .map_err(|()| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))?
                .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Superseded))?;
                drop(lineage);
                remaining_work -= read.work;
                self.require_current_read(relational, &read, &mut remaining_work)?;
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
        runtime.primary_provider.graph.with_runtime(|relational| {
            let mut remaining_work = maximum_work.get();
            for receipt in receipts {
                if receipt.runtime_authority() != runtime.runtime.authority_identity()
                    || receipt.principal_scope().binding_identity()
                        != &runtime.installed_schema.binding_identity()
                {
                    return Err(denial(WorthQueryOutputDemandDenialKind::ForeignSettlement));
                }
                let read = runtime
                    .primary_provider
                    .graph
                    .output_lineage
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .source_facts_for_receipt(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        receipt,
                        remaining_work,
                    )
                    .map_err(|()| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))?
                    .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Superseded))?;
                remaining_work -= read.work;
                self.require_current_read(relational, &read, &mut remaining_work)?;
            }
            Ok(())
        })
    }

    /// Compares the output witness and every source fact at this observation.
    /// That full verification re-establishes the row's marks, so the next
    /// demand at this image reads them instead of verifying again.
    fn require_current_read(
        &self,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        read: &crate::domain_computation::primary_graph::output_lineage::RetainedOutputCurrentnessRead,
        remaining_work: &mut usize,
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
            owner,
            remaining_work,
        )?;
        require_current_facts(relational, snapshot, read.facts.iter(), remaining_work).map_err(
            |error| match read.verification_requirement {
                Some(crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason::NativeFactRevisionUnavailable(ordinal)) => WorthQueryOutputDemandDenial::new(
                    error.kind(),
                    format!("{}; native revision unavailable for: {}", error.subject(),
                        read.facts.get(ordinal).map_or_else(|| "unavailable retained fact".into(), |fact| fact.locator_identity())),
                ),
                Some(crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason::IndexedSelectionFactDenied(ordinal, denial)) => WorthQueryOutputDemandDenial::new(
                    error.kind(),
                    format!("{}; native indexed selection denied: {denial:?}; fact: {}", error.subject(),
                        read.facts.get(ordinal).map_or_else(|| "unavailable retained fact".into(), |fact| fact.locator_identity())),
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
        if let Ok(selected) = relational.read_truth().positioned_snapshot(snapshot) {
            let mut admission = owner.edit_admission();
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
                    &mut admission,
                );
            }
            // A row that cannot be re-established keeps requiring verification.
            let _ = owner.reestablish_verified(
                relational,
                snapshot,
                &selected,
                &read.identity,
                &read.facts,
                &mut admission,
            );
        }
        Ok(())
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
    owner: &crate::domain_computation::primary_graph::SourceInvalidationOwner,
    remaining_work: &mut usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    let witness = witness
        .and_then(|witness| witness.get())
        .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable))?;
    let mut admission = owner.read_admission(*remaining_work);
    let unchanged = witness.unchanged_in(relational, snapshot, &mut admission);
    let charged = usize::try_from(admission.charged_work())
        .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))?;
    *remaining_work = remaining_work
        .checked_sub(charged)
        .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))?;
    match unchanged {
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

fn require_current_facts<'fact>(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: impl IntoIterator<
        Item = &'fact crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact,
    >,
    remaining_work: &mut usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    for fact in facts {
        let available = *remaining_work;
        let prepaid = prepay_exact_probe(fact, remaining_work)?;
        let (current, work) = fact
            .source_currentness_in(relational, snapshot, available)
            .map_err(|failure| match failure {
                WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                    denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded)
                }
                WorthQuerySourceCurrentnessFailure::Unavailable => denial_subject(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    fact.locator_identity(),
                ),
            })?;
        *remaining_work -= work.saturating_sub(prepaid);
        if !current {
            return Err(denial_subject(
                WorthQueryOutputDemandDenialKind::Superseded,
                fact.locator_identity(),
            ));
        }
    }
    Ok(())
}

fn prepay_exact_probe(
    fact: &crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact,
    remaining_work: &mut usize,
) -> Result<usize, WorthQueryOutputDemandDenial> {
    let work = fact
        .exact_probe_work()
        .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))?
        .unwrap_or(0);
    if work > *remaining_work {
        return Err(denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded));
    }
    *remaining_work -= work;
    Ok(work)
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
