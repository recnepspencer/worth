use std::sync::Arc;
use worth_foundational::facade::{DurableWorldOwnerIdentity, WorldOwnerSlotIdentity};
use worth_runtime_world::facade::WorldDurableHomeBinding;
use worth_store::physical_runtime::ServingPhysicalRuntime;
pub(super) fn attach_bridge(
    section: &worth_store_relational::RelationalStoreSection,
    builder: worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder,
    graph_role: String,
) -> Result<
    worth_runtime_world::facade::RuntimeWorldCorrespondencePort,
    crate::ApplicationHomeOpenRefusal,
> {
    let (_, bridge) = build_bridge(section, builder, graph_role)?;
    Ok(bridge.runtime_world_correspondence_port())
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
pub(super) struct ApplicationHomeBridgeComposition {
    pub(super) source: worth_runtime_bridge::facade::RuntimeBridgeRelationalSource,
    pub(super) ordinary: worth_runtime_bridge::facade::RuntimeBridge,
    pub(super) conditional:
        Arc<std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>>,
}
impl ApplicationHomeBridgeComposition {
    pub(super) fn close(self) {
        self.conditional
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .close_conditional_resources();
        // Drops the same source and ordinary Bridge after actual conditional
        // cleanup; independently retained caller handles remain real custody.
        drop(self);
    }
}
pub(super) fn attach_query_bridge(
    section: &worth_store_relational::RelationalStoreSection,
    builder: worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder,
    graph_role: String,
    conditional_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
) -> Result<ApplicationHomeBridgeComposition, crate::ApplicationHomeOpenRefusal> {
    let (source, ordinary) = build_bridge(section, builder, graph_role)?;
    let conditional =
        worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
            ordinary.clone(),
            conditional_budget,
        )
        .and_then(|builder| builder.seal())
        .map_err(crate::ApplicationHomeOpenRefusal::ConditionalBridge)?;
    Ok(ApplicationHomeBridgeComposition {
        source,
        ordinary,
        conditional: Arc::new(std::sync::RwLock::new(conditional)),
    })
}
/// Actual admitted Home custody. No Clone, raw identity constructor or second
/// extraction exists outside the Home's private composition.
pub(crate) struct UnclaimedWorldSlot {
    medium: Arc<super::ApplicationWorldMedium>,
}
impl UnclaimedWorldSlot {
    pub(crate) fn from_admitted_home(
        serving: Arc<ServingPhysicalRuntime>,
        descriptor: &crate::descriptor::HomeDescriptor,
    ) -> Self {
        let owner = DurableWorldOwnerIdentity::from_recorded(
            descriptor.identity,
            WorldOwnerSlotIdentity::from_recorded(0),
        );
        Self {
            medium: Arc::new(super::ApplicationWorldMedium {
                serving,
                binding: WorldDurableHomeBinding::from_admitted_adapter(
                    owner,
                    descriptor.incarnation,
                    *descriptor.binding.selected().revision().bytes(),
                ),
                placement: crate::open::physical_admission::placement(),
                publications: Arc::new(super::WorldPublications::default()),
            }),
        }
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
