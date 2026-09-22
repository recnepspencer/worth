#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeClientAreaBounds {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeWindowIdentity(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProcessBoundNativeClientAreaObservation {
    process_id: u32,
    window: NativeWindowIdentity,
    bounds: NativeClientAreaBounds,
    dpi: u32,
    window_lookup_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeClientPixelCapture {
    process_id: u32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    capture_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeClientPixelPoint {
    x: u32,
    y: u32,
    capture_width: u32,
    capture_height: u32,
    landing_tolerance: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NormalNativeCloseRequestObservation {
    process_id: u32,
    request_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeWindowVisibilityTransitionObservation {
    minimized_observations: u32,
    restored_observations: u32,
    restored_client: ProcessBoundNativeClientAreaObservation,
}

impl NativeClientAreaBounds {
    pub(crate) fn new(left: i32, top: i32, right: i32, bottom: i32) -> Option<Self> {
        (right > left && bottom > top).then_some(Self {
            left,
            top,
            right,
            bottom,
        })
    }

    pub(crate) fn left(self) -> i32 {
        self.left
    }

    pub(crate) fn top(self) -> i32 {
        self.top
    }

    pub(crate) fn right(self) -> i32 {
        self.right
    }

    pub(crate) fn bottom(self) -> i32 {
        self.bottom
    }

    pub(crate) fn width(self) -> u32 {
        (self.right - self.left) as u32
    }

    pub(crate) fn height(self) -> u32 {
        (self.bottom - self.top) as u32
    }
}

impl ProcessBoundNativeClientAreaObservation {
    pub(crate) fn new(
        process_id: u32,
        window: NativeWindowIdentity,
        bounds: NativeClientAreaBounds,
        dpi: u32,
        window_lookup_count: u32,
    ) -> Self {
        Self {
            process_id,
            window,
            bounds,
            dpi,
            window_lookup_count,
        }
    }

    pub(crate) fn process_id(self) -> u32 {
        self.process_id
    }

    pub(crate) fn bounds(self) -> NativeClientAreaBounds {
        self.bounds
    }

    pub(crate) fn dpi(self) -> u32 {
        self.dpi
    }

    pub(crate) fn window(self) -> NativeWindowIdentity {
        self.window
    }

    pub(crate) fn window_lookup_count(self) -> u32 {
        self.window_lookup_count
    }
}

impl NativeWindowIdentity {
    pub(crate) fn from_native_value(value: usize) -> Option<Self> {
        (value != 0).then_some(Self(value))
    }
}

impl NativeClientPixelCapture {
    pub(crate) fn new(process_id: u32, width: u32, height: u32, rgba: Vec<u8>) -> Option<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;
        (rgba.len() == expected).then_some(Self {
            process_id,
            width,
            height,
            rgba,
            capture_count: 1,
        })
    }

    pub(crate) fn process_id(&self) -> u32 {
        self.process_id
    }

    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub(crate) fn cropped(&self, [x, y, width, height]: [u32; 4]) -> Option<Self> {
        let right = x.checked_add(width)?;
        let bottom = y.checked_add(height)?;
        if width == 0 || height == 0 || right > self.width || bottom > self.height {
            return None;
        }
        let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
        for row in y..bottom {
            let start = (row as usize * self.width as usize + x as usize) * 4;
            rgba.extend_from_slice(&self.rgba[start..start + width as usize * 4]);
        }
        Self::new(self.process_id, width, height, rgba)
            .map(|capture| capture.with_capture_count(self.capture_count))
    }

    pub(crate) fn capture_count(&self) -> u32 {
        self.capture_count
    }

    pub(crate) fn with_capture_count(mut self, capture_count: u32) -> Self {
        self.capture_count = capture_count;
        self
    }
}

impl NativeClientPixelPoint {
    pub(crate) fn interior(
        capture: &NativeClientPixelCapture,
        x: u32,
        y: u32,
        landing_tolerance: u32,
    ) -> Option<Self> {
        let right = x.checked_add(landing_tolerance)?;
        let bottom = y.checked_add(landing_tolerance)?;
        (x >= landing_tolerance
            && y >= landing_tolerance
            && right < capture.width()
            && bottom < capture.height())
        .then_some(Self {
            x,
            y,
            capture_width: capture.width(),
            capture_height: capture.height(),
            landing_tolerance,
        })
    }

    pub(crate) fn coordinates(self) -> (u32, u32) {
        (self.x, self.y)
    }

    pub(crate) fn capture_extent(self) -> (u32, u32) {
        (self.capture_width, self.capture_height)
    }

    pub(crate) fn landing_tolerance(self) -> u32 {
        self.landing_tolerance
    }
}

#[test]
fn cropped_pixels_preserve_selected_rows_and_capture_provenance() {
    let rgba: Vec<u8> = (0_u8..12)
        .flat_map(|value| [value, value, value, 255])
        .collect();
    let capture = NativeClientPixelCapture::new(7, 4, 3, rgba)
        .unwrap()
        .with_capture_count(3);
    let cropped = capture.cropped([1, 1, 2, 2]).unwrap();
    assert_eq!(
        (
            cropped.process_id(),
            cropped.width(),
            cropped.height(),
            cropped.capture_count()
        ),
        (7, 2, 2, 3)
    );
    let selected: Vec<_> = cropped
        .rgba()
        .chunks_exact(4)
        .map(|pixel| pixel[0])
        .collect();
    assert_eq!(selected, [5, 6, 9, 10]);
    for region in [
        [0, 0, 0, 1],
        [3, 0, 2, 1],
        [0, 2, 1, 2],
        [u32::MAX, 0, 2, 1],
    ] {
        assert!(capture.cropped(region).is_none());
    }
}

impl NormalNativeCloseRequestObservation {
    pub(crate) fn one(process_id: u32) -> Self {
        Self {
            process_id,
            request_count: 1,
        }
    }

    pub(crate) fn process_id(self) -> u32 {
        self.process_id
    }

    pub(crate) fn request_count(self) -> u32 {
        self.request_count
    }
}

impl NativeWindowVisibilityTransitionObservation {
    pub(crate) fn observed(restored_client: ProcessBoundNativeClientAreaObservation) -> Self {
        Self {
            minimized_observations: 1,
            restored_observations: 1,
            restored_client,
        }
    }

    pub(crate) const fn minimized_observations(self) -> u32 {
        self.minimized_observations
    }

    pub(crate) const fn restored_observations(self) -> u32 {
        self.restored_observations
    }

    pub(crate) const fn restored_client(self) -> ProcessBoundNativeClientAreaObservation {
        self.restored_client
    }
}
