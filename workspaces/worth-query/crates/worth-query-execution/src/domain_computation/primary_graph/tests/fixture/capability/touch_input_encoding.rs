//! Counts how many times a thread serializes a `CapabilityTouchInput`, so a
//! proof can show how many times one request encodes its input.

use serde::ser::SerializeStruct;

use super::CapabilityTouchInput;

thread_local! {
    static ENCODINGS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

impl CapabilityTouchInput {
    /// Starts counting this thread's touch input encodings.
    pub fn reset_encoding_count() {
        ENCODINGS.with(|count| count.set(0));
    }

    /// How many times this thread serialized a touch input since the reset.
    pub fn encoding_count() -> u32 {
        ENCODINGS.with(std::cell::Cell::get)
    }
}

/// Serializes exactly as a derived impl would, counting each call.
impl serde::Serialize for CapabilityTouchInput {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ENCODINGS.with(|count| count.set(count.get() + 1));
        let mut state = serializer.serialize_struct("CapabilityTouchInput", 9)?;
        state.serialize_field("account", &self.account)?;
        state.serialize_field("action", &self.action)?;
        state.serialize_field("purpose", &self.purpose)?;
        state.serialize_field("disclosure", &self.disclosure)?;
        state.serialize_field("related_account", &self.related_account)?;
        state.serialize_field("request_record", &self.request_record)?;
        state.serialize_field("prior_record", &self.prior_record)?;
        state.serialize_field("amount", &self.amount)?;
        state.serialize_field("caller_time", &self.caller_time)?;
        state.end()
    }
}
