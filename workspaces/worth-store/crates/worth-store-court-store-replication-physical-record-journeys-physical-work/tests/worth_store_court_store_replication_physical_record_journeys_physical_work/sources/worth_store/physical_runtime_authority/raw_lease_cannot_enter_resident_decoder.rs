use worth_store_buffer_pool::PhysicalFrameLease;

fn open_decoder(lease: &PhysicalFrameLease) {
    lease.with_owner_decoder(|bytes| bytes.len());
}

fn main() {}
