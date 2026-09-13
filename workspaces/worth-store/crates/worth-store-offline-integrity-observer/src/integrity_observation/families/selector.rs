use super::super::{
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome, OfflinePhysicalDamageCause,
    OfflinePhysicalFormatField,
};
use super::durable_frame::{damaged_field, read_durable_frame, read_u64};
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityFormatDeclaration;

pub(crate) const ROOT_SELECTOR_BYTES: usize = 107;
const ROOT_SELECTOR_KIND: u8 = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorRole {
    Current = 1,
    Previous = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OfflineSelectorFacts {
    pub(crate) format: [u8; 10],
    pub(crate) store_identity: [u8; 16],
    pub(crate) selector_identity: u64,
    pub(crate) root_generation: u64,
    pub(crate) linked_selector_identity: Option<u64>,
    pub(crate) linked_root_generation: Option<u64>,
}

pub(crate) fn read_selector(
    bytes: &[u8],
    expected_role: SelectorRole,
    declaration: PhysicalIntegrityFormatDeclaration,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<OfflineSelectorFacts, OfflineIntegrityOutcome> {
    let frame = read_durable_frame(
        bytes,
        ROOT_SELECTOR_BYTES,
        ROOT_SELECTOR_KIND,
        declaration,
        counters,
    )?;
    counters.selector_payload_decoder_entries += 1;
    if frame.payload[51..].iter().any(|byte| *byte != 0) {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::MalformedPayload,
            99,
            8,
            OfflinePhysicalFormatField::Reserved,
        ));
    }
    let mut store_identity = [0; 16];
    store_identity.copy_from_slice(&frame.payload[..16]);
    if store_identity == [0; 16] {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::ScopeMismatch,
            48,
            16,
            OfflinePhysicalFormatField::StoreIdentity,
        ));
    }
    if frame.payload[16] != expected_role as u8 {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::ScopeMismatch,
            64,
            1,
            OfflinePhysicalFormatField::SelectorRole,
        ));
    }
    if frame.identity == 0 {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::ScopeMismatch,
            28,
            8,
            OfflinePhysicalFormatField::FrameIdentity,
        ));
    }
    let root_generation = read_u64(frame.payload, 17);
    if root_generation == 0 {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::Pointer,
            65,
            8,
            OfflinePhysicalFormatField::RootGeneration,
        ));
    }
    let linked_selector = read_u64(frame.payload, 25);
    let linked_generation = read_u64(frame.payload, 33);
    if (linked_selector == 0) != (linked_generation == 0) {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::Pointer,
            73,
            16,
            OfflinePhysicalFormatField::LinkedSelector,
        ));
    }
    if frame.payload[41..51] != frame.format {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::ScopeMismatch,
            89,
            10,
            OfflinePhysicalFormatField::EmbeddedFormat,
        ));
    }
    Ok(OfflineSelectorFacts {
        format: frame.format,
        store_identity,
        selector_identity: frame.identity,
        root_generation,
        linked_selector_identity: (linked_selector != 0).then_some(linked_selector),
        linked_root_generation: (linked_generation != 0).then_some(linked_generation),
    })
}
