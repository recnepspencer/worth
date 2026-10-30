pub(crate) use {
    crate::phase_eight_process::history::parent_oracle::canonical_membership_frame::crc32c,
    crate::phase_eight_process::history::parent_oracle::canonical_membership_frame::decode_frame,
    crate::phase_eight_process::history::parent_oracle::canonical_membership_frame::FRAME_BYTES,
};
pub(crate) use {
    crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::parse_placement,
    crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::Placement,
    crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::RecordIdentity,
};
pub(crate) use {
    worth_store_physical_format::CurrentPhysicalRecordPlacement,
    worth_store_physical_format::DurableExtentRecordPlacement,
    worth_store_physical_format::ExtentArenaId, worth_store_physical_format::ExtentArenaRange,
    worth_store_physical_format::PersistedRecordIdentity,
    worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
    worth_store_physical_format::PhysicalRootRoutingBlock,
    worth_store_physical_format::SelectedRecordContentClass,
    worth_store_physical_format::SelectedRecordRouteMetadata,
};

pub(crate) const ENTRY_OFFSET: usize = FRAME_BYTES + 40;

pub(crate) fn selected_extent_leaf() -> Vec<u8> {
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let record = worth_store_physical_format::PersistedRecordIdentity::new([9; 16], 5).unwrap();
    let extent =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(
                worth_store_physical_format::PhysicalExtentId::from_raw(11).unwrap(),
            )
            .with_extent_generation(
                worth_store_physical_format::PhysicalGeneration::from_raw(2).unwrap(),
            );
    let metadata = worth_store_physical_format::SelectedRecordRouteMetadata::primary(
        worth_store_physical_format::SelectedRecordContentClass::Opaque,
    )
    .unwrap();
    let placement = worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(
        worth_store_physical_format::DurableExtentRecordPlacement::new_selected(
            record,
            extent,
            23,
            worth_store_physical_format::ExtentArenaRange::new(
                worth_store_physical_format::ExtentArenaId::new(3).unwrap(),
                4096,
                8192,
            )
            .unwrap(),
            metadata,
        )
        .unwrap(),
    );
    worth_store_physical_format::PhysicalRootRoutingBlock::leaf(4, 2, 7, vec![placement], 1)
        .unwrap()
        .encode(format)
}

pub(crate) fn reseal(bytes: &mut [u8]) {
    let mut covered = Vec::with_capacity(bytes.len() - 4);
    covered.extend_from_slice(&bytes[..44]);
    covered.extend_from_slice(&bytes[FRAME_BYTES..]);
    bytes[44..FRAME_BYTES].copy_from_slice(&crc32c(&covered).to_le_bytes());
}
