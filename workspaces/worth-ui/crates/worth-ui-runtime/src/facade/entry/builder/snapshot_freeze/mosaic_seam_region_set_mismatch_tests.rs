use crate::capability::{
    CapabilityDiagnosticCode, MosaicRegionKindDescriptor, MosaicRegionKindId, MosaicRegionRole,
    MosaicSeamPaintContract, RegistryFamily,
};

#[test]
fn freeze_reports_and_excludes_a_seam_contract_with_a_mismatched_region_set() {
    let primary = MosaicRegionKindId::new("region.primary").unwrap();
    let auxiliary = MosaicRegionKindId::new("region.auxiliary").unwrap();
    let seam = MosaicSeamPaintContract::admit([primary.clone()], [], [], []).unwrap();

    let report = super::CapabilityRegistrationBuilder::new()
        .register_mosaic_region_kind(MosaicRegionKindDescriptor::new(
            primary,
            MosaicRegionRole::primary(),
        ))
        .register_mosaic_region_kind(MosaicRegionKindDescriptor::new(
            auxiliary,
            MosaicRegionRole::auxiliary(),
        ))
        .register_mosaic_seam_paint_contract(seam)
        .unwrap()
        .freeze_with_registration_report();

    assert!(report.registration_diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == CapabilityDiagnosticCode::MosaicSeamPaintRegionSetMismatch
    }));
    assert_eq!(
        report
            .accepted_snapshot()
            .freeze_report()
            .registry_family_width(RegistryFamily::MosaicSeamPaint),
        Some(0)
    );
    assert!(report
        .accepted_snapshot()
        .mosaic_regions()
        .seam_paint()
        .is_none());
}
