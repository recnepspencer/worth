//! Independent checkpoints derived from the Platform Pulse layout spec at the
//! 1536 x 1024 concept extent. These values deliberately do not import the
//! product's element geometry or theme descriptors.

pub(crate) const LOGICAL_EXTENT: [u32; 2] = [1536, 1024];
pub(crate) const SOURCE_SIGNAL_POINT: [u32; 2] = [30, 905];
pub(crate) const REVIEW_TARGET_POINT: [u32; 2] =
    [QUERY_POSTURE_REGION[0] + 6, QUERY_POSTURE_REGION[1] + 13];
pub(crate) const STABLE_BRAND_REGION: [u32; 4] = [26, 26, 204, 40];
/// The deployments panel keeps the 24-point gutter at the viewport's right
/// edge and starts below the masthead (57), gutter, greeting (66), card row
/// (98), traffic row (348) and three 20-point gaps. The Query status badge
/// keeps its concept insets inside that panel: 29 from its right edge and
/// 143 below its top.
const DEPLOYMENTS_RIGHT: u32 = LOGICAL_EXTENT[0] - 24;
const LOWER_ROW_TOP: u32 = 57 + 24 + 66 + 98 + 348 + 3 * 20;
pub(crate) const QUERY_POSTURE_REGION: [u32; 4] =
    [DEPLOYMENTS_RIGHT - 29 - 85, LOWER_ROW_TOP + 143, 85, 27];
pub(crate) const POSITIVE_RGB: [u8; 3] = [46, 157, 114];
pub(crate) const CAUTION_RGB: [u8; 3] = [201, 135, 45];
pub(crate) const REVIEW_TARGET_RGB: [u8; 3] = [237, 233, 255];
pub(crate) const CHANNEL_TOLERANCE: u8 = 12;
