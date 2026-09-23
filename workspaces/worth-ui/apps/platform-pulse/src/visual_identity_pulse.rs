pub const PLATFORM_PULSE_IDENTITY_TARGET_AUTHORED_NAME: &str =
    "component:platform.pulse.component.review_target";
pub const PLATFORM_PULSE_CANONICAL_LOGICAL_EXTENT: [u32; 2] = [1536, 1024];
pub const PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT: [u32; 2] = [1536, 1024];
pub const PLATFORM_PULSE_BACKGROUND_LOGICAL_POINT: [u32; 2] = [250, 80];
pub const PLATFORM_PULSE_TARGET_LOGICAL_POINT: [u32; 2] = [1398, 792];
/// The logical client extent every native courtroom world (and the product
/// itself) opens at; the certified lane's physical expectation is this
/// extent at the certified scale.
pub const PLATFORM_PULSE_NATIVE_WINDOW_LOGICAL_EXTENT: [u32; 2] = [160, 96];
pub const PLATFORM_PULSE_MAXIMUM_CAPTURE_SCALE: u32 = 4;
pub const PLATFORM_PULSE_MAXIMUM_PIXEL_BYTES: u64 = 100_663_296;
pub const PLATFORM_PULSE_TARGET_RGB: [u8; 3] = [0xac, 0x67, 0xf2];
pub const PLATFORM_PULSE_CONFIRMATION_RGB: [u8; 3] = [0x17, 0x1d, 0x25];
pub const PLATFORM_PULSE_VISIBLE_REGION_COUNT: u64 = 15;
pub const PLATFORM_PULSE_HIT_TEST_REGION_COUNT: u64 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseVisualIdentityScenario {
    target_authored_name: &'static str,
    logical_extent: [u32; 2],
    background_logical_point: [u32; 2],
    target_logical_point: [u32; 2],
}

impl PlatformPulseVisualIdentityScenario {
    pub const fn canonical() -> Self {
        Self {
            target_authored_name: PLATFORM_PULSE_IDENTITY_TARGET_AUTHORED_NAME,
            logical_extent: PLATFORM_PULSE_CANONICAL_LOGICAL_EXTENT,
            background_logical_point: PLATFORM_PULSE_BACKGROUND_LOGICAL_POINT,
            target_logical_point: PLATFORM_PULSE_TARGET_LOGICAL_POINT,
        }
    }

    pub const fn target_authored_name(self) -> &'static str {
        self.target_authored_name
    }

    pub const fn logical_extent(self) -> [u32; 2] {
        self.logical_extent
    }

    pub const fn background_logical_point(self) -> [u32; 2] {
        self.background_logical_point
    }

    pub const fn target_logical_point(self) -> [u32; 2] {
        self.target_logical_point
    }
}
