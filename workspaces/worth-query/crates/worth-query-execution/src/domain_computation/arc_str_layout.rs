//! Checked physical backing and initialized header for one owned `Arc<str>`.

pub(in crate::domain_computation) fn backing_bytes(payload_bytes: usize) -> Option<usize> {
    let header = std::alloc::Layout::array::<std::sync::atomic::AtomicUsize>(2).ok()?;
    let payload = std::alloc::Layout::array::<u8>(payload_bytes).ok()?;
    let (layout, _) = header.extend(payload).ok()?;
    Some(layout.pad_to_align().size())
}

pub(in crate::domain_computation) const fn initialized_header_work() -> usize {
    2 * std::mem::size_of::<std::sync::atomic::AtomicUsize>()
}
