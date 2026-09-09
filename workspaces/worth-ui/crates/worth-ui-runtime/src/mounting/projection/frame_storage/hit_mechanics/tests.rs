use super::*;
use worth_ui_host_contract::*;

const BUDGET: UiMountedSpatialBudget = UiMountedSpatialBudget {
    node_visits: 1024,
    candidates: 128,
};

#[test]
fn canonical_hit_rows_keep_spatial_membership_through_edits_and_denials() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let peer = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let base = row(instance, surface, binding, 0);
    let mut source = UiMountedHitMechanicSource::default();
    source.replace(instance, Some(base)).unwrap();
    source
        .replace(peer, Some(row(peer, surface, binding, 1)))
        .unwrap();
    let predecessor = source.clone();
    let bytes = source.retained_structural_bytes().unwrap();
    let digest = source.digest();
    let unchanged = source.replace(instance, Some(base)).unwrap();
    assert_eq!((unchanged.node_visits, unchanged.node_copies), (0, 0));
    assert!(unchanged.map_key_probes > 0);
    assert_eq!(source.retained_structural_bytes(), Some(bytes));
    assert_eq!(source.digest(), digest);

    assert!(
        matches!(source.allocation_candidates(binding, UiMountedCoordinateSpace::HostSurface, [5.0, 5.0],
        UiMountedSpatialBudget { node_visits: 1, candidates: 128 }),
        Err(UiMountedSpatialQueryDenial::NodeBudget { work }) if work.node_visits == 1 && work.map_key_probes > 0)
    );
    assert!(
        matches!(source.allocation_candidates(binding, UiMountedCoordinateSpace::HostSurface, [5.0, 5.0],
        UiMountedSpatialBudget { node_visits: 1024, candidates: 1 }),
        Err(UiMountedSpatialQueryDenial::CandidateBudget { work }) if work.map_key_probes > 0)
    );
    assert!(matches!(
        source.replace(instance, Some(row(instance, surface, binding, 1))),
        Err(UiMountedProjectionDenial::DuplicateHitTestOrder { .. })
    ));
    assert!(matches!(
        source.replace(peer, Some(base)),
        Err(UiMountedProjectionDenial::HitTestNodeReceiptMismatch)
    ));
    assert_eq!(source.digest(), digest);
    assert_eq!(source.retained_structural_bytes(), Some(bytes));
    assert_eq!(
        candidates(&source, binding, [5.0, 5.0]),
        candidates(&predecessor, binding, [5.0, 5.0])
    );

    let moved = geometry(
        base,
        [100.0, 0.0, 10.0, 10.0],
        [102.0, 2.0, 6.0, 6.0],
        UiMountedCoordinateSpace::HostSurface,
    )
    .unwrap();
    source.replace(instance, Some(moved)).unwrap();
    assert_eq!(candidates(&source, binding, [5.0, 5.0]), [peer]);
    assert_eq!(candidates(&source, binding, [103.0, 3.0]), [instance]);
    assert!(candidates(&source, binding, [101.0, 3.0]).is_empty());
    assert!(candidates(&source, binding, [108.0, 3.0]).is_empty());
    assert!(candidates(&predecessor, binding, [103.0, 3.0]).is_empty());

    assert!(matches!(
        geometry(
            moved,
            [100.0, 0.0, 10.0, 10.0],
            [102.0, 2.0, 0.0, 6.0],
            UiMountedCoordinateSpace::HostSurface,
        ),
        Err(UiMountedHitTestCompletionDenial::NonAreaGeometry)
    ));
    assert_eq!(candidates(&source, binding, [103.0, 3.0]), [instance]);
    let clipped = geometry(
        moved,
        [100.0, 0.0, 10.0, 10.0],
        [120.0, 0.0, 10.0, 10.0],
        UiMountedCoordinateSpace::HostSurface,
    )
    .unwrap();
    source.replace(instance, Some(clipped)).unwrap();
    assert_eq!(
        source.len(),
        2,
        "disjoint clipping removes spatial membership, not the canonical mechanic"
    );
    assert!(candidates(&source, binding, [103.0, 3.0]).is_empty());

    let local = geometry(
        base,
        [0.0, 0.0, 10.0, 10.0],
        [0.0, 0.0, 10.0, 10.0],
        UiMountedCoordinateSpace::Viewport,
    )
    .unwrap();
    source.replace(instance, Some(local)).unwrap();
    assert_eq!(candidates(&source, binding, [5.0, 5.0]), [peer]);
    assert_eq!(
        source
            .allocation_candidates(
                binding,
                UiMountedCoordinateSpace::Viewport,
                [5.0, 5.0],
                BUDGET
            )
            .unwrap()
            .instances,
        [instance]
    );
    source.replace(instance, None).unwrap();
    source.replace(peer, None).unwrap();
    assert_eq!(source.len(), 0);
    assert_eq!(source.retained_structural_bytes(), Some(0));
    assert_eq!(candidates(&predecessor, binding, [5.0, 5.0]).len(), 2);
}

