//! Checked scalar conversions shared by the graph preparation bounds.
use std::mem::size_of;

use crate::data::error::SignalError;

pub(super) fn bytes<T>(count: u64) -> Result<u64, SignalError> {
    count
        .checked_mul(size_of::<T>() as u64)
        .ok_or_else(overflow)
}

pub(super) fn overflow() -> SignalError {
    SignalError::invalid_input("graph epoch resource bound overflow")
}
