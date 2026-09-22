//! The scale the product will run at, qualified the way winit 0.30.13
//! resolves it on X11 (`x11/util/randr.rs`): `WINIT_X11_SCALE_FACTOR` wins,
//! then a settings manager's `Xft/DPI`, then `Xft.dpi` from the resource
//! database, then the screen's millimetre extent. The lane certifies one
//! scale row, so every stage must be absent or agree with it: an override or
//! a settings manager is a denial, `Xft.dpi` must name the qualified dpi
//! exactly (Xvfb reports 0 mm through randr, so winit would otherwise fall
//! to 1.0), and the screen's own millimetres must round to the same dpi so
//! the record's "dpi" means one thing.
use std::env;
use std::ffi::OsString;

use x11rb::protocol::xproto::{ConnectionExt as _, Screen};
use x11rb::resource_manager::{self, Database};
use x11rb::rust_connection::RustConnection;

use crate::native_platform::CERTIFIED_SCALE_MILLI;

use super::connection::X11QualificationDenial;

/// X's logical baseline; winit divides `Xft.dpi` by it.
const BASELINE_DPI: u32 = 96;
/// Xvfb is started with `-dpi 144` and RESOURCE_MANAGER carries `Xft.dpi: 144`.
pub(super) const QUALIFIED_DPI: u32 = BASELINE_DPI * CERTIFIED_SCALE_MILLI / 1_000;
const _: () = assert!(
    QUALIFIED_DPI * 1_000 == BASELINE_DPI * CERTIFIED_SCALE_MILLI,
    "the certified scale is not a whole dpi"
);
const SCALE_OVERRIDE_VARIABLE: &str = "WINIT_X11_SCALE_FACTOR";
const XFT_DPI_RESOURCE: &str = "Xft.dpi";

/// Every stage winit consults, in winit's order; the qualified dpi.
pub(super) fn qualify(
    connection: &RustConnection,
    screen_index: usize,
    screen: &Screen,
) -> Result<u32, X11QualificationDenial> {
    qualify_no_override(env::var_os(SCALE_OVERRIDE_VARIABLE))
        .map_err(X11QualificationDenial::Dpi)?;
    qualify_no_settings_manager(connection, screen_index)?;
    let database = resource_manager::new_from_default(connection).map_err(dpi_denial)?;
    qualify_xft_dpi(&database).map_err(X11QualificationDenial::Dpi)?;
    qualify_screen_dpi(screen).map_err(X11QualificationDenial::Dpi)
}

/// winit treats an empty variable as unset; anything else replaces the dpi.
fn qualify_no_override(override_value: Option<OsString>) -> Result<(), String> {
    match override_value {
        Some(value) if !value.is_empty() => Err(format!(
            "{SCALE_OVERRIDE_VARIABLE}={value:?} would override the server's declared dpi"
        )),
        _ => Ok(()),
    }
}

/// winit reads `Xft/DPI` from the `_XSETTINGS_S<n>` owner before the
/// resource database; the lane runs no settings manager. Asked with
/// `only_if_exists`, so qualifying interns nothing on the server.
fn qualify_no_settings_manager(
    connection: &RustConnection,
    screen_index: usize,
) -> Result<(), X11QualificationDenial> {
    let selection = format!("_XSETTINGS_S{screen_index}");
    let atom = connection
        .intern_atom(true, selection.as_bytes())
        .map_err(dpi_denial)?
        .reply()
        .map_err(dpi_denial)?
        .atom;
    if atom == x11rb::NONE {
        return Ok(());
    }
    let owner = connection
        .get_selection_owner(atom)
        .map_err(dpi_denial)?
        .reply()
        .map_err(dpi_denial)?
        .owner;
    if owner == x11rb::NONE {
        Ok(())
    } else {
        Err(X11QualificationDenial::Dpi(format!(
            "a settings manager owns {selection} ({owner:#x}) and would supply Xft/DPI"
        )))
    }
}

