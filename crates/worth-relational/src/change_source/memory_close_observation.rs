//! A completed, waiting or refused step of the original memory retirement.
use crate::runtime::close_denial::RelationalRuntimeCloseDenial;

#[derive(Debug)]
pub enum RelationalMemoryCloseObservation {
    Advanced,
    Busy,
    Stopped(worth_execution::WorkCeilingDenial),
    Refused(RelationalRuntimeCloseDenial),
    Complete,
}
