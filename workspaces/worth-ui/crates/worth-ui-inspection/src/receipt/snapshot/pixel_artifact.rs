#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiVisualPixelFormat {
    Rgba8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiVisualPixelColorSpace {
    Srgb,
    AdapterDeclared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiVisualPixelRetentionDisposition {
    Retained,
    Disposed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiVisualPixelCaptureSource {
    NativePresentation,
    RedactedNativePresentation,
    DerivedSnapshotCrop {
        parent_snapshot: u64,
        client_origin: [u32; 2],
    },
}

#[derive(Debug)]
pub struct UiVisualPixelArtifact {
    dimensions: [u32; 2],
    stride: u32,
    format: UiVisualPixelFormat,
    color_space: UiVisualPixelColorSpace,
    bytes: Box<[u8]>,
    source: UiVisualPixelCaptureSource,
    redaction: crate::UiVisualPixelRedaction,
    retention: UiVisualPixelRetentionDisposition,
    validity: UiVisualPixelArtifactValidity,
}

#[doc(hidden)]
pub struct UiVisualNativePixelArtifactInput {
    pub dimensions: [u32; 2],
    pub stride: u32,
    pub bytes: Box<[u8]>,
    pub color_space: UiVisualPixelColorSpace,
    pub redaction: crate::UiVisualPixelRedaction,
}

#[doc(hidden)]
pub struct UiVisualDerivedPixelArtifactInput {
    pub dimensions: [u32; 2],
    pub stride: u32,
    pub bytes: Box<[u8]>,
    pub color_space: UiVisualPixelColorSpace,
    pub redaction: crate::UiVisualPixelRedaction,
    pub parent_snapshot: u64,
    pub client_origin: [u32; 2],
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct UiVisualPixelArtifactValidity(Rc<Cell<bool>>);

impl UiVisualPixelArtifact {
    #[doc(hidden)]
    pub fn from_runtime_projection(mut input: UiVisualNativePixelArtifactInput) -> Self {
        let source = match input.redaction {
            crate::UiVisualPixelRedaction::UnredactedSyntheticContent => {
                UiVisualPixelCaptureSource::NativePresentation
            }
            crate::UiVisualPixelRedaction::OpaqueBlack => {
                for pixel in input.bytes.as_chunks_mut::<4>().0 {
                    pixel.copy_from_slice(&[0, 0, 0, u8::MAX]);
                }
                UiVisualPixelCaptureSource::RedactedNativePresentation
            }
        };
        Self {
            dimensions: input.dimensions,
            stride: input.stride,
            format: UiVisualPixelFormat::Rgba8,
            color_space: input.color_space,
            bytes: input.bytes,
            source,
            redaction: input.redaction,
            retention: UiVisualPixelRetentionDisposition::Retained,
            validity: UiVisualPixelArtifactValidity(Rc::new(Cell::new(true))),
        }
    }

    #[doc(hidden)]
    pub fn from_runtime_derived_crop(input: UiVisualDerivedPixelArtifactInput) -> Self {
        Self {
            dimensions: input.dimensions,
            stride: input.stride,
            format: UiVisualPixelFormat::Rgba8,
            color_space: input.color_space,
            bytes: input.bytes,
            source: UiVisualPixelCaptureSource::DerivedSnapshotCrop {
                parent_snapshot: input.parent_snapshot,
                client_origin: input.client_origin,
            },
            redaction: input.redaction,
            retention: UiVisualPixelRetentionDisposition::Retained,
            validity: UiVisualPixelArtifactValidity(Rc::new(Cell::new(true))),
        }
    }

    #[doc(hidden)]
    pub fn bind_runtime_validity(mut self, validity: UiVisualPixelArtifactValidity) -> Self {
        self.validity = validity;
        self
    }

    pub const fn dimensions(&self) -> [u32; 2] {
        self.dimensions
    }

    pub const fn stride(&self) -> u32 {
        self.stride
    }

    pub const fn format(&self) -> UiVisualPixelFormat {
        self.format
    }

    pub const fn color_space(&self) -> UiVisualPixelColorSpace {
        self.color_space
    }

    pub const fn capture_source(&self) -> UiVisualPixelCaptureSource {
        self.source
    }

    pub const fn redaction(&self) -> crate::UiVisualPixelRedaction {
        self.redaction
    }

    pub fn bytes(&self) -> &[u8] {
        if self.validity.0.get() {
            &self.bytes
        } else {
            &[]
        }
    }

    pub fn retention(&self) -> UiVisualPixelRetentionDisposition {
        if self.validity.0.get() {
            self.retention
        } else {
            UiVisualPixelRetentionDisposition::Disposed
        }
    }
}

impl UiVisualPixelArtifactValidity {
    #[doc(hidden)]
    pub fn issued_by_runtime(validity: Rc<Cell<bool>>) -> Self {
        Self(validity)
    }
}
use std::cell::Cell;
use std::rc::Rc;
