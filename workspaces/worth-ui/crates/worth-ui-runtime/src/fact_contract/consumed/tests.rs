use super::*;
use crate::declaration::{UiAspectName, UiAspectSemanticSlice};
use crate::fact_contract::{UiProducedFactFamily, UiSubsystemConsumedFactRule};

#[test]
fn subsystem_rules_intersect_only_declaration_owned_aspect_families() {
    let background = UiAspectName::from_semantic_slice(UiAspectSemanticSlice::AppearanceBackground);
    let text = UiAspectName::from_semantic_slice(UiAspectSemanticSlice::ContentText);
    let root = UiAspectName::from_semantic_slice(UiAspectSemanticSlice::StructureProductRoot);

    assert!(UiConsumedFactContract::declared_aspect(
        UiProducedFactFamily::HostDeviceScale,
        background
    )
    .is_some());
    assert!(UiConsumedFactContract::declared_aspect(
        UiProducedFactFamily::HostDeviceScale,
        text.clone()
    )
    .is_none());
    assert!(UiConsumedFactContract::declared_aspect(UiProducedFactFamily::Query, text).is_some());
    assert!(UiConsumedFactContract::declared_aspect(UiProducedFactFamily::Query, root).is_some());
}

#[test]
fn every_non_authored_fact_family_has_an_explicit_subsystem_rule() {
    for family in [
        UiProducedFactFamily::HostViewport,
        UiProducedFactFamily::HostDeviceScale,
        UiProducedFactFamily::PointerPresenceTarget,
        UiProducedFactFamily::Measurement,
        UiProducedFactFamily::Query,
        UiProducedFactFamily::CommittedScrollExtent,
        UiProducedFactFamily::CommittedPortalAnchor,
        UiProducedFactFamily::CommittedFocus,
        UiProducedFactFamily::CommittedSelection,
        UiProducedFactFamily::CommittedMotionTrack,
        UiProducedFactFamily::CommittedCommandRoute,
    ] {
        assert!(UiSubsystemConsumedFactRule::all().any(|rule| rule.fact_family() == family));
    }
    assert!(!UiSubsystemConsumedFactRule::all()
        .any(|rule| rule.fact_family() == UiProducedFactFamily::AuthoredSource));
}
