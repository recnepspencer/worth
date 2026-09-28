use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

#[cfg(feature = "certification-support")]
use winit::dpi::PhysicalSize;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, Window, WindowAttributes, WindowId};

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

/// The window a host presents into: a platform window, or the client area
/// the offscreen pump stands in for one.
#[derive(Clone)]
pub(crate) enum UiNativeWindowHandle {
    Platform(Arc<Window>),
    Offscreen(Rc<UiNativeOffscreenWindow>),
}

/// An offscreen client area: its physical extent and scale, whether a new
/// extent awaits its `Resized` event, and whether the host has requested a
/// redraw the pump has not yet run.
pub(crate) struct UiNativeOffscreenWindow {
    extent: Cell<[u32; 2]>,
    resized: Cell<bool>,
    scale_factor: f64,
    redraw_requested: Cell<bool>,
}

pub(super) struct UiNativeOpenedWindow {
    window: UiNativeWindowHandle,
    crossing_count: u8,
}

pub(crate) struct UiNativeOwnedWindow(UiNativeOwnedResource<UiNativeWindowHandle>);

impl UiNativeWindowHandle {
    pub(crate) fn request_redraw(&self) {
        match self {
            Self::Platform(window) => window.request_redraw(),
            Self::Offscreen(window) => window.redraw_requested.set(true),
        }
    }
}

impl UiNativeOffscreenWindow {
    pub(crate) fn new(extent: [u32; 2], scale_factor: f64) -> Self {
        Self {
            extent: Cell::new(extent),
            resized: Cell::new(false),
            scale_factor,
            redraw_requested: Cell::new(false),
        }
    }

    /// Resizes the client area. As on a platform window, the host reads the
    /// new extent at once and learns of it from a `Resized` event the pump
    /// dispatches in its next turn.
    pub(crate) fn resize(&self, extent: [u32; 2]) {
        self.extent.set(extent);
        self.resized.set(true);
    }

    /// Consumes the extent awaiting its `Resized` event, if any.
    pub(crate) fn take_resize(&self) -> Option<[u32; 2]> {
        self.resized.replace(false).then(|| self.extent.get())
    }

    pub(crate) fn resize_pending(&self) -> bool {
        self.resized.get()
    }

    pub(crate) fn extent(&self) -> [u32; 2] {
        self.extent.get()
    }

    pub(crate) const fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub(crate) fn redraw_pending(&self) -> bool {
        self.redraw_requested.get()
    }

    /// Consumes the pending redraw request, if any.
    pub(crate) fn take_redraw(&self) -> bool {
        self.redraw_requested.replace(false)
    }

    /// Opens this client area as the host's window.
    pub(super) fn open(self: Rc<Self>) -> UiNativeOpenedWindow {
        UiNativeOpenedWindow {
            window: UiNativeWindowHandle::Offscreen(self),
            crossing_count: 1,
        }
    }
}

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
        match &*self.0 {
            UiNativeWindowHandle::Platform(window) => {
                let size = window.inner_size();
                [size.width, size.height]
            }
            UiNativeWindowHandle::Offscreen(window) => window.extent(),
        }
    }

    pub(crate) fn request_redraw(&self) {
        self.0.request_redraw();
    }

    /// A handle that requests redraws without borrowing the host state that
    /// owns this window.
    pub(crate) fn redraw_handle(&self) -> UiNativeWindowHandle {
        (*self.0).clone()
    }

    /// The platform's identity for this window; an offscreen client area
    /// has none, and receives no platform events.
    pub(crate) fn id(&self) -> Option<WindowId> {
        self.platform().map(|window| window.id())
    }

    pub(crate) fn set_cursor(&self, cursor: CursorIcon) {
        if let Some(window) = self.platform() {
            window.set_cursor(cursor);
        }
    }

    /// The client area's origin on the desktop; an offscreen client area is
    /// on no desktop.
    pub(crate) fn inner_position(&self) -> Option<PhysicalPosition<i32>> {
        self.platform()
            .and_then(|window| window.inner_position().ok())
    }

    /// The window graphics prepare their presentation target for.
    pub(crate) fn graphics_window(&self) -> crate::native::graphics::UiNativeGraphicsWindow {
        use crate::native::graphics::UiNativeGraphicsWindow;
        match &*self.0 {
            UiNativeWindowHandle::Platform(window) => {
                UiNativeGraphicsWindow::Platform(Arc::clone(window))
            }
            UiNativeWindowHandle::Offscreen(window) => UiNativeGraphicsWindow::Offscreen {
                extent: window.extent(),
                scale_factor: window.scale_factor(),
            },
        }
    }

    /// The platform window, which surfaces, pointer ports and captures need.
    pub(crate) fn platform(&self) -> Option<&Arc<Window>> {
        match &*self.0 {
            UiNativeWindowHandle::Platform(window) => Some(window),
            UiNativeWindowHandle::Offscreen(_) => None,
        }
    }

    #[cfg(feature = "certification-support")]
    pub(crate) fn request_client_physical_size(&self, extent: [u32; 2]) {
        match &*self.0 {
            UiNativeWindowHandle::Platform(window) => {
                let _ = window.request_inner_size(PhysicalSize::new(extent[0], extent[1]));
            }
            UiNativeWindowHandle::Offscreen(window) => window.resize(extent),
        }
    }

    pub(crate) fn close(self, registry: &mut UiNativeResourceRegistry) {
        self.0.close(registry);
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
            .map(|window| UiNativeOpenedWindow {
                window: UiNativeWindowHandle::Platform(Arc::new(window)),
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
