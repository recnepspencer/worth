//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQuerySharedConsumerDeliveryPolicy>(
            "workspaces/worth-query/crates/worth-query/src/live/maintenance/consumer_policy.rs",
            "WorthQuerySharedConsumerDeliveryPolicy",
            128 * 1024,
        );
        owner.field::<String>("purpose");
        owner.field::<String>("disclosure");
        owner.field::<String>("continuation");
        owner.field::<crate::subscription::DeliveryBackpressurePolicy>("backpressure");
    }
}
