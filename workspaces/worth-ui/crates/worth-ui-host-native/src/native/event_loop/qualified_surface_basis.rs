//! Certification-triggered presentation-surface basis succession.

use winit::event_loop::ActiveEventLoop;

#[cfg(feature = "certification-support")]
use super::UiNativeEventLoopRunDenial;
use super::{UiNativeEventLoopApplication, UiNativeEventLoopClient};
#[cfg(feature = "certification-support")]
use crate::native::UiNativeHostState;

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    #[cfg(feature = "certification-support")]
    pub(super) fn apply_qualified_surface_basis_successor(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> bool {
        let successor = {
            let mut state = self.shared.borrow_mut();
            let completed = state.retained_frame_observations.len() as u64;
            state.qualification.take_surface_basis_successor(completed)
        };
        let Some(successor) = successor else {
            return false;
        };
        let replacement = {
            let mut state = self.shared.borrow_mut();
            let UiNativeHostState {
                presentation_owners,
                resources,
                window,
                ..
            } = &mut *state;
            presentation_owners.as_mut().zip(window.as_ref()).map_or(
                Err(()),
                |(crate::native::UiNativePresentationOwners { device, surface }, window)| {
                    let current_scale = surface.state().scale_factor();
                    let current_extent = surface.state().extent();
                    let (scale, extent) =
                        successor_basis(current_scale, current_extent, successor.change())?;
                    window.request_client_physical_size(extent);
                    crate::native::lifecycle::rebind_surface_scale(
                        device, surface, scale, extent, resources,
                    )
                },
            )
        };
        if replacement != Ok(true) {
            self.fail(event_loop, UiNativeEventLoopRunDenial::GraphicsPreparation);
            return true;
        }
        let cause = match successor.change() {
            crate::qualification::UiNativeQualificationSurfaceBasisChange::ClientPhysicalWidthDelta(_) => {
                crate::native::UiNativeRecoveryCause::Resize
            }
            crate::qualification::UiNativeQualificationSurfaceBasisChange::DpiScaleMultiplierMilli(_) => {
                crate::native::UiNativeRecoveryCause::Dpi
            }
        };
        {
            let mut state = self.shared.borrow_mut();
            state.require_surface_reconstruction(cause);
        }
        self.commit_visible_surface_readiness(event_loop);
        false
    }

    #[cfg(not(feature = "certification-support"))]
    pub(super) fn apply_qualified_surface_basis_successor(
        &mut self,
        _event_loop: &ActiveEventLoop,
    ) -> bool {
        false
    }
}

#[cfg(feature = "certification-support")]
fn successor_basis(
    current_scale: f64,
    current_extent: [u32; 2],
    change: crate::qualification::UiNativeQualificationSurfaceBasisChange,
) -> Result<(f64, [u32; 2]), ()> {
    use crate::qualification::UiNativeQualificationSurfaceBasisChange as Change;

    match change {
        Change::ClientPhysicalWidthDelta(delta) => {
            let width = i64::from(current_extent[0]) + i64::from(delta);
            let width = u32::try_from(width).map_err(|_| ())?;
            Ok((current_scale, [width, current_extent[1]]))
        }
        Change::DpiScaleMultiplierMilli(multiplier) => {
            let factor = f64::from(multiplier) / 1_000.0;
            let width = (f64::from(current_extent[0]) * factor).round();
            let height = (f64::from(current_extent[1]) * factor).round();
            if !(1.0..=f64::from(u32::MAX)).contains(&width)
                || !(1.0..=f64::from(u32::MAX)).contains(&height)
            {
                return Err(());
            }
            Ok((current_scale * factor, [width as u32, height as u32]))
        }
    }
}
