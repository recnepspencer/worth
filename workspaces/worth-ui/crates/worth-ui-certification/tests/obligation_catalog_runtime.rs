use worth_ui::facade::obligations::{UiObligationFamily, UiObligationFamilyCatalog};

#[test]
fn obligation_family_catalog_is_closed_and_typed_without_a_catch_all_family() {
    let families = UiObligationFamilyCatalog::closed().families();

    assert_eq!(
        families,
        &[
            UiObligationFamily::StructuralLegality,
            UiObligationFamily::ParticipationLegality,
            UiObligationFamily::SlotContract,
            UiObligationFamily::MeasurementRequirement,
            UiObligationFamily::QueryBindingRequirement,
            UiObligationFamily::IntentOperabilityRequirement,
            UiObligationFamily::PortalHostRequirement,
            UiObligationFamily::FocusRouteRequirement,
            UiObligationFamily::MotionSupportRequirement,
            UiObligationFamily::ScrollRoutingRequirement,
            UiObligationFamily::SelectionStateRequirement,
            UiObligationFamily::CommandRouteRequirement,
            UiObligationFamily::AccessibilityRequirement,
            UiObligationFamily::HostCapabilityRequirement,
            UiObligationFamily::DiagnosticSurfaceRequirement,
        ]
    );
    assert_eq!(families.len(), 15);
    assert!(!families.iter().enumerate().any(|(index, family)| {
        families
            .iter()
            .skip(index + 1)
            .any(|candidate| candidate == family)
    }));
}
