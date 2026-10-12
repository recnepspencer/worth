use super::*;

pub(super) fn target_dx_digest(public_surface: &EffectLifecyclePublicSurfaceInventory) -> String {
    let required = [
        EffectPublicSurfaceKind::CommonPathIntentAuthoring,
        EffectPublicSurfaceKind::WritebackCommonPath,
        EffectPublicSurfaceKind::InspectableLoweredPlan,
        EffectPublicSurfaceKind::DenialOrRebind,
        EffectPublicSurfaceKind::SupportDiscovery,
        EffectPublicSurfaceKind::BatchExecution,
        EffectPublicSurfaceKind::DiagnosticsEnvelope,
        EffectPublicSurfaceKind::ProductionCertification,
    ];
    hash_parts(
        &required
            .iter()
            .map(|kind| {
                let row = public_surface
                    .rows()
                    .iter()
                    .find(|row| row.surface_kind() == *kind)
                    .expect("required public surface row should exist");
                format!(
                    "{}:{}:{}",
                    kind.as_str(),
                    row.entrypoint().unwrap_or("none"),
                    row.row_for_reporting()
                )
            })
            .collect::<Vec<_>>(),
    )
}
