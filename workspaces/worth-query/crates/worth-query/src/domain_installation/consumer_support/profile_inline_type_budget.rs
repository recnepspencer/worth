//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryConsumerSupportPosture>("workspaces/worth-query/crates/worth-query/src/domain_installation/consumer_support/profile.rs", "WorthQueryConsumerSupportPosture", 128 * 1024);
    }
    {
        let _owner = census.owner::<WorthQueryConsumerSupportDimension>("workspaces/worth-query/crates/worth-query/src/domain_installation/consumer_support/profile.rs", "WorthQueryConsumerSupportDimension", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryConsumerSupportProfile>("workspaces/worth-query/crates/worth-query/src/domain_installation/consumer_support/profile.rs", "WorthQueryConsumerSupportProfile", 128 * 1024);
        owner
            .field::<[WorthQueryConsumerSupportPosture; WorthQueryConsumerSupportDimension::COUNT]>(
                "postures",
            );
    }
}
