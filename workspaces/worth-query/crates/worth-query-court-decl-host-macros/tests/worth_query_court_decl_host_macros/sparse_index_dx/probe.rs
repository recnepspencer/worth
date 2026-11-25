//! The derived binary focus-window probe of the sparse interval index, in
//! Unix microseconds bounding `CanonicalTimestamp` fields.
use worth_query_decl::facade::{application_schema::CanonicalTimestamp, WorthBinaryValue};
use worth_query_host::facade::ordered_index::WorthQueryInterval1DProbe;

fn check_window(window: &FocusWindow) -> Result<(), &'static str> {
    (window.start_micros < window.end_micros)
        .then_some(())
        .ok_or("a focus window ends after it starts")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthBinaryValue)]
#[worth_binary(namespace = "sparse.dx.focus_window", version = 1, validate = check_window)]
pub struct FocusWindow {
    pub start_micros: i64,
    pub end_micros: i64,
}

impl WorthQueryInterval1DProbe<CanonicalTimestamp> for FocusWindow {
    fn interval_1d_bounds(&self) -> (CanonicalTimestamp, CanonicalTimestamp) {
        let at = |micros_since_unix_epoch| CanonicalTimestamp {
            micros_since_unix_epoch,
        };
        (at(self.start_micros), at(self.end_micros))
    }
}
