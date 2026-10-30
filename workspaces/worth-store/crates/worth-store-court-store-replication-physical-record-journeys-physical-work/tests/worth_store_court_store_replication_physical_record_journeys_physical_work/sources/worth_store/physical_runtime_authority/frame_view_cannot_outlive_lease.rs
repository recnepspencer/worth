use worth_store_buffer_pool::PhysicalFrameLease;

fn escape_frame_bytes(lease: PhysicalFrameLease) -> &'static [u8] {
    &lease
}

fn main() {}
