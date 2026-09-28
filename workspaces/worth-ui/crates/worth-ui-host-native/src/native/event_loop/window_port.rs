use std::sync::Arc;

use winit::dpi::LogicalSize;
#[cfg(feature = "certification-support")]
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::native::{UiNativeOwnedResource, UiNativeResourceClass, UiNativeResourceRegistry};
use crate::UiNativeWindowConfiguration;

/// Contractual window-opening boundary. It returns only the owned OS window;
/// runtime composition remains responsible for lifecycle settlement.
pub(super) trait UiNativeWindowPort {
    fn open(
        event_loop: &ActiveEventLoop,
        configuration: &UiNativeWindowConfiguration,
    ) -> Result<UiNativeOpenedWindow, UiNativeWindowPortDenial>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiNativeWindowPortDenial {
    Creation,
}

pub(super) struct UiWinitNativeWindowPort;

pub(super) struct UiNativeOpenedWindow {
    window: Arc<Window>,
    crossing_count: u8,
}

pub(crate) struct UiNativeOwnedWindow(UiNativeOwnedResource<Arc<Window>>);

impl UiNativeOpenedWindow {
    pub(super) fn register(
        self,
        registry: &mut UiNativeResourceRegistry,
    ) -> Result<(UiNativeOwnedWindow, u8), ()> {
        UiNativeOwnedResource::register(self.window, UiNativeResourceClass::Window, registry)
            .map(|window| (UiNativeOwnedWindow(window), self.crossing_count))
            .map_err(drop)
    }
}

impl UiNativeOwnedWindow {
    pub(crate) fn client_physical_size(&self) -> [u32; 2] {
        let size = self.inner_size();
        [size.width, size.height]
    }

    #[cfg(feature = "certification-support")]
    pub(crate) fn request_client_physical_size(&self, extent: [u32; 2]) {
        let _ = self
            .0
            .request_inner_size(PhysicalSize::new(extent[0], extent[1]));
    }

    pub(crate) fn close(self, registry: &mut UiNativeResourceRegistry) {
        self.0.close(registry);
    }
}

impl std::ops::Deref for UiNativeOwnedWindow {
    type Target = Arc<Window>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UiNativeWindowPort for UiWinitNativeWindowPort {
    fn open(
        event_loop: &ActiveEventLoop,
        configuration: &UiNativeWindowConfiguration,
    ) -> Result<UiNativeOpenedWindow, UiNativeWindowPortDenial> {
        let [width, height] = configuration.initial_logical_size();
        let mut attributes = WindowAttributes::default()
            .with_title(configuration.title())
            .with_transparent(
                crate::native_profile::WORTH_UI_NATIVE_CLIENT_BACKGROUND
                    .requests_transparent_window(),
            )
            .with_inner_size(LogicalSize::new(f64::from(width), f64::from(height)));
        let minimum = configuration
            .minimum_logical_size()
            .map(|[width, height]| LogicalSize::new(f64::from(width), f64::from(height)));
        if let Some(minimum) = minimum {
            attributes = attributes.with_min_inner_size(minimum);
        }
        event_loop
            .create_window(attributes)
            .inspect(|window| trace_minimum(configuration, window.scale_factor()))
            .map(Arc::new)
            .map(|window| UiNativeOpenedWindow {
                window,
                crossing_count: 1,
            })
            .map_err(|_| UiNativeWindowPortDenial::Creation)
    }
}

/// Traces the least client extent the window allows at `scale_factor`, if
/// the configuration sets one. The window keeps its minimum in logical size,
/// so the traced extent changes with the scale.
pub(super) fn trace_minimum(configuration: &UiNativeWindowConfiguration, scale_factor: f64) {
    if let Some([width, height]) = configuration.minimum_logical_size() {
        let least =
            LogicalSize::new(f64::from(width), f64::from(height)).to_physical::<u32>(scale_factor);
        crate::native::resize_trace::minimum([least.width, least.height]);
    }
}
