//! Bounded scalar control granules; allocation and seal always check immediately.
pub(in crate::authority::lease) struct WriteCadence {
    remaining: usize,
    interval: usize,
}
impl WriteCadence {
    pub(in crate::authority::lease) fn new<T>() -> Self {
        // At most 32 scalar writes or 64 KiB of inline moves. An individual
        // value/destructor remains the caller's indivisible operation.
        let interval = (64 * 1024 / std::mem::size_of::<T>().max(1)).clamp(1, 32);
        Self {
            remaining: 0,
            interval,
        }
    }
    pub(in crate::authority::lease) fn before(&self) -> bool {
        self.remaining == 0
    }
    pub(in crate::authority::lease) fn completed(&mut self) -> bool {
        if self.remaining == 0 {
            self.remaining = self.interval;
        }
        self.remaining -= 1;
        self.remaining == 0
    }
}
