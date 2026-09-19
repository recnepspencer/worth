const PE_SIGNATURE_BYTES: usize = 4;
const COFF_HEADER_BYTES: usize = 20;
const OPTIONAL_HEADER_STACK_RESERVE_OFFSET: usize = 72;
const PE32_PLUS_MAGIC: u16 = 0x20b;

#[test]
fn product_binary_carries_the_bounded_platform_pulse_stack_reserve() {
    let binary = std::fs::read(env!("CARGO_BIN_EXE_worth-ui-platform-pulse"))
        .expect("Cargo-built Platform Pulse executable should be readable");
    let pe_offset = read_u32(&binary, 0x3c) as usize;
    assert_eq!(
        binary.get(pe_offset..pe_offset + PE_SIGNATURE_BYTES),
        Some(b"PE\0\0".as_slice()),
        "Platform Pulse should be a PE executable"
    );
    let optional_header = pe_offset + PE_SIGNATURE_BYTES + COFF_HEADER_BYTES;
    assert_eq!(
        read_u16(&binary, optional_header),
        PE32_PLUS_MAGIC,
        "the x86_64 Platform Pulse executable should use PE32+"
    );
    let reserve = read_u64(
        &binary,
        optional_header + OPTIONAL_HEADER_STACK_RESERVE_OFFSET,
    );
    let expected = env!("WORTH_UI_PLATFORM_PULSE_STACK_RESERVE_BYTES")
        .parse::<u64>()
        .expect("build script stack reserve should be an integer");
    assert_eq!(
        reserve, expected,
        "the product binary must retain its measured Windows main-stack budget"
    );
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("PE u16 field should be in bounds"),
    )
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("PE u32 field should be in bounds"),
    )
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("PE u64 field should be in bounds"),
    )
}
