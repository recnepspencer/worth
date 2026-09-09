use super::tests::row;
use super::*;
use worth_ui_host_contract::*;

#[test]
fn hit_change_queries_do_not_enter_an_unchanged_dense_binding() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let changed_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let untouched_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let changed = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut previous = UiPresentedHitIndex::default();
    previous.replace_base(
        changed,
        Some(row(
            issuer,
            changed_binding,
            surface,
            changed,
            1,
            [0.0, 0.0, 10.0, 10.0],
        )),
    );
    for n in 0..300 {
        let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        previous.replace_base(
            instance,
            Some(row(
                issuer,
                untouched_binding,
                surface,
                instance,
                n,
                [0.0, 0.0, 10.0, 10.0],
            )),
        );
    }
    let mut current = previous.clone();
    current.replace_base(changed, None);
    let changes = UiPresentedHitChanges::between(previous, current);
    assert_eq!(changes.changed_count(), 1);
    assert!(changes.comparison_steps() < 128);
    assert_eq!(
        changes.affects(untouched_binding, [5.0, 5.0], None),
        Ok((false, UiHitTestSpatialWork::default()))
    );
    let (affected, work) = changes.affects(changed_binding, [5.0, 5.0], None).unwrap();
    assert!(affected && work.node_visits > 0);
}

#[test]
fn hit_changes_distinguish_receipt_refresh_order_change_and_local_query_exhaustion() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let next_issuer =
        UiMountedNodeReceiptIssuer::mint_for(UiMountedFrameIdentity::mint_unbound().unwrap())
            .unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut previous = UiPresentedHitIndex::default();
    previous.replace_base(
        instance,
        Some(row(
            issuer,
            binding,
            surface,
            instance,
            1,
            [0.0, 0.0, 10.0, 10.0],
        )),
    );
    let mut current = previous.clone();
    current.replace_base(
        instance,
        Some(row(
            next_issuer,
            binding,
            surface,
            instance,
            1,
            [0.0, 0.0, 10.0, 10.0],
        )),
    );
    let changes = UiPresentedHitChanges::between(previous.clone(), current.clone());
    assert_eq!(changes.changed_count(), 0);
    assert_eq!(
        changes.affects(binding, [5.0, 5.0], Some(instance)),
        Ok((false, UiHitTestSpatialWork::default()))
    );
    current.replace_base(
        instance,
        Some(row(
            next_issuer,
            binding,
            surface,
            instance,
            2,
            [0.0, 0.0, 10.0, 10.0],
        )),
    );
    let changes = UiPresentedHitChanges::between(previous.clone(), current.clone());
    assert!(
        changes.affects(binding, [5.0, 5.0], None).unwrap().0,
        "order alone changes targeting"
    );
    for rank in 3..303 {
        let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        current.replace_base(
            instance,
            Some(row(
                issuer,
                binding,
                surface,
                instance,
                rank,
                [20.0, 20.0, 10.0, 10.0],
            )),
        );
    }
    let changes = UiPresentedHitChanges::between(previous, current);
    assert!(
        matches!(changes.affects(binding, [25.0, 25.0], None), Err(UiPresentedHitQueryDenial::CandidateBudget { work }) if work.region_tests > 256)
    );
}
