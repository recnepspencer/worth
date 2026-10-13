//! An unleased immutable-read kernel can refuse only for its caller's request.
use super::{ReadOnlyPacket, RelationalExecutionDenialCause};
use worth_foundational::PartitionIdentity;

#[derive(Debug)]
pub(crate) struct UnleasedReadStop {
    pub identity: PartitionIdentity,
    pub cause: RelationalExecutionDenialCause,
}

/// Descriptor order is admitted before dispatch. A stopped batch discards its
/// complete prefix; no result escapes before every request-backed read settles.
pub(crate) fn execute_unleased_read_only_packets<P, T>(
    packets: &[ReadOnlyPacket<P>],
    kernel: impl Fn(&P) -> Result<T, RelationalExecutionDenialCause>,
) -> Result<Vec<T>, UnleasedReadStop> {
    packets
        .iter()
        .map(|packet| {
            kernel(&packet.value).map_err(|cause| UnleasedReadStop {
                identity: packet.identity,
                cause,
            })
        })
        .collect()
}
