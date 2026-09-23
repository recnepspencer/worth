//! What the Linux X11 certification profile requires of the X server before
//! any observation is trusted: XTEST for input synthesis and a 24-bit root
//! so `get_image` returns one BGRX pixel per 32 bits, and a root large enough
//! to hold the product's canonical window at the certified scale, because the
//! root is the one capture surface X offers and a client area past its edge
//! cannot be read back whole. The scale row the lane certifies at is
//! `scale.rs`.
use worth_ui_platform_pulse::visual_identity_pulse::{
    PLATFORM_PULSE_CANONICAL_LOGICAL_EXTENT, PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT,
};

use crate::native_platform::certified_physical_extent;

pub(super) const QUALIFIED_ROOT_DEPTH: u8 = 24;
pub(super) const REQUIRED_XTEST_VERSION: (u8, u16) = (2, 2);

pub(super) fn qualify_xtest_version(major: u8, minor: u16) -> Result<(), String> {
    if (major, minor) >= REQUIRED_XTEST_VERSION {
        Ok(())
    } else {
        Err(format!(
            "XTEST {major}.{minor} is below the required {}.{}",
            REQUIRED_XTEST_VERSION.0, REQUIRED_XTEST_VERSION.1
        ))
    }
}

pub(super) fn qualify_root_depth(depth: u8) -> Result<(), String> {
    if depth == QUALIFIED_ROOT_DEPTH {
        Ok(())
    } else {
        Err(format!(
            "root depth {depth} is not the qualified {QUALIFIED_ROOT_DEPTH}"
        ))
    }
}

/// The largest client area a courtroom world opens, in physical pixels at
/// the certified scale: the product's own window and the canonical world's.
/// Without a window manager the server maps it at the origin, so the screen
/// must be at least this large. The lane starts Xvfb at 3840x2160 (a 4K panel
/// at 150 %, the Windows record's class of host).
pub(super) fn required_screen_extent() -> [u32; 2] {
    let product = certified_physical_extent(PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT);
    let canonical = certified_physical_extent(PLATFORM_PULSE_CANONICAL_LOGICAL_EXTENT);
    [product[0].max(canonical[0]), product[1].max(canonical[1])]
}

pub(super) fn qualify_screen_extent(width: u16, height: u16) -> Result<(), String> {
    let required = required_screen_extent();
    if u32::from(width) >= required[0] && u32::from(height) >= required[1] {
        Ok(())
    } else {
        Err(format!(
            "screen {width}x{height} cannot hold the product's {}x{} client area at the \
             certified scale",
            required[0], required[1]
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        qualify_root_depth, qualify_screen_extent, qualify_xtest_version, required_screen_extent,
    };

    #[test]
    fn xtest_below_two_point_two_is_rejected_and_newer_minors_and_majors_pass() {
        assert!(qualify_xtest_version(2, 1).is_err());
        assert!(qualify_xtest_version(1, 9).is_err());
        assert!(qualify_xtest_version(2, 2).is_ok());
        assert!(qualify_xtest_version(3, 0).is_ok());
    }

    #[test]
    fn only_a_24_bit_root_is_qualified() {
        assert!(qualify_root_depth(24).is_ok());
        assert!(qualify_root_depth(32).is_err());
        assert!(qualify_root_depth(16).is_err());
    }

    #[test]
    fn the_screen_must_hold_the_product_window_at_the_certified_scale() {
        // 1536x1024 logical at 1.5: the first lane run at 1920x1080 mapped a
        // 2304x1536 client area and every capture-bound world failed late.
        assert_eq!(required_screen_extent(), [2304, 1536]);
        assert!(qualify_screen_extent(1920, 1080).is_err());
        assert!(qualify_screen_extent(2304, 1535).is_err());
        assert!(qualify_screen_extent(2303, 1536).is_err());
        assert!(qualify_screen_extent(2304, 1536).is_ok());
        assert!(qualify_screen_extent(3840, 2160).is_ok());
    }
}
