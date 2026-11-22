//! Lossless calendar-day values. Every `CanonicalDate` is canonical, so the
//! day count is the whole representation and its integer order is date order.
use super::{ExpressionValue, Repr};
use crate::values::CanonicalDate;

impl ExpressionValue {
    pub fn date(value: CanonicalDate) -> Self {
        Self(Repr::Date(value.days_from_unix_epoch))
    }

    pub fn as_date(&self) -> Option<CanonicalDate> {
        match self.0 {
            Repr::Date(days_from_unix_epoch) => Some(CanonicalDate {
                days_from_unix_epoch,
            }),
            _ => None,
        }
    }
}
