use worth_store_physical_format::{
    PageGenerationCell, PhysicalBinaryEncodingWitness, PhysicalHeaderAuthority, PhysicalPageKind,
    PHYSICAL_HEADER_LENGTH,
};

pub(crate) fn data_page_bytes(cell: PageGenerationCell, payload: &[u8]) -> Vec<u8> {
    let headers = header_authority();
    encoded_bytes(
        headers.encode_page_header(cell, PhysicalPageKind::DataPage, payload_length(payload)),
        payload,
    )
}

fn header_authority() -> PhysicalHeaderAuthority {
    PhysicalHeaderAuthority::for_canonical_physical_format(
        PhysicalBinaryEncodingWitness::physical_format_canonical()
            .expect("canonical certification physical format"),
    )
}

fn encoded_bytes(header: [u8; PHYSICAL_HEADER_LENGTH as usize], payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(header.len() + payload.len())
        .expect("bounded certification fixture allocation");
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(payload);
    bytes
}

fn payload_length(payload: &[u8]) -> u32 {
    u32::try_from(payload.len()).expect("certification fixture payload fits physical format")
}
