//! Finite installed capacity and the existing request-response temporal family.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationResourceLimits {
    pub maximum_live: u32,
    pub maximum_retained_results: u32,
    pub ingress_slots_per_request: u32,
    pub maximum_input_bytes: u64,
    pub maximum_frame_bytes: u64,
}

impl ApplicationResourceLimits {
    pub fn is_valid(self) -> bool {
        self.maximum_live > 0
            && self.maximum_retained_results > 0
            && self.ingress_slots_per_request > 0
            && self.maximum_input_bytes > 0
            && self.maximum_frame_bytes > 0
            && self
                .maximum_live
                .checked_add(self.maximum_retained_results)
                .is_some()
            && u64::from(self.maximum_live)
                .checked_mul(u64::from(self.ingress_slots_per_request))
                .and_then(|slots| slots.checked_mul(self.maximum_frame_bytes))
                .is_some()
    }
}

/// These durations lower to the existing native fixed-delay/per-attempt family.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationResourceRequestResponseProfile {
    pub retry_max_attempts: u32,
    pub retry_delay_milliseconds: u64,
    pub timeout_milliseconds: u64,
}

impl ApplicationResourceRequestResponseProfile {
    pub const fn is_valid(self) -> bool {
        self.retry_max_attempts > 0
            && self.retry_delay_milliseconds > 0
            && self.timeout_milliseconds > 0
    }
}
