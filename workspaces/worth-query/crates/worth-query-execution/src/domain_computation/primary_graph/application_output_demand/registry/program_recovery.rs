use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn retain_program_recovery<
        Inventory: 'static,
        Root: 'static,
        Demand: Clone + Send + Sync + 'static,
    >(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        demand: &Demand,
        preparation: &super::WorthQueryRequiredOutputSourcePreparation,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let source_commit = receipt
            .committed_product_publication()
            .composite_commit()
            .clone();
        let occurrence = receipt.product_branch().occurrence();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custody = ProgramRecoveryCustody {
            provider_runtime_instance_id: receipt.provider_runtime_instance_id(),
            product_occurrence: occurrence,
            source_commit,
            inventory: std::any::TypeId::of::<Inventory>(),
            root: std::any::TypeId::of::<Root>(),
            demand: Box::new(demand.clone()),
        };
        Self::retain_program_recovery_custody(&mut state, preparation, custody)
    }

    pub(super) fn retain_program_recovery_custody(
        state: &mut DemandRegistryState,
        preparation: &super::WorthQueryRequiredOutputSourcePreparation,
        custody: ProgramRecoveryCustody,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if preparation.occurrence != custody.product_occurrence
            || state
                .source_preparations
                .get(&custody.product_occurrence)
                .is_none_or(|entry| entry.retired || entry.active == 0)
        {
            let (kind, subject) = if preparation.occurrence != custody.product_occurrence {
                (
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "program recovery preparation belongs to another occurrence",
                )
            } else {
                (
                    WorthQueryOutputDemandDenialKind::Closed,
                    "program recovery occurrence retired before custody transfer",
                )
            };
            return Err(denial(kind, subject));
        }
        let source_commit = custody.source_commit.clone();
        let occurrence = custody.product_occurrence;
        state.program_recovery.retain(|entry| {
            entry.source_commit != source_commit && entry.product_occurrence != occurrence
        });
        state.program_recovery.push(custody);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn recover_program_demand<
        Inventory: 'static,
        Root: 'static,
        Demand: Clone + 'static,
    >(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<Demand, WorthQueryOutputDemandDenial> {
        let source_commit = receipt.committed_product_publication().composite_commit();
        let occurrence = receipt.product_branch().occurrence();
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(entry) = state.program_recovery.iter().find(|entry| {
            &entry.source_commit == source_commit
                && entry.provider_runtime_instance_id == receipt.provider_runtime_instance_id()
        }) {
            if entry.inventory != std::any::TypeId::of::<Inventory>()
                || entry.root != std::any::TypeId::of::<Root>()
            {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::ForeignSettlement,
                    "recovery custody belongs to another program inventory or root",
                ));
            }
            return entry
                .demand
                .downcast_ref::<Demand>()
                .cloned()
                .ok_or_else(|| {
                    denial(
                        WorthQueryOutputDemandDenialKind::ForeignSettlement,
                        "recovery custody demand type does not match its installed root",
                    )
                });
        }
        if state.program_recovery.iter().any(|entry| {
            entry.product_occurrence == occurrence
                && entry.provider_runtime_instance_id == receipt.provider_runtime_instance_id()
        }) {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                "program recovery custody belongs to a newer source commit",
            ));
        }
        Err(denial(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            "installed program has no retained recovery custody for this source",
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn release_program_recovery(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .program_recovery
            .retain(|entry| &entry.source_commit != source_commit);
    }

    pub(super) fn retire_program_recovery_occurrence(
        state: &mut DemandRegistryState,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        state
            .program_recovery
            .retain(|entry| entry.product_occurrence != occurrence);
    }
}

fn denial(kind: WorthQueryOutputDemandDenialKind, subject: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
