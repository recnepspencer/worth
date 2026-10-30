//! Optional construction-input observations for the test-cost census.
//!
//! A fingerprint describes a named builder's explicit input recipe in this
//! binary. It grants no permission to share a live fixture or its generated
//! filesystem identity. Callers include all variable construction inputs;
//! opaque runtime state is not replaceable with Debug text or a pointer.

mod construction_inputs;
mod recording;

pub use construction_inputs::SetupConstructionInputs;
pub use recording::{record, SetupObservation, SetupObservationDisposition};
