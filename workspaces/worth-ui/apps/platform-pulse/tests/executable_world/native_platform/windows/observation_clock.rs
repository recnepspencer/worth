//! Monotonic brackets establish input/observation order, not display latency.
use super::NativePlatformFailure;

pub(super) fn qpc_100ns() -> Result<i64, NativePlatformFailure> {
    let frequency = winsafe::QueryPerformanceFrequency()
        .map_err(|error| NativePlatformFailure::InputDelivery(error.to_string()))?;
    let counter = winsafe::QueryPerformanceCounter()
        .map_err(|error| NativePlatformFailure::InputDelivery(error.to_string()))?;
    if counter < 0 || frequency <= 0 {
        return Err(NativePlatformFailure::InputDelivery(
            "invalid monotonic input bracket".to_owned(),
        ));
    }
    i64::try_from(i128::from(counter) * 10_000_000 / i128::from(frequency)).map_err(|_| {
        NativePlatformFailure::InputDelivery("monotonic input bracket overflow".to_owned())
    })
}
