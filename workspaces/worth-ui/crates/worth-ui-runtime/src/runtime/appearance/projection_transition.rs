use super::projection::{
    UiAppearanceChangeReceipt, UiAppearanceMountAffinity, UiAppearanceProjection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiAppearanceProjectionKey {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
}

struct UiAppearanceProjectionEntry {
    key: UiAppearanceProjectionKey,
    projection: UiAppearanceProjection,
    sidecar: crate::mounting::UiMountedAppearanceSidecar,
}

pub(crate) struct UiAppearanceProjectionTransitionState {
    entries: Vec<UiAppearanceProjectionEntry>,
}

impl UiAppearanceProjectionTransitionState {
    pub(crate) fn predecessor_for(
        &self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        affinity: UiAppearanceMountAffinity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
    ) -> Option<&UiAppearanceProjection> {
        let key = UiAppearanceProjectionKey {
            session,
            generation: generation.clone(),
            surface: affinity.surface,
            graph_node,
            mounted_instance: affinity.mounted_instance,
            incarnation,
        };
        self.entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.projection)
    }

    pub(crate) fn mount(
        &mut self,
        projection: UiAppearanceProjection,
        input: crate::mounting::UiMountedAppearanceLoweringInput,
        affinity: UiAppearanceMountAffinity,
    ) -> Result<UiAppearanceChangeReceipt, UiAppearanceMountDenial> {
        let key = key_for(&projection);
        let position = self.entries.iter().position(|entry| entry.key == key);
        let predecessor = position.map(|index| &self.entries[index].projection);
        let mut sidecar = position
            .map(|index| self.entries[index].sidecar.clone())
            .unwrap_or_default();
        let work = sidecar
            .mount(input)
            .map_err(UiAppearanceMountDenial::Lowering)?;
        let receipt = UiAppearanceChangeReceipt::from_resolved_mount(
            predecessor,
            &projection,
            &work,
            affinity,
        )
        .map_err(UiAppearanceMountDenial::Affinity)?;
        let entry = UiAppearanceProjectionEntry {
            key,
            projection,
            sidecar,
        };
        if let Some(position) = position {
            self.entries[position] = entry;
        } else {
            self.entries.push(entry);
        }
        Ok(receipt)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceMountDenial {
    Lowering(crate::mounting::UiMountedAppearanceLoweringDenial),
    Affinity(super::projection::UiAppearanceMountAffinityDenial),
}

impl Default for UiAppearanceProjectionTransitionState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

fn key_for(projection: &UiAppearanceProjection) -> UiAppearanceProjectionKey {
    let basis = projection.state().basis();
    UiAppearanceProjectionKey {
        session: basis.session(),
        generation: basis.generation().clone(),
        surface: basis.surface(),
        graph_node: basis.graph_node(),
        mounted_instance: basis.mounted_instance(),
        incarnation: basis.incarnation(),
    }
}
