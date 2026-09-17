use std::sync::Arc;

use super::super::{UiNativeTextValue, UiPresentProjection, UiProjectionAvailability};
use super::UiProjectionInputPosture;

pub(super) fn scalar_input(
    availability: &UiProjectionAvailability<UiNativeTextValue>,
) -> (UiProjectionInputPosture, Option<Arc<str>>) {
    match availability {
        UiProjectionAvailability::Present(UiPresentProjection::Current(value)) => (
            UiProjectionInputPosture::Current,
            Some(Arc::from(value.as_str())),
        ),
        UiProjectionAvailability::Present(UiPresentProjection::RetainedStale {
            value,
            activity,
        }) => (
            UiProjectionInputPosture::RetainedStale(activity.kind()),
            Some(Arc::from(value.as_str())),
        ),
        UiProjectionAvailability::Unavailable(receipt) => {
            (UiProjectionInputPosture::Unavailable(receipt.kind()), None)
        }
        UiProjectionAvailability::Stopped(receipt) => {
            (UiProjectionInputPosture::Stopped(receipt.kind()), None)
        }
    }
}
