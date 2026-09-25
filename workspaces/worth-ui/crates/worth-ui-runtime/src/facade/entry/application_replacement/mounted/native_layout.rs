use super::super::WorthUiPreparedApplicationActivation;
use crate::facade::entry::{UiNativeMountedComponentLayoutInput, UiNativeMountedRegionLayoutInput};

/// Exact candidate-mounted inputs lent to the native application's layout owner.
/// The application supplies rectangles; the runtime supplies only identities,
/// admitted measurement contracts, and the basis that binds the result.
pub struct UiNativeReplacementLayoutInput {
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    viewport: worth_ui_host_contract::UiMountedCanonicalBox,
    basis: crate::mounting::UiMountedLayoutBasis,
    revision: crate::mounting::UiMountedLayoutRevision,
    components: Box<[UiNativeMountedComponentLayoutInput]>,
    regions: Box<[UiNativeMountedRegionLayoutInput]>,
}

pub(crate) type UiNativeReplacementLayoutSupplier<'supplier> = dyn FnMut(UiNativeReplacementLayoutInput) -> Option<crate::mounting::UiMountedSurfaceGeometryBatch>
    + 'supplier;

impl UiNativeReplacementLayoutInput {
    pub fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }

    pub fn viewport(&self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.viewport
    }

    pub fn basis(&self) -> crate::mounting::UiMountedLayoutBasis {
        self.basis.clone()
    }

    pub fn revision(&self) -> crate::mounting::UiMountedLayoutRevision {
        self.revision
    }

    pub fn components(&self) -> &[UiNativeMountedComponentLayoutInput] {
        &self.components
    }

    pub fn regions(&self) -> &[UiNativeMountedRegionLayoutInput] {
        &self.regions
    }
}

pub(super) fn prepare_input(
    application: &WorthUiPreparedApplicationActivation,
    successor: &crate::mounting::UiMountedGraphReplacementSuccessor,
    bindings: &crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    viewport: worth_ui_host_contract::UiMountedCanonicalBox,
) -> Result<UiNativeReplacementLayoutInput, crate::mounting::UiMountedOccurrenceGeometryDenial> {
    let authority = application.candidate_replacement_authority();
    let graph = application.candidate_graph();
    let identity = successor.layout_validation_identity();
    let mounted = identity.projection_instances(&[surface]);
    let mut components = Vec::new();
    let mut component_instances = std::collections::BTreeMap::<Box<str>, _>::new();
    for mounted in &mounted {
        let node = graph
            .lookup()
            .graph_node(mounted.graph_node_identity())
            .ok_or(crate::mounting::UiMountedOccurrenceGeometryDenial::UnknownMountedInstance)?;
        let node = node.value();
        let authored = node.declaration_identity().authored_semantic_name();
        let component_name = authored.strip_prefix("component:").unwrap_or(authored);
        let Ok(component_id) = crate::capability::ComponentId::new(component_name) else {
            continue;
        };
        let Some(descriptor) = authority.capabilities().components().get(&component_id) else {
            continue;
        };
        component_instances.insert(authored.into(), mounted.identity());
        components.push((authored.to_owned(), mounted.identity(), descriptor));
    }
    let components = components
        .into_iter()
        .map(|(authored, instance, descriptor)| {
            UiNativeMountedComponentLayoutInput::from_descriptor(
                authored,
                instance,
                Some(descriptor),
                authority.capabilities().components(),
                |component| {
                    component_instances
                        .get(format!("component:{}", component.as_str()).as_str())
                        .copied()
                },
            )
        })
        .collect::<Vec<_>>();
    let declaration = bindings
        .bound_owners()
        .find_map(|(declaration, runtime, _)| (runtime == surface).then_some(declaration));
    let regions = declaration
        .map(|declaration| {
            mounted
                .iter()
                .flat_map(|mounted| {
                    crate::runtime::session::WorthUiApplicationSessionState::region_declarations_from_plan(
                        application.candidate_plan(),
                        graph,
                        authority.authored_overlay_material(),
                        declaration,
                        mounted.graph_node_identity(),
                    )
                    .0
                    .into_iter()
                    .map(move |binding| {
                        UiNativeMountedRegionLayoutInput::from_mounted_region(
                            mounted.identity(),
                            binding.region_kind(),
                            binding.declaration(),
                            binding.executed_region(),
                        )
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let generation = authority.generation_identity().clone();
    let basis = successor.candidate_layout_basis(surface, generation)?;
    let revision = successor.next_candidate_layout_revision(surface)?;
    Ok(UiNativeReplacementLayoutInput {
        surface,
        viewport,
        basis,
        revision,
        components: components.into_boxed_slice(),
        regions: regions.into_boxed_slice(),
    })
}
