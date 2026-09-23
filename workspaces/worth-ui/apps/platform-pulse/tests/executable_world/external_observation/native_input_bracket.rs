//! Monotonic brackets establish causal ordering of physical input delivery.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NativeInputDeliveryTiming {
    pub(crate) before_qpc_100ns: i64,
    pub(crate) after_qpc_100ns: i64,
}
