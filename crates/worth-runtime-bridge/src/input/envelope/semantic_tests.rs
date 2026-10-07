use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey,
    AuthoritativeAspectChangeKind, CanonicalFieldPath, FieldKey,
};

use super::{
    BridgeAspectChangeWideningCause, BridgeSemanticAspectChange, BridgeSemanticAspectChangeBreadth,
};

fn field_path() -> CanonicalFieldPath {
    CanonicalFieldPath::new([FieldKey::new("rank").unwrap()]).unwrap()
}

fn binding() -> AspectBinding {
    AspectBinding::EntityField {
        field: FieldKey::new("rank").unwrap(),
    }
}

fn widened(cause: BridgeAspectChangeWideningCause) -> BridgeSemanticAspectChange {
    BridgeSemanticAspectChange::from_declared_authoritative_widening(
        AspectKey::new("Portfolio.Facts").unwrap(),
        AspectIdentity(7),
        AspectContractRevision(3),
        binding(),
        AuthoritativeAspectChangeKind::FieldSet,
        Some(field_path()),
        cause,
    )
}

#[test]
fn bridge_owns_effective_change_breadth_after_declared_widening() {
    let exact = BridgeSemanticAspectChange::from_authoritative_publication(
        AspectKey::new("Portfolio.Facts").unwrap(),
        AspectIdentity(7),
        AspectContractRevision(3),
        binding(),
        AuthoritativeAspectChangeKind::FieldSet,
        Some(field_path()),
    );
    assert_eq!(
        exact.effective_breadth(),
        BridgeSemanticAspectChangeBreadth::ExactField
    );
    assert_eq!(exact.effective_field_path(), Some(&field_path()));

    for cause in [
        BridgeAspectChangeWideningCause::FieldToWholeAspect,
        BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect,
    ] {
        let change = widened(cause);
        assert_eq!(
            change.effective_breadth(),
            BridgeSemanticAspectChangeBreadth::WholeAspect
        );
        assert_eq!(change.effective_field_path(), None);
    }
    assert_eq!(
        widened(BridgeAspectChangeWideningCause::AspectToEntity).effective_breadth(),
        BridgeSemanticAspectChangeBreadth::Entity
    );
    assert_eq!(
        widened(BridgeAspectChangeWideningCause::SurfaceBroadening).effective_breadth(),
        BridgeSemanticAspectChangeBreadth::Surface
    );
}

#[test]
fn bridge_owns_whole_aspect_to_field_change_kind_intersection() {
    let whole = BridgeSemanticAspectChange::from_authoritative_publication(
        AspectKey::new("Portfolio.Facts").unwrap(),
        AspectIdentity(7),
        AspectContractRevision(3),
        binding(),
        AuthoritativeAspectChangeKind::WholeAspectSet,
        None,
    );
    assert!(whole.intersects_relevant_change(AuthoritativeAspectChangeKind::FieldSet));
    assert!(whole.intersects_relevant_change(AuthoritativeAspectChangeKind::FieldClear));
    assert!(
        !whole.intersects_relevant_change(AuthoritativeAspectChangeKind::RelationSourceEndpoint)
    );
}

#[test]
fn bridge_carries_eight_segment_scope_alongside_field_path() {
    let path = worth_signal::facade::ScopePath::new(
        [
            "rates", "usd", "swaps", "book", "desk", "curve", "tenor", "5y",
        ]
        .map(str::to_owned),
    )
    .unwrap();
    let region = worth_signal::facade::ChangedRegion::exact(path.clone());
    let change = BridgeSemanticAspectChange::from_authoritative_publication(
        AspectKey::new("Portfolio.Facts").unwrap(),
        AspectIdentity(7),
        AspectContractRevision(3),
        binding(),
        AuthoritativeAspectChangeKind::FieldSet,
        Some(field_path()),
    )
    .with_scope_change(region.clone());
    assert_eq!(change.effective_field_path(), Some(&field_path()));
    assert_eq!(change.scope_change(), Some(&region));
    assert_eq!(change.scope_change().unwrap().path(), &path);
    assert_eq!(change.scope_change().unwrap().path().depth(), 8);
}
