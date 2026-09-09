use super::*;
use crate::mounting::projection::appearance::{
    UiMountedAppearanceClip as Clip, UiMountedAppearanceClipDenial as Denial,
    UiMountedAppearanceSidecar,
};
use crate::mounting::projection::frame_storage::UiMountedAppearanceNodeInputContext;
use worth_ui_host_contract::{UiAppearanceClip, UiMountedAppearanceMechanic};

#[path = "appearance_retention.rs"]
mod appearance_retention;
#[path = "text_visibility.rs"]
mod text_visibility;

// This mounted-producer world exercises real completion and retained lowering.
// Node declarations/receipts are fixture inputs; it is not an authored launch proof.
#[test]
fn portal_appearance_geometry_removes_restores_and_preserves_other_surface() {
    let mut world = GeometryWorld::new();
    let initial = world.frame(&[0, 1], None);
    let first = context(&initial, world.children[0]);
    let foreign = context(&initial, world.children[1]);
    assert_geometry(
        &first,
        [8.0, 52.0, 220.0, 120.0],
        [20_000, 60_000, 280_000, 320_000],
    );
    assert_geometry(
        &foreign,
        [8.0, 52.0, 220.0, 120.0],
        [20_000, 60_000, 280_000, 320_000],
    );
    let recompleted = world.frame(&[0, 1], Some(&initial));
    assert_geometry(
        &context(&recompleted, world.children[0]),
        [8.0, 52.0, 220.0, 120.0],
        [20_000, 60_000, 280_000, 320_000],
    );
    let (session, binding, _, vector, theme) = crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let mut sidecar = UiMountedAppearanceSidecar::default();
    let initial_work = sidecar.mount(lower(&first, &projection)).unwrap();
    let [UiMountedAppearanceMechanic::Surface(surface)] = initial_work.successor().mechanics()
    else {
        panic!("resolved foreground-independent surface must cross the mounted producer");
    };
    assert_eq!(surface.bounds().x(), 8_000);
    assert_eq!(
        surface.clip(),
        UiAppearanceClip::new(20_000, 60_000, 280_000, 320_000).unwrap()
    );

    let closed = world.frame(&[1], Some(&initial));
    assert_eq!(
        context(&closed, world.children[0]).appearance_clip(),
        Clip::Suppressed
    );
    assert_eq!(
        closed
            .semantic
            .node(world.children[1])
            .unwrap()
            .appearance_geometry,
        initial
            .semantic
            .node(world.children[1])
            .unwrap()
            .appearance_geometry,
        "the unchanged surface inherits its completed fact"
    );
    let removal = sidecar
        .mount(lower(&context(&closed, world.children[0]), &projection))
        .unwrap();
    assert!(removal.successor().mechanics().is_empty());
    assert!(matches!(
        removal.changes(),
        [worth_ui_host_contract::UiMountedAppearanceMechanicChange::Remove(_)]
    ));
    assert_eq!(removal.damage().len(), 1);
    assert_eq!(removal.damage()[0].x(), 8_000);
    assert!(sidecar.current_node_receipts().is_empty());

    // A derived-state rebuild must not query current Portal inputs again.
    let mut reconstructing = closed.clone();
    reconstructing.portal_overlays = initial.portal_overlays.clone();
    let suppressed = context(&reconstructing, world.children[0]);
    assert_eq!(suppressed.appearance_clip(), Clip::Suppressed);
    let rebuilt = sidecar
        .reconstruct(lower(&suppressed, &projection))
        .unwrap();
    assert!(rebuilt.successor().mechanics().is_empty());

    let restored = world.frame(&[0, 1], Some(&closed));
    let restored_context = context(&restored, world.children[0]);
    let restored_work = sidecar
        .mount(lower(&restored_context, &projection))
        .unwrap();
    assert_eq!(restored_work.successor().mechanics().len(), 1);
    assert_geometry(
        &restored_context,
        [8.0, 52.0, 220.0, 120.0],
        [20_000, 60_000, 280_000, 320_000],
    );
    assert_geometry(
        &context(&initial, world.children[0]),
        [8.0, 52.0, 220.0, 120.0],
        [20_000, 60_000, 280_000, 320_000],
    );
    assert_eq!(
        context(&closed, world.children[0]).appearance_clip(),
        Clip::Suppressed
    );
    let _ = session.shutdown();
}

