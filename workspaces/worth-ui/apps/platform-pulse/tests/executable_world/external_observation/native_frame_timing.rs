//! External compositor timestamps and the bounds around actual OS input delivery.
use super::NativeClientPixelCapture;

pub(crate) struct NativeTimedClientPixelCapture {
    pub(crate) captured_qpc_100ns: i64,
    pub(crate) acquired_qpc_100ns: i64,
    pub(crate) copied_qpc_100ns: i64,
    pub(crate) pixels: NativeClientPixelCapture,
}

/// SendInput does not expose its exact injection instant. These counter readings
/// bracket that call; consumers must retain the uncertainty instead of inventing
/// an exact input timestamp.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NativeInputDeliveryTiming {
    pub(crate) before_qpc_100ns: i64,
    pub(crate) after_qpc_100ns: i64,
}
