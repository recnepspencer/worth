//! One X connection per observer process. Every request that can fail is
//! mapped to the contract's typed failure at the call site that owns the
//! meaning, the way `windows/` maps each Win32 result; this module names the
//! connection, the screen, the atoms and the pixel layout the observer needs,
//! and qualifies the server once.
use std::fmt::Display;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, Atom, AtomEnum, ConnectionExt as _, Screen, Window};
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

use crate::native_platform::NativePlatformFailure;

use super::environment;
use super::pixel_layout::PixelLayout;
use super::scale;

/// Why the display could not be qualified. `Clone` so the process-wide
/// `OnceLock` can hand the same answer to every caller of `certified()`.
#[derive(Clone, Debug)]
pub(super) enum X11QualificationDenial {
    Environment(String),
    Dpi(String),
}

impl From<X11QualificationDenial> for NativePlatformFailure {
    fn from(denial: X11QualificationDenial) -> Self {
        match denial {
            X11QualificationDenial::Environment(detail) => Self::EnvironmentQualification(detail),
            X11QualificationDenial::Dpi(detail) => Self::DpiAwareness(detail),
        }
    }
}

pub(super) struct X11Observation {
    connection: RustConnection,
    screen: usize,
    atoms: Atoms,
    pixel_layout: PixelLayout,
    dpi: u32,
}

#[derive(Clone, Copy)]
pub(super) struct Atoms {
    pub(super) net_wm_pid: Atom,
    pub(super) wm_protocols: Atom,
    pub(super) wm_delete_window: Atom,
    pub(super) net_supporting_wm_check: Atom,
}

impl X11Observation {
    /// Connects to `$DISPLAY` and qualifies the server for the profile:
    /// XTEST 2.2, a 24-bit TrueColor root whose pixel layout is exactly
    /// decodable and that holds the product window at the certified scale, the
    /// certified scale row, and no window manager (the observer plays the
    /// ICCCM manager's part itself, so a real one would race it).
    pub(super) fn connect_qualified() -> Result<Self, X11QualificationDenial> {
        let (connection, screen) = x11rb::connect(None).map_err(environment_denial)?;
        let atoms = Atoms {
            net_wm_pid: intern(&connection, b"_NET_WM_PID")?,
            wm_protocols: intern(&connection, b"WM_PROTOCOLS")?,
            wm_delete_window: intern(&connection, b"WM_DELETE_WINDOW")?,
            net_supporting_wm_check: intern(&connection, b"_NET_SUPPORTING_WM_CHECK")?,
        };
        let xtest = connection
            .xtest_get_version(
                environment::REQUIRED_XTEST_VERSION.0,
                environment::REQUIRED_XTEST_VERSION.1,
            )
            .map_err(environment_denial)?
            .reply()
            .map_err(environment_denial)?;
        environment::qualify_xtest_version(xtest.major_version, xtest.minor_version)
            .map_err(X11QualificationDenial::Environment)?;
        let root_screen = &connection.setup().roots[screen];
        environment::qualify_root_depth(root_screen.root_depth)
            .map_err(X11QualificationDenial::Environment)?;
        environment::qualify_screen_extent(
            root_screen.width_in_pixels,
            root_screen.height_in_pixels,
        )
        .map_err(X11QualificationDenial::Environment)?;
        let dpi = scale::qualify(&connection, screen, root_screen)?;
        let pixel_layout = root_pixel_layout(&connection, root_screen)
            .map_err(X11QualificationDenial::Environment)?;
        let observation = Self {
            connection,
            screen,
            atoms,
            pixel_layout,
            dpi,
        };
        observation.qualify_no_window_manager()?;
        Ok(observation)
    }

    pub(super) fn connection(&self) -> &RustConnection {
        &self.connection
    }

    pub(super) fn screen(&self) -> &Screen {
        &self.connection.setup().roots[self.screen]
    }

    pub(super) fn root(&self) -> Window {
        self.screen().root
    }

    pub(super) fn atoms(&self) -> Atoms {
        self.atoms
    }

    pub(super) fn pixel_layout(&self) -> PixelLayout {
        self.pixel_layout
    }

