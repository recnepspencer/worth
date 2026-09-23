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
                let (facts, lineage_work) = if let Some(receipt) = settlement.receipt.as_ref() {
                    lineage.source_facts_for_receipt(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        receipt,
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
                remaining_work -= lineage_work;
                require_current_facts(
                    relational,
                    self.application_basis().snapshot_handle(),
                    facts.iter(),
                    &mut remaining_work,
                )?;
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
                let (facts, lineage_work) = runtime
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
                remaining_work -= lineage_work;
                for fact in facts.iter() {
                    let (current, work) = fact
                        .source_currentness_in(
                            relational,
                            self.application_basis().snapshot_handle(),
                            remaining_work,
                        )
                        .map_err(|failure| match failure {
                            WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                                denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded)
                            }
                            WorthQuerySourceCurrentnessFailure::Unavailable => denial_subject(
                                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                                fact.locator_identity(),
                            ),
                        })?;
                    remaining_work -= work;
                    if !current {
                        return Err(denial_subject(
                            WorthQueryOutputDemandDenialKind::Superseded,
                            fact.locator_identity(),
                        ));
                    }
                }
            }
            Ok(())
        })
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
        let (current, work) = fact
            .source_currentness_in(relational, snapshot, *remaining_work)
            .map_err(|failure| match failure {
                WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                    denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded)
                }
                WorthQuerySourceCurrentnessFailure::Unavailable => denial_subject(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    fact.locator_identity(),
                ),
            })?;
        *remaining_work -= work;
        if !current {
            return Err(denial_subject(
                WorthQueryOutputDemandDenialKind::Superseded,
                fact.locator_identity(),
            ));
        }
    }
    Ok(())
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
