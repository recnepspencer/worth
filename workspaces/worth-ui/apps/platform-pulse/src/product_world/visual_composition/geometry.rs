use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentViewportAxisPlacement, ComponentViewportRegion,
};

/// A rectangle Pulse authors at a fixed position from the viewport origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseLogicalRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl PlatformPulseLogicalRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn origin(self) -> [u32; 2] {
        [self.x, self.y]
    }

    pub const fn extent(self) -> [u32; 2] {
        [self.width, self.height]
    }

    pub const fn right(self) -> u32 {
        self.x + self.width
    }

    pub const fn bottom(self) -> u32 {
        self.y + self.height
    }

    pub const fn contains(self, point: [u32; 2]) -> bool {
        point[0] >= self.x
            && point[0] < self.right()
            && point[1] >= self.y
            && point[1] < self.bottom()
    }

    pub fn allocation(self) -> ComponentAllocationMeasurementContract {
        let x = u16::try_from(self.x).expect("Pulse authored x fits viewport contract");
        let y = u16::try_from(self.y).expect("Pulse authored y fits viewport contract");
        let width = u16::try_from(self.width).expect("Pulse authored width fits viewport contract");
        let height =
            u16::try_from(self.height).expect("Pulse authored height fits viewport contract");
        viewport_region(
            ComponentViewportAxisPlacement::fixed_from_start(x, width)
                .expect("Pulse authored region width is nonzero"),
            ComponentViewportAxisPlacement::fixed_from_start(y, height)
                .expect("Pulse authored region height is nonzero"),
        )
    }
}

fn viewport_region(
    horizontal: ComponentViewportAxisPlacement,
    vertical: ComponentViewportAxisPlacement,
) -> ComponentAllocationMeasurementContract {
    ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
        horizontal, vertical,
    ))
}