    /// The qualified dpi: the certified scale row as winit will resolve it.
    pub(super) fn dpi(&self) -> u32 {
        self.dpi
    }

    /// Re-qualifies every stage of the scale, for observations that must
    /// prove the dpi the binding saw still holds.
    pub(super) fn qualify_dpi(&self) -> Result<u32, X11QualificationDenial> {
        scale::qualify(&self.connection, self.screen, self.screen())
    }

    /// `_NET_WM_PID` as winit sets it (`x11/window.rs`), or `None` when the
    /// window carries no such property; a window without it is never a match.
    pub(super) fn window_process_id(
        &self,
        window: Window,
    ) -> Result<Option<u32>, NativePlatformFailure> {
        let reply = self
            .connection
            .get_property(
                false,
                window,
                self.atoms.net_wm_pid,
                AtomEnum::CARDINAL,
                0,
                1,
            )
            .map_err(enumeration_failure)?
            .reply();
        match reply {
            Ok(reply) => Ok(reply.value32().and_then(|mut values| values.next())),
            // The window vanished between enumeration and the read: not a match.
            Err(x11rb::errors::ReplyError::X11Error(error))
                if error.error_kind == x11rb::protocol::ErrorKind::Window =>
            {
                Ok(None)
            }
            Err(error) => Err(enumeration_failure(error)),
        }
    }

    /// An EWMH manager announces itself through `_NET_SUPPORTING_WM_CHECK`
    /// on the root. The lane's Xvfb runs none; a manager present would
    /// reparent, iconify and restack behind the observer's back.
    fn qualify_no_window_manager(&self) -> Result<(), X11QualificationDenial> {
        let reply = self
            .connection
            .get_property(
                false,
                self.root(),
                self.atoms.net_supporting_wm_check,
                AtomEnum::WINDOW,
                0,
                1,
            )
            .map_err(environment_denial)?
            .reply()
            .map_err(environment_denial)?;
        match reply.value32().and_then(|mut values| values.next()) {
            None => Ok(()),
            Some(manager) => Err(X11QualificationDenial::Environment(format!(
                "a window manager owns the display (_NET_SUPPORTING_WM_CHECK={manager:#x})"
            ))),
        }
    }
}

fn root_pixel_layout(connection: &RustConnection, screen: &Screen) -> Result<PixelLayout, String> {
    let setup = connection.setup();
    let visual = screen
        .allowed_depths
        .iter()
        .filter(|depth| depth.depth == screen.root_depth)
        .flat_map(|depth| depth.visuals.iter())
        .find(|visual| visual.visual_id == screen.root_visual)
        .ok_or_else(|| {
            format!(
                "root visual {:#x} is not listed at the root depth",
                screen.root_visual
            )
        })?;
    let format = setup
        .pixmap_formats
        .iter()
        .find(|format| format.depth == screen.root_depth)
        .ok_or_else(|| format!("no pixmap format at root depth {}", screen.root_depth))?;
    PixelLayout::qualified(visual, format, setup.image_byte_order)
}

fn intern(connection: &RustConnection, name: &[u8]) -> Result<Atom, X11QualificationDenial> {
    Ok(connection
        .intern_atom(false, name)
        .map_err(environment_denial)?
        .reply()
        .map_err(environment_denial)?
        .atom)
}

fn environment_denial(error: impl Display) -> X11QualificationDenial {
    X11QualificationDenial::Environment(error.to_string())
}

pub(super) fn enumeration_failure(error: impl Display) -> NativePlatformFailure {
    NativePlatformFailure::WindowEnumeration(error.to_string())
}

pub(super) fn actuation_failure(error: impl Display) -> NativePlatformFailure {
    NativePlatformFailure::WindowActuation(error.to_string())
}

pub(super) fn capture_failure(error: impl Display) -> NativePlatformFailure {
    NativePlatformFailure::ClientCapture(error.to_string())
}

pub(super) fn input_failure(error: impl Display) -> NativePlatformFailure {
    NativePlatformFailure::InputDelivery(error.to_string())
}

/// Keeps `xproto` in scope for sibling modules through one import site.
pub(super) use xproto::Drawable;
