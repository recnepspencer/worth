mod harness_fixture;
pub use harness_fixture::BridgeHarnessFixture;

mod recording_signal_sink;
mod recording_writeback_authority;
pub use recording_signal_sink::RecordingSignalBridgeSink;
pub use recording_writeback_authority::RecordingTruthWritebackAuthority;

pub use crate::relational_source::bridge_source_tests::support::{NamedTruth, RelationalFixture};

pub use crate::relational_source::bridge_source_tests::struct_field_patch::ProfileTruth;

mod scenario_relational_truth;
pub use scenario_relational_truth::ScenarioRelationalTruth;
