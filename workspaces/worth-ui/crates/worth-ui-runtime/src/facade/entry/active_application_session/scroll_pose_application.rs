//! Evidence of accepted Scroll hit-index maintenance.
impl super::super::WorthUiActiveApplicationSession {
    /// Presented hit-index maintenance done for the last accepted scroll sample.
    pub fn last_scroll_hit_index_work(&self) -> crate::facade::mounted::UiHitTestSpatialWork {
        self.mounted.last_scroll_hit_index_work()
    }
}
