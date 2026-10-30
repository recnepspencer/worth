//! The public lease retains the exact Native physical Source association.
use super::super::source::occurrences::PersistentSourceRows;
use super::*;
use worth_relational::facade::runtime::ManagedIndexSourceObservation;

struct MaintainedCurrency {
    association: WorthQueryInstalledManagedIndexParent,
    observation: Arc<ManagedIndexSourceObservation>,
    _allocation: Arc<SourceFactsAllocation>,
}
impl RootCurrency for MaintainedCurrency {
    fn current(&self, association: &WorthQueryInstalledManagedIndexParent) -> bool {
        // The Native image belongs to the Source completion's exact selected
        // World basis. Live public currentness is checked by the real Parent
        // and Source sessions; Native may already have a pending successor.
        self.association.same_association(association)
    }
}
impl<'parent, 'request, 'authority> ManagedSelectionLease<'parent, 'request, 'authority> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn issue_maintained(
        proof: WorthQueryManagedIndexSelectionProof<'parent, 'request, 'authority>,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        source: Arc<WorthQueryCompletedSourceRead>,
        observation: Arc<ManagedIndexSourceObservation>,
        rows: &PersistentSourceRows,
        slots: ExecutionArray<Slot>,
        dependencies: Arc<ManagedSourceDependencies>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Self, SelectionDenial> {
        proof.consult().map_err(SelectionDenial::Session)?;
        if !proof.admits(parent, request, association, &source) {
            return Err(SelectionDenial::ForeignSelection);
        }
        let execution = proof.execution();
        let roots = controlled(&execution, |control| {
            let mut output = ExecutionRequestArrayBuilder::allocate(execution, slots.len())
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
            for slot in slots.elements() {
                control.checkpoint(1)?;
                if slot.owner != rows.owner() || slot.ordinal != 0 {
                    return Err(control.refuse(Stop::ForeignSourceSlot).into());
                }
                let entity = slot
                    .entity
                    .filter(|entity| rows.contains(*entity))
                    .ok_or_else(|| control.refuse(Stop::SourceSlotUnavailable))?;
                output
                    .push(entity)
                    .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
            }
            output
                .seal()
                .map_err(|cause| control.refuse(Stop::Storage(cause)).into())
        })
        .map_err(SelectionDenial::Work)?;
        MaterialCompletion::new(&execution, allocation).arc::<MaintainedCurrency>()?;
        let scope = source.scope();
        let lease = Self {
            proof,
            source,
            rows: SelectionRows {
                parent: parent.identity(),
                association: association.clone(),
                scope,
                roots,
                dependencies,
                currency: Arc::new(MaintainedCurrency {
                    association: association.clone(),
                    observation,
                    _allocation: Arc::clone(allocation),
                }),
            },
        };
        lease.rows(parent, request)?;
        Ok(lease)
    }
}
