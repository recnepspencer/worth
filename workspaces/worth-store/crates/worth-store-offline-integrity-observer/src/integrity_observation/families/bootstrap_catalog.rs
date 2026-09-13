use super::durable_frame::{read_durable_frame, read_u64};
use super::physical_fields::scope;
use crate::integrity_observation::{
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalFormatField as Field,
};
use worth_store_physical_format::integrity_declarations::families::root::BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION;

pub(crate) fn read_bootstrap_catalog(
    bytes: &[u8],
    store: [u8; 16],
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<(), OfflineIntegrityOutcome> {
    let frame = read_durable_frame(
        bytes,
        82,
        1,
        BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION,
        counters,
    )?;
    scope(frame.payload[..16] == store, 48, 16, Field::StoreIdentity)?;
    scope(
        frame.identity != 0 && read_u64(frame.payload, 16) == frame.identity,
        64,
        8,
        Field::ManifestGeneration,
    )?;
    scope(
        frame.payload[24..34] == frame.format,
        72,
        10,
        Field::EmbeddedFormat,
    )
}