#[test]
fn portal_completion_preserves_independent_ancestry_and_translates_resolved_clip() {
    let mut world = GeometryWorld::new();
    let child = world.children[0];
    for denial in [
        Denial::ScrollBindingUnavailable(crate::graph::UiGraphNodeIdentity::new(4_152)),
        Denial::MosaicBindingUnavailable(crate::graph::UiGraphNodeIdentity::new(4_152)),
    ] {
        world.set_clip(child, Clip::Unresolved(denial));
        let frame = world.frame(&[0, 1], None);
        assert_eq!(
            context(&frame, child).appearance_clip(),
            Clip::Unresolved(denial)
        );
    }
    world.set_clip(
        child,
        Clip::Ancestor(UiAppearanceClip::new(-10_000, -10_000, 110_000, 70_000).unwrap()),
    );
    let frame = world.frame(&[0, 1], None);
    // The exact occurrence retains its position relative to the anchor. After
    // the Portal rebase, coverage starts at 20,60 and ends at 100,100.
    assert_geometry(
        &context(&frame, child),
        [8.0, 52.0, 220.0, 120.0],
        [20_000, 60_000, 80_000, 40_000],
    );
    world.set_clip(
        child,
        Clip::Ancestor(UiAppearanceClip::new(500_000, 0, 10_000, 10_000).unwrap()),
    );
    let empty = world.frame(&[0, 1], None);
    assert_eq!(context(&empty, child).appearance_clip(), Clip::Suppressed);
    assert_eq!(
        context(&frame, child).appearance_clip(),
        Clip::Ancestor(UiAppearanceClip::new(20_000, 60_000, 80_000, 40_000).unwrap())
    );
}

#[test]
fn foreground_candidates_use_retained_portal_geometry_and_ancestor_clip() {
    let mut world = GeometryWorld::new();
    let child = world.children[0];
    world.set_clip(
        child,
        Clip::Ancestor(UiAppearanceClip::new(-10_000, -10_000, 110_000, 70_000).unwrap()),
    );
    let mut frame = world.frame(&[0, 1], None);
    // Drop live overlay lookup inputs after completion: ordinary lowering must
    // consume the carried geometry, not rediscover the Portal owner.
    frame.portal_overlays = Vec::new().into();
    let rows = frame.appearance_text_candidates(child).unwrap();
    assert_eq!(rows.len(), 2);
    for row in rows {
        assert_eq!(row.bounds().x(), 8.0);
        assert_eq!(row.bounds().y(), 52.0);
        assert_eq!(row.clip_bounds().x(), 20.0);
        assert_eq!(row.clip_bounds().y(), 60.0);
        assert_eq!(row.clip_bounds().width(), 80.0);
        assert_eq!(row.clip_bounds().height(), 40.0);
        assert_eq!(row.origin_x(), 8.0);
        assert!(row.origin_y() >= 52.0);
        assert_eq!(row.performed_layout_cost(), None);
    }
}

fn lower(
    context: &UiMountedAppearanceNodeInputContext,
    projection: &crate::runtime::appearance::UiAppearanceProjection,
) -> crate::mounting::UiMountedAppearanceLoweringInput {
    context
        .lower_retained_projection(
            projection,
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            Ok(worth_ui_host_contract::UiAppearanceLogicalLength::new(1_000).unwrap()),
        )
        .unwrap()
}

fn context(
    frame: &UiMountedProjectionFrame,
    child: UiMountedInstanceIdentity,
) -> UiMountedAppearanceNodeInputContext {
    frame
        .appearance_node_inputs_for_reconstruction()
        .unwrap()
        .into_iter()
        .find(|context| context.mounted_instance == child)
        .unwrap()
}

