//! Admission retains the execution owner's monotonic mechanism.
pub use worth_execution::{
    MonotonicClock as WorthQueryAdmissionClock,
    MonotonicTimeAdvanceDenial as WorthQueryAdmissionTimeAdvanceDenial,
    MonotonicTimeController as WorthQueryAdmissionTimeController,
    MonotonicTimeConstructionDenial as WorthQueryAdmissionTimeConstructionDenial,
};
