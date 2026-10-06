//! How large the swapchain is kept for a window extent.
//!
//! A DirectComposition swapchain presents at its own pixel size and the window
//! clips it, so the Windows profile, which presents through one, may keep the
//! swapchain larger than the window. A live resize then reallocates the
//! swapchain buffers, which drains the GPU and costs milliseconds each time,
//! only when the window outgrows them or shrinks well below them, instead of on
//! every frame of a drag. Frames draw only the window's extent; the rest of the
//! buffer is cleared and clipped.
//! Other profiles present a swapchain the window system sizes the window to, so
//! they keep it exactly at the window extent.

/// Pixels per axis the swapchain size is rounded up to.
#[cfg(target_os = "windows")]
const SWAPCHAIN_GRANULE: u32 = match super::QUALIFIED_DX12_PRESENTATION_SYSTEM {
    wgpu::Dx12SwapchainKind::DxgiFromVisual => 256,
    // An HWND swapchain stretches to the window, so it must match it exactly.
    wgpu::Dx12SwapchainKind::DxgiFromHwnd => 1,
};
#[cfg(not(target_os = "windows"))]
const SWAPCHAIN_GRANULE: u32 = 1;

/// The swapchain size to present `extent` from, keeping `configured` while it
/// still holds the extent with less than two granules of slack. `limit` is the
/// device's largest texture dimension, which rounding never exceeds.
pub(crate) fn swapchain_extent(extent: [u32; 2], configured: [u32; 2], limit: u32) -> [u32; 2] {
    swapchain_extent_in(extent, configured, limit, SWAPCHAIN_GRANULE)
}

/// The swapchain size to configure when the window extent becomes `extent`,
/// or `None` while the swapchain `configured` still serves it. A suspended
/// surface holds no configured swapchain, so it always configures one.
pub(crate) fn reconfiguration(
    extent: [u32; 2],
    configured: [u32; 2],
    limit: u32,
    suspended: bool,
) -> Option<[u32; 2]> {
    reconfiguration_in(extent, configured, limit, suspended, SWAPCHAIN_GRANULE)
}

fn reconfiguration_in(
    extent: [u32; 2],
    configured: [u32; 2],
    limit: u32,
    suspended: bool,
    granule: u32,
) -> Option<[u32; 2]> {
    let swapchain = swapchain_extent_in(extent, configured, limit, granule);
    (swapchain != configured || suspended).then_some(swapchain)
}

fn swapchain_extent_in(
    extent: [u32; 2],
    configured: [u32; 2],
    limit: u32,
    granule: u32,
) -> [u32; 2] {
    std::array::from_fn(|axis| {
        let (extent, configured) = (extent[axis], configured[axis]);
        if configured >= extent && configured - extent < 2 * granule - 1 {
            return configured;
        }
        extent
            .checked_next_multiple_of(granule)
            .filter(|rounded| *rounded <= limit)
            .unwrap_or(extent)
    })
}

#[cfg(test)]
mod tests {
    use super::{reconfiguration_in, swapchain_extent_in};

    const LIMIT: u32 = 16_384;

    #[test]
    fn an_exact_profile_follows_the_window_extent() {
        assert_eq!(
            swapchain_extent_in([801, 600], [800, 600], LIMIT, 1),
            [801, 600]
        );
        assert_eq!(
            swapchain_extent_in([799, 600], [800, 600], LIMIT, 1),
            [799, 600]
        );
    }

    #[test]
    fn a_drag_within_the_slack_keeps_the_swapchain() {
        let configured = swapchain_extent_in([1_000, 700], [0, 0], LIMIT, 256);
        assert_eq!(configured, [1_024, 768]);
        for width in [1_000, 1_024, 900, 800, 769] {
            assert_eq!(
                swapchain_extent_in([width, 700], configured, LIMIT, 256),
                configured
            );
        }
    }

    #[test]
    fn outgrowing_or_leaving_two_granules_of_slack_resizes() {
        assert_eq!(
            swapchain_extent_in([1_025, 700], [1_024, 768], LIMIT, 256),
            [1_280, 768]
        );
        assert_eq!(
            swapchain_extent_in([513, 700], [1_024, 768], LIMIT, 256),
            [768, 768]
        );
    }

    #[test]
    fn a_basis_the_swapchain_serves_leaves_it_configured() {
        assert_eq!(
            reconfiguration_in([900, 700], [1_024, 768], LIMIT, false, 256),
            None
        );
        assert_eq!(
            reconfiguration_in([1_100, 700], [1_024, 768], LIMIT, false, 256),
            Some([1_280, 768])
        );
    }

    #[test]
    fn a_resumed_surface_configures_even_at_its_old_size() {
        assert_eq!(
            reconfiguration_in([900, 700], [1_024, 768], LIMIT, true, 256),
            Some([1_024, 768])
        );
    }

    #[test]
    fn rounding_never_exceeds_the_device_limit() {
        assert_eq!(
            swapchain_extent_in([16_200, 700], [0, 0], 16_300, 256),
            [16_200, 768]
        );
    }
}
