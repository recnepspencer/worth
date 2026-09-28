//! Where a host presents its frames: a window's swapchain, or, for the
//! offscreen pass, a texture the host renders into exactly as it would into a
//! swapchain texture. The offscreen target has the swapchain's configured
//! extent, format and usage, so every stage before present runs unchanged;
//! presenting it only releases the texture.

use std::sync::Mutex;

pub(crate) enum UiWgpuPresentationTarget {
    Window(wgpu::Surface<'static>),
    Offscreen(Mutex<Option<wgpu::Texture>>),
}

/// The texture one frame renders into, and what presenting it means.
pub(crate) enum UiWgpuAcquiredTexture {
    Surface(wgpu::SurfaceTexture),
    Offscreen(wgpu::Texture),
}

impl UiWgpuPresentationTarget {
    pub(super) const fn offscreen() -> Self {
        Self::Offscreen(Mutex::new(None))
    }

    pub(super) fn configure(
        &self,
        device: &wgpu::Device,
        configuration: &wgpu::SurfaceConfiguration,
    ) {
        match self {
            Self::Window(surface) => surface.configure(device, configuration),
            // As a swapchain recreates its textures when configured, the next
            // acquisition allocates from this device and configuration, so a
            // texture of a lost device or an earlier format never survives.
            Self::Offscreen(texture) => {
                *texture
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            }
        }
    }
}

impl UiWgpuAcquiredTexture {
    /// Acquires the offscreen texture for a frame, allocating it only when
    /// the target was configured since the previous frame.
    pub(crate) fn offscreen(
        target: &Mutex<Option<wgpu::Texture>>,
        device: &wgpu::Device,
        configuration: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let mut texture = target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if texture.is_none() {
            *texture = Some(device.create_texture(&wgpu::TextureDescriptor {
                label: Some("worth-ui-offscreen-presentation-target"),
                size: wgpu::Extent3d {
                    width: configuration.width,
                    height: configuration.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: configuration.format,
                usage: configuration.usage,
                view_formats: &[],
            }));
        }
        Self::Offscreen(
            texture
                .clone()
                .expect("the offscreen target was allocated above"),
        )
    }

    pub(crate) fn texture(&self) -> &wgpu::Texture {
        match self {
            Self::Surface(output) => &output.texture,
            Self::Offscreen(texture) => texture,
        }
    }

    pub(crate) fn present(self) {
        match self {
            Self::Surface(output) => output.present(),
            Self::Offscreen(_) => {}
        }
    }
}