#[test]
fn binding_reconstruction_and_replacement_retire_only_owned_buckets() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let peer = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let other_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let other_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let mut source = UiMountedHitMechanicSource::default();
    source
        .replace(instance, Some(row(instance, surface, binding, 0)))
        .unwrap();
    source
        .replace(peer, Some(row(peer, other_surface, other_binding, 0)))
        .unwrap();
    let replacement = replacement(surface);
    let (rebound, work) = source.rebind(&[(binding, replacement)]).unwrap();
    assert_eq!(work.reconstructed_rows, 2);
    assert!(work.map_key_probes > 0);
    assert_eq!(work.node_copies, 2);
    assert!(candidates(&rebound, binding, [5.0, 5.0]).is_empty());
    assert_eq!(
        candidates(&rebound, replacement.binding_generation(), [5.0, 5.0]),
        [instance]
    );
    assert_eq!(candidates(&rebound, other_binding, [5.0, 5.0]), [peer]);
    assert_eq!(candidates(&source, binding, [5.0, 5.0]), [instance]);
    assert!(source
        .rebind(&[(binding, self::replacement(other_surface))])
        .is_err());
    assert_eq!(candidates(&source, binding, [5.0, 5.0]), [instance]);
    source
        .replace(
            instance,
            Some(row(instance, surface, replacement.binding_generation(), 0)),
        )
        .unwrap();
    assert!(candidates(&source, binding, [5.0, 5.0]).is_empty());
    assert_eq!(
        candidates(&source, replacement.binding_generation(), [5.0, 5.0]),
        [instance]
    );
    assert_eq!(candidates(&source, other_binding, [5.0, 5.0]), [peer]);
}

fn candidates(
    source: &UiMountedHitMechanicSource,
    binding: UiSurfaceBindingGeneration,
    point: [f64; 2],
) -> Vec<UiMountedInstanceIdentity> {
    source
        .allocation_candidates(
            binding,
            UiMountedCoordinateSpace::HostSurface,
            point,
            BUDGET,
        )
        .unwrap()
        .instances
}

// These fixtures exercise completed-mechanic storage, not presentation admission.
fn row(
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    rank: u32,
) -> UiMountedHitTestMechanic {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let mut instances = crate::runtime::persistent_index::UiPersistentOrdSet::default();
    instances.insert(instance);
    let receipts = crate::mounting::UiMountedNodeReceiptBasis::mint(frame, instances).unwrap();
    UiMountedHitTestMechanic::complete_from_runtime_mounting(UiMountedHitTestCompletionInput {
        frame,
        surface,
        binding,
        mounted_instance: instance,
        node_receipt: receipts.receipt_for(instance).unwrap(),
        bounds: bounds(
            [0.0, 0.0, 10.0, 10.0],
            UiMountedCoordinateSpace::HostSurface,
        ),
        clip_bounds: bounds(
            [0.0, 0.0, 10.0, 10.0],
            UiMountedCoordinateSpace::HostSurface,
        ),
        order: UiMountedHitTestOrder::from_runtime_plan(rank),
    })
    .unwrap()
}

fn geometry(
    row: UiMountedHitTestMechanic,
    allocation: [f32; 4],
    clip: [f32; 4],
    space: UiMountedCoordinateSpace,
) -> Result<UiMountedHitTestMechanic, UiMountedHitTestCompletionDenial> {
    UiMountedHitTestMechanic::complete_from_runtime_mounting(UiMountedHitTestCompletionInput {
        frame: row.frame(),
        surface: row.surface(),
        binding: row.binding(),
        mounted_instance: row.mounted_instance(),
        node_receipt: row.node_receipt(),
        bounds: bounds(allocation, space),
        clip_bounds: bounds(clip, space),
        order: row.order(),
    })
}

fn bounds(
    [x, y, width, height]: [f32; 4],
    coordinate_space: UiMountedCoordinateSpace,
) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .unwrap()
}

fn replacement(
    surface: UiSemanticSurfaceIdentity,
) -> crate::mounting::UiSurfaceBindingIdentityView {
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol must admit")
    };
    let request = UiHostSurfaceRegistrationRequest::from_runtime(UiHostSurfaceRegistrationInput {
        host_session_identity: 1,
        semantic_surface_identity: surface,
        host_surface_identity: UiHostSurfaceIdentity::mint_unbound().unwrap(),
        binding_generation: binding,
        protocol,
        capability_generation: WorthUiHostCapabilityObservationGeneration::new(1),
        capability_profile_digest: 1,
        presentation_mode: UiHostSurfacePresentationMode::NativeDisplay,
    });
    crate::mounting::UiSurfaceBindingIdentityView::new(
        crate::mounting::identity_view::UiSurfaceBindingConstruction {
            request,
            binding_generation: binding,
            baseline: request.baseline_identity(),
            profile: crate::mounting::UiSurfaceBindingProfile::new(
                1000,
                crate::mounting::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        },
    )
}