fn assert_geometry(
    context: &UiMountedAppearanceNodeInputContext,
    expected: [f32; 4],
    clip: [i32; 4],
) {
    let UiMountedAllocationProjection::Known { bounds, .. } = context.allocation() else {
        panic!("known allocation required");
    };
    assert_eq!(
        [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
        expected
    );
    assert_eq!(
        bounds.coordinate_space(),
        UiMountedCoordinateSpace::HostSurface,
        "Portal completion preserves the occurrence geometry's surface authority"
    );
    assert_eq!(
        context.appearance_clip(),
        Clip::Ancestor(
            UiAppearanceClip::new(clip[0], clip[1], clip[2] as u32, clip[3] as u32,).unwrap()
        )
    );
}

struct GeometryWorld {
    owners: [UiMountedInstanceIdentity; 2],
    children: [UiMountedInstanceIdentity; 2],
    surfaces: [UiSemanticSurfaceIdentity; 2],
    bindings: [UiSurfaceBindingGeneration; 2],
    semantic: UiMountedSemanticProjection,
    fonts: Arc<worth_ui_text::UiGlobalFontCollection>,
}

impl GeometryWorld {
    fn new() -> Self {
        let owners = std::array::from_fn(|_| UiMountedInstanceIdentity::mint_unbound().unwrap());
        let children = std::array::from_fn(|_| UiMountedInstanceIdentity::mint_unbound().unwrap());
        let surfaces = std::array::from_fn(|_| UiSemanticSurfaceIdentity::mint_unbound().unwrap());
        let bindings = std::array::from_fn(|_| UiSurfaceBindingGeneration::mint_unbound().unwrap());
        let mut records = Vec::new();
        let mut mounted_surfaces = Vec::new();
        for i in 0..2 {
            let semantic =
                portal_semantic_projection(owners[i], children[i], surfaces[i], bindings[i]);
            records.extend(semantic.nodes_in_order().cloned());
            mounted_surfaces.push(UiMountedProjectionSurface {
                surface: surfaces[i],
                binding: bindings[i],
                audience: UiMountedProjectionAudience::full(),
            });
        }
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let mut world = Self {
            owners,
            children,
            surfaces,
            bindings,
            semantic: UiMountedSemanticProjection::initial(records, mounted_surfaces),
            fonts: Arc::new(fonts),
        };
        for child in children {
            world.set_clip(
                child,
                Clip::Unresolved(Denial::PortalBindingUnavailable(
                    crate::graph::UiGraphNodeIdentity::new(4_152),
                )),
            );
        }
        world
    }

    fn set_clip(&mut self, child: UiMountedInstanceIdentity, clip: Clip) {
        let mut node = self.semantic.node(child).unwrap().clone();
        node.appearance_clip = clip;
        self.semantic.insert_node(node);
    }

    fn frame(
        &mut self,
        open: &[usize],
        previous: Option<&UiMountedProjectionFrame>,
    ) -> UiMountedProjectionFrame {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let semantic = previous.map_or_else(
            || self.semantic.clone(),
            |previous| previous.semantic.clone(),
        );
        // Only owner0 changes after initial construction. Owner1's exact overlay
        // input and completed node geometry persist on its separate surface.
        let overlays: Vec<_> = open
            .iter()
            .map(|&i| {
                if i == 1 {
                    if let Some(input) = previous.and_then(|previous| {
                        previous
                            .portal_overlays
                            .iter()
                            .find(|input| input.owner() == self.owners[1])
                    }) {
                        return *input;
                    }
                }
                portal_overlay(frame, self.owners[i], self.surfaces[i], self.bindings[i])
            })
            .collect();
        let changed: Vec<_> = if previous.is_some() {
            vec![self.owners[0], self.children[0]]
        } else {
            semantic.mounted_instances().collect()
        };
        let mut projection = UiMountedProjectionFrame::new(UiMountedProjectionFrameInput {
            frame,
            content_generation: UiMountedContentGeneration::mint_unbound().unwrap(),
            receipt_basis: crate::mounting::UiMountedNodeReceiptBasis::mint(
                frame,
                semantic.membership(),
            )
            .unwrap(),
            plan_digest: 0x3154,
            semantic,
            counters: crate::mounting::UiMountStageCounters::begin(
                crate::mounting::UiMountWorkClass::SemanticDelta,
            ),
            capability_generation: WorthUiHostCapabilityObservationGeneration::new(1),
            capability_profile_digest: 1,
            font_collection: Arc::clone(&self.fonts),
            mechanics: previous
                .map_or_else(Default::default, |previous| previous.mechanic_source()),
            presentation_effects: Default::default(),
            diagnostics: Default::default(),
            portal_overlays: overlays.into(),
            portal_overlays_changed: true,
            changed_instances: changed.into(),
        });
        projection.complete_mechanics().unwrap();
        projection
    }
}
