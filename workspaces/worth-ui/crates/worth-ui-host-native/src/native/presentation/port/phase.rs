use worth_ui_host_contract::UiHostPresentationCostReport;

use crate::native::UiNativePresentationAccess;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeSurfaceAcquireFailure {
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
    DeviceLost,
}

#[must_use]
pub(super) struct UiNativePreparedPresentation {
    cost: UiHostPresentationCostReport,
}

#[must_use]
pub(super) struct UiNativeSurfaceAcquiredPresentation {
    output: wgpu::SurfaceTexture,
    cost: UiHostPresentationCostReport,
}

#[must_use]
pub(super) struct UiNativeEncodedPresentation {
    output: wgpu::SurfaceTexture,
    commands: wgpu::CommandBuffer,
    readback: wgpu::Buffer,
    cost: UiHostPresentationCostReport,
}

#[must_use]
pub(super) struct UiNativeSubmittedPresentation {
    output: wgpu::SurfaceTexture,
    readback: wgpu::Buffer,
    cost: UiHostPresentationCostReport,
}

#[must_use]
pub(super) struct UiNativePresentHandoff {
    readback: wgpu::Buffer,
    cost: UiHostPresentationCostReport,
}

impl UiNativePreparedPresentation {
    pub(super) const fn new(cost: UiHostPresentationCostReport) -> Self {
        Self { cost }
    }

    pub(super) fn acquire(
        self,
        graphics: &UiNativePresentationAccess,
    ) -> Result<UiNativeSurfaceAcquiredPresentation, UiNativeSurfaceAcquireFailure> {
        if graphics.device_lost() {
            return Err(UiNativeSurfaceAcquireFailure::DeviceLost);
        }
        if graphics.surface_suspended() {
            return Err(UiNativeSurfaceAcquireFailure::Occluded);
        }
        let output = match graphics.surface().get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output) => output,
            wgpu::CurrentSurfaceTexture::Suboptimal(_) => {
                return Err(UiNativeSurfaceAcquireFailure::Outdated);
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err(UiNativeSurfaceAcquireFailure::Timeout);
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(UiNativeSurfaceAcquireFailure::Occluded);
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return Err(UiNativeSurfaceAcquireFailure::Outdated);
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(UiNativeSurfaceAcquireFailure::Lost);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(UiNativeSurfaceAcquireFailure::Validation);
            }
        };
        Ok(UiNativeSurfaceAcquiredPresentation {
            output,
            cost: self.cost,
        })
    }
}

impl UiNativeSurfaceAcquiredPresentation {
    pub(super) fn encode_with(
        self,
        encode: impl FnOnce(&wgpu::Texture) -> (wgpu::CommandBuffer, wgpu::Buffer),
    ) -> UiNativeEncodedPresentation {
        let (commands, readback) = encode(&self.output.texture);
        UiNativeEncodedPresentation {
            output: self.output,
            commands,
            readback,
            cost: self.cost,
        }
    }
}

impl UiNativeEncodedPresentation {
    pub(super) fn submit(self, queue: &wgpu::Queue) -> UiNativeSubmittedPresentation {
        queue.submit([self.commands]);
        UiNativeSubmittedPresentation {
            output: self.output,
            readback: self.readback,
            cost: self.cost,
        }
    }
}

impl UiNativeSubmittedPresentation {
    pub(super) fn hand_off(self) -> UiNativePresentHandoff {
        self.output.present();
        UiNativePresentHandoff {
            readback: self.readback,
            cost: self.cost,
        }
    }
}

impl UiNativePresentHandoff {
    pub(super) fn into_parts(self) -> (wgpu::Buffer, UiHostPresentationCostReport) {
        (self.readback, self.cost)
    }
}