/// The database exactly as winit builds it (`resource_manager::new_from_default`:
/// RESOURCE_MANAGER on the root, else the home files), parsed as winit parses it.
fn qualify_xft_dpi(database: &Database) -> Result<u32, String> {
    let Some(spelling) = database.get_string(XFT_DPI_RESOURCE, "") else {
        return Err(format!(
            "the resource database carries no {XFT_DPI_RESOURCE}; the lane loads \
             `{XFT_DPI_RESOURCE}: {QUALIFIED_DPI}` into RESOURCE_MANAGER on a -noreset server"
        ));
    };
    let dpi: f64 = spelling
        .parse()
        .map_err(|_| format!("{XFT_DPI_RESOURCE} {spelling:?} is not a number winit accepts"))?;
    if dpi == f64::from(QUALIFIED_DPI) {
        Ok(QUALIFIED_DPI)
    } else {
        Err(format!(
            "{XFT_DPI_RESOURCE} is {spelling}, not the qualified {QUALIFIED_DPI}"
        ))
    }
}

/// The screen's physical dpi as the server declares it: pixels over
/// millimetres. Rounded, because Xvfb's millimetre extent is itself derived
/// from `-dpi` and rounded.
fn screen_dpi(width_pixels: u16, width_millimeters: u16) -> Option<u32> {
    if width_millimeters == 0 {
        return None;
    }
    let dpi = f64::from(width_pixels) * 25.4 / f64::from(width_millimeters);
    Some(dpi.round() as u32)
}

fn qualify_screen_dpi(screen: &Screen) -> Result<u32, String> {
    match screen_dpi(screen.width_in_pixels, screen.width_in_millimeters) {
        Some(dpi) if dpi == QUALIFIED_DPI => Ok(dpi),
        Some(dpi) => Err(format!(
            "screen dpi {dpi} is not the qualified {QUALIFIED_DPI}"
        )),
        None => Err("screen declares zero millimetre width".to_owned()),
    }
}

fn dpi_denial(error: impl std::fmt::Display) -> X11QualificationDenial {
    X11QualificationDenial::Dpi(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use x11rb::resource_manager::Database;

    use super::{qualify_no_override, qualify_xft_dpi, screen_dpi, QUALIFIED_DPI};

    #[test]
    fn the_certified_scale_row_is_144_dpi() {
        assert_eq!(QUALIFIED_DPI, 144);
    }

    #[test]
    fn only_a_present_non_empty_override_is_a_denial() {
        assert!(qualify_no_override(None).is_ok());
        assert!(qualify_no_override(Some(OsString::new())).is_ok());
        assert!(qualify_no_override(Some(OsString::from("1.5"))).is_err());
        assert!(qualify_no_override(Some(OsString::from("randr"))).is_err());
    }

    #[test]
    fn xft_dpi_must_be_present_numeric_and_exactly_the_qualified_dpi() {
        let qualified = format!("Xft.dpi:\t{QUALIFIED_DPI}\n");
        assert_eq!(
            qualify_xft_dpi(&Database::new_from_data(qualified.as_bytes())),
            Ok(QUALIFIED_DPI)
        );
        assert!(qualify_xft_dpi(&Database::new_from_data(b"Xft.antialias:\t1\n")).is_err());
        assert!(qualify_xft_dpi(&Database::new_from_data(b"Xft.dpi:\t96\n")).is_err());
        assert!(qualify_xft_dpi(&Database::new_from_data(b"Xft.dpi:\t144.5\n")).is_err());
        assert!(qualify_xft_dpi(&Database::new_from_data(b"Xft.dpi:\thigh\n")).is_err());
    }

    #[test]
    fn xvfb_dpi_144_geometry_rounds_to_the_qualified_dpi_and_zero_width_is_rejected() {
        // Xvfb -dpi 144 declares 677 mm for the lane's 3840 px and 339 mm for
        // 1920 px (both measured with xdpyinfo).
        assert_eq!(screen_dpi(3840, 677), Some(QUALIFIED_DPI));
        assert_eq!(screen_dpi(1920, 339), Some(QUALIFIED_DPI));
        // -dpi 96 declares 508 mm: a 1.0 server is not the certified row.
        assert_ne!(screen_dpi(1920, 508), Some(QUALIFIED_DPI));
        assert_eq!(screen_dpi(1920, 0), None);
    }
}
