//! The installed product root's committed-publication pulse, for cheap
//! "anything committed?" checks and commit wake-ups by hosts.
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::execution_runtime::product_world::WorthQueryCommitPulse;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The pulse shared by every clone of this installed runtime: its
    /// version advances after each advancement that published to World on
    /// any of its branches. It grants no read and records no commit.
    pub fn commit_pulse(&self) -> WorthQueryCommitPulse {
        self.product_runtime().commit_pulse()
    }
}
