//! Relational owns a scan's coverage by deriving identities from its entries.
use super::{read_only_packets::PacketAdmissionDenial, PacketExecutionStop};
use worth_execution::{ChargedBytes, ExecutionScan, ScanDenial};
use worth_foundational::PartitionIdentity;

pub(crate) fn admit_ordered_scan<T: ChargedBytes>(
    entries: Vec<(PartitionIdentity, T)>,
) -> Result<ExecutionScan<T>, PacketExecutionStop> {
    let identities = entries.iter().map(|(identity, _)| *identity).collect();
    ExecutionScan::try_from_ordered(identities, entries).map_err(|denial| {
        PacketExecutionStop::Admission(match denial {
            ScanDenial::IdentitiesNotCanonical => {
                PacketAdmissionDenial::ExpectedIdentitiesNotCanonical
            }
            ScanDenial::MemoryOverflow => PacketAdmissionDenial::MemoryOverflow,
            // Only this owner supplies both sequences, from the same entries.
            ScanDenial::CoverageMismatch => {
                unreachable!("Relational derived matching scan coverage")
            }
        })
    })
}
