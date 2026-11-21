//! Native read failures survive ordinary handler error ownership.
use super::*;
impl<Stopped: std::fmt::Debug> std::fmt::Display
    for WorthQueryPartitionedComputationDenial<Stopped>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "partitioned computation stopped: {self:?}")
    }
}
impl<Stopped: std::fmt::Debug + 'static> std::error::Error
    for WorthQueryPartitionedComputationDenial<Stopped>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(WorthQueryComputationReadDenial::OutputAvailability(stop))
            | Self::Partition {
                cause:
                    WorthQueryComputationPartitionStop::Read(
                        WorthQueryComputationReadDenial::OutputAvailability(stop),
                    ),
                ..
            } => Some(stop.as_ref()),
            _ => None,
        }
    }
}
