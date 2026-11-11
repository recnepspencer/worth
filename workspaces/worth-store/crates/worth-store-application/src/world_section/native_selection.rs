//! The actual Home binds one World source to its actual Native receiver.
use crate::home_context::HomeContext;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_proof::{
    HomeWorldSelectionCompletionOwner, HomeWorldSelectionOwner, VerifiedHomeWorldSelection,
};
use worth_relational::facade::durability::RelationalWorldSourceCatalog;
use worth_runtime_world::facade::{
    PreparedWorldHomeData, PreparedWorldHomeSource, WorldReadmissionDenial as Denial,
};

pub(crate) struct HomeWorldPreparation {
    pub(crate) source: PreparedWorldHomeSource,
    pub(crate) selection:
        Option<Arc<VerifiedHomeWorldSelection<Arc<RelationalWorldSourceCatalog>>>>,
    completion: Option<HomeWorldSelectionCompletionOwner>,
}
#[derive(Debug)]
struct HomeSelectionBacking {
    _memory: Option<ExecutionMemoryReservation>,
}
impl HomeWorldPreparation {
    pub(crate) fn bind(
        context: &HomeContext,
        data: PreparedWorldHomeData,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let medium = context
            .world_slot
            .as_ref()
            .expect("original World slot before attachment")
            .medium();
        let catalog = data.native_selection();
        let source = data.bind(medium, request)?;
        let (selection, completion) = match catalog {
            None => (None, None),
            Some(catalog) => {
                let fixed = HomeWorldSelectionOwner::retained_occurrence_bytes()
                    + size_of::<HomeSelectionBacking>()
                    + 2 * size_of::<usize>()
                    + size_of::<VerifiedHomeWorldSelection<Arc<RelationalWorldSourceCatalog>>>()
                    + 2 * size_of::<usize>();
                let memory = request
                    .reserve_retained_memory(fixed as u64)
                    .map_err(|cause| {
                        Denial::Medium(
                            worth_runtime_world::facade::WorldDurableMediumDenial::Request(cause),
                        )
                    })?;
                let owner = HomeWorldSelectionOwner::from_retained_home_owner(
                    context.descriptor.identity.bytes(),
                    context.descriptor.incarnation.fence(),
                    Arc::new(HomeSelectionBacking { _memory: memory }),
                );
                let authority = owner.authority();
                let (source, completion) = owner.admit_source(catalog);
                let source = authority
                    .verify_source(source)
                    .expect("same original Home source issuer");
                (Some(Arc::new(source)), Some(completion))
            }
        };
        Ok(Self {
            source,
            selection,
            completion,
        })
    }
    pub(crate) fn complete(
        &mut self,
        native: &worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
    ) {
        if let Some(completion) = self.completion.take() {
            native
                .complete_home_world_selection(completion.complete())
                .expect("the Home completes only its same original Native handoff");
        }
    }
}
