mod resident_admission;
mod root_protocol_routes;

pub(in crate::physical_runtime) use resident_admission::ResidentAdmissionCounterCells;
pub use resident_admission::ResidentAdmissionCounters;
pub(in crate::physical_runtime) use root_protocol_routes::RootProtocolRouteCounterCells;
pub use root_protocol_routes::{PhysicalRootProtocolRoute, RootProtocolRouteCounters};
