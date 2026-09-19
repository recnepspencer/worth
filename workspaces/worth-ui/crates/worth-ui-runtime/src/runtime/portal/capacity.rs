pub(super) const UI_PORTAL_LIVE_ROW_CAPACITY: usize = 1_024;

pub(super) const fn live_row_limit() -> u16 {
    1_024
}

pub(super) const fn admits_new_live_row(active_rows: usize) -> bool {
    active_rows < UI_PORTAL_LIVE_ROW_CAPACITY
}
