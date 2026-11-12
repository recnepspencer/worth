use std::sync::Arc;
use worth_foundational::facade::{DurableWorldOwnerIdentity, WorldOwnerSlotIdentity};
use worth_runtime_world::facade::WorldDurableHomeBinding;
use worth_store::physical_runtime::ServingPhysicalRuntime;
pub(super) fn attach_bridge(
    section: &worth_store_relational::RelationalStoreSection,
    builder: worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder,
    graph_role: String,
) -> Result<ApplicationHomeBridgeComposition, crate::ApplicationHomeOpenRefusal> {
    let (source, ordinary) = build_bridge(section, builder, graph_role)?;
    Ok(ApplicationHomeBridgeComposition {
        source,
        ordinary,
        conditional: None,
        retirement: std::sync::Mutex::new(None),
    })
}

fn build_bridge(
    section: &worth_store_relational::RelationalStoreSection,
    builder: worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder,
    graph_role: String,
) -> Result<
    (
        worth_runtime_bridge::facade::RuntimeBridgeRelationalSource,
        worth_runtime_bridge::facade::RuntimeBridge,
    ),
    crate::ApplicationHomeOpenRefusal,
> {
    let source = worth_runtime_bridge::facade::RuntimeBridgeRelationalSource::for_graph_role(
        section.world_composition_runtime(),
        graph_role,
    )
    .map_err(crate::ApplicationHomeOpenRefusal::BridgeSource)?;
    let bridge = builder
        .with_source_adapter(source.clone())
        .with_relational_source(source.clone())
        .build()
        .map_err(crate::ApplicationHomeOpenRefusal::Bridge)?;
    Ok((source, bridge))
}

/// These are the actual three owner objects shared by the Query/Home recipe.
pub(crate) struct ApplicationHomeBridgeComposition {
    pub(super) source: worth_runtime_bridge::facade::RuntimeBridgeRelationalSource,
    pub(super) ordinary: worth_runtime_bridge::facade::RuntimeBridge,
    pub(super) retirement: std::sync::Mutex<Option<super::close::PreparedHomeBridgeRetirement>>,
    pub(super) conditional:
        Option<Arc<worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly>>,
}
/// Actual admitted Home custody. No Clone, raw identity constructor or second
/// extraction exists outside the Home's private composition.
pub(crate) struct UnclaimedWorldSlot {
    signal: crate::signal_medium::UnclaimedSignalSlot,
    program_source: Arc<worth_proof::InstalledBranchProgramSourceOwner>,
    medium: Arc<super::ApplicationWorldMedium>,
}
impl UnclaimedWorldSlot {
    pub(crate) fn owner_identity_for_home(
        home: worth_foundational::facade::HomeIdentity,
    ) -> DurableWorldOwnerIdentity {
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(0))
    }
    pub(crate) fn from_admitted_home(
        serving: Arc<ServingPhysicalRuntime>,
        descriptor: &crate::descriptor::HomeDescriptor,
        source_binding: Option<WorldDurableHomeBinding>,
        transition: worth_proof::HomeProgramTransitionAuthority,
        transition_attempt_frontier: Option<u64>,
        checkpoint: &Arc<crate::checkpoint::owner::HomeCheckpointOwner>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, worth_execution::WorkCeilingDenial> {
        let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<super::ApplicationWorldMedium>())
            .expect("fixed original World medium layout");
        let funding = request.reserve_retained_memory(layout.pad_to_align().size() as u64)?;
        let owner = Self::owner_identity_for_home(descriptor.identity);
        let program_source = worth_proof::InstalledBranchProgramSourceOwner::fresh();
        let program_authority = program_source.authority();
        Ok(Self {
            signal: crate::signal_medium::UnclaimedSignalSlot::from_admitted_home(
                &serving, descriptor,
            ),
            program_source: Arc::new(program_source),
            medium: Arc::new(super::ApplicationWorldMedium {
                serving: std::sync::Mutex::new(Some(serving)),
                _funding: funding,
                binding: source_binding
                    .unwrap_or_else(|| {
                        WorldDurableHomeBinding::from_admitted_adapter(
                            owner,
                            descriptor.incarnation,
                            *descriptor.binding.selected().revision().bytes(),
                        )
                    })
                    .with_branch_program_source_authority(program_authority)
                    .with_home_transition_authority(transition)
                    .with_home_transition_attempt_frontier(transition_attempt_frontier),
                placement: crate::open::physical_admission::placement(),
                publications: Arc::new(super::WorldPublications::default()),
                checkpoint: Arc::downgrade(checkpoint),
            }),
        })
    }
    pub(in crate::world_section) fn program_source_owner(
        &self,
    ) -> Arc<worth_proof::InstalledBranchProgramSourceOwner> {
        Arc::clone(&self.program_source)
    }
    pub(crate) fn signal_slot(&self) -> &crate::signal_medium::UnclaimedSignalSlot {
        &self.signal
    }
    pub(crate) fn publications(&self) -> Arc<super::WorldPublications> {
        self.medium.publications.clone()
    }
    pub(in crate::world_section) fn medium(
        &self,
    ) -> Arc<dyn worth_runtime_world::facade::RuntimeWorldDurableMedium> {
        self.medium.clone()
    }
}
