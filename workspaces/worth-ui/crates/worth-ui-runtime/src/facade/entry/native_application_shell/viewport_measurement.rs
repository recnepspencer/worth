use super::viewport_extent::{UiNativeViewportBasis, UiNativeViewportSettlement};
use super::WorthUiNativeApplicationShell;

struct UiPendingNativeViewportMeasurements {
    settlement: UiNativeViewportSettlement,
    capability: crate::facade::WorthUiHostMeasurementCapability,
    inputs: Vec<crate::facade::WorthUiHostMeasurementSessionInput>,
}

impl WorthUiNativeApplicationShell {
    /// Return the latest observed client viewport in logical host-surface
    /// coordinates for application-owned native layout.
    pub fn native_layout_viewport(&self) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
        let basis = self.viewport.observed()?;
        if basis.scale_factor_milli == 0 {
            return None;
        }
        let scale = basis.scale_factor_milli as f32;
        worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
            worth_ui_host_contract::UiMountedCanonicalBoxInput {
                x: 0.0,
                y: 0.0,
                width: basis.client_physical_extent[0] as f32 * 1_000.0 / scale,
                height: basis.client_physical_extent[1] as f32 * 1_000.0 / scale,
                coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
            },
        )
        .ok()
    }

    /// Reports whether host-owned viewport evidence requires one successor
    /// presentation. The extent and measurement authority remain inside the
    /// runtime; applications use this only to avoid idle duplicate frames.
    pub fn native_viewport_presentation_pending(&self) -> bool {
        self.viewport.owed().is_some()
    }

    pub(crate) fn observe_native_viewport_readiness(
        &mut self,
        client_physical_extent: [u32; 2],
        scale_factor_milli: u32,
        submit_successor: bool,
    ) {
        let basis = UiNativeViewportBasis {
            client_physical_extent,
            scale_factor_milli,
            binding: self.binding,
        };
        self.viewport.observe(basis, || {
            submit_successor && !self.session.viewport_measurement_witnesses().is_empty()
        });
    }

    pub(super) fn observe_native_viewport_binding_successor(&mut self, submit_successor: bool) {
        let Some(observed) = self.viewport.observed() else {
            return;
        };
        self.observe_native_viewport_readiness(
            observed.client_physical_extent,
            self.scale_factor_milli,
            submit_successor,
        );
    }

    pub(super) fn settle_pending_native_viewport_measurements(
        &mut self,
    ) -> Result<
        (),
        super::super::mounted_application_presentation::UiMountedHostMeasurementSettlementStop,
    > {
        let Some(pending) = self.pending_native_viewport_measurements() else {
            return Ok(());
        };
        self.session
            .settle_mounted_host_measurements(Some((pending.capability, pending.inputs)))?;
        self.viewport.measure(pending.settlement);
        Ok(())
    }

    fn pending_native_viewport_measurements(&self) -> Option<UiPendingNativeViewportMeasurements> {
        let settlement = self.viewport.prepare()?;
        let viewport_measurement_authority = self.session.viewport_measurement_witnesses();
        if viewport_measurement_authority.is_empty() {
            return None;
        }
        let capability = self.session.host_measurement_capability();
        let assumptions = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
            capability.capability_report(),
            1,
            2,
            3,
            4,
        );
        let inputs = viewport_measurement_authority
            .iter()
            .copied()
            .map(|authority| {
                crate::facade::WorthUiHostMeasurementSessionInput::new(
                    authority.request_identity(),
                    worth_ui_host_contract::UiMeasurementEvidenceFamily::ViewportExtent,
                    crate::host::UiHostMeasurementNeed::ViewportExtent(
                        worth_ui_host_contract::UiViewportExtentRequest,
                    ),
                    authority.evidence_generation(),
                    crate::host::UiHostMeasurementNormalizationContext::viewport_logical_exact(
                        assumptions,
                    ),
                )
            })
            .collect::<Vec<_>>();
        Some(UiPendingNativeViewportMeasurements {
            settlement,
            capability,
            inputs,
        })
    }
}

#[cfg(test)]
#[path = "viewport_measurement/tests.rs"]
mod tests;
