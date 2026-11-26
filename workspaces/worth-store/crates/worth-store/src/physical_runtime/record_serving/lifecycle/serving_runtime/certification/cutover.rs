use super::ServingPhysicalRuntime;

impl ServingPhysicalRuntime {
    /// Derives the selected root's checkpoint manifest roster exactly as
    /// checkpoint cutover does (carrying the previous verified roster forward
    /// by its routing delta when one exists), then rebuilds it by an
    /// independent full scan of the same root. Returns both rosters in order
    /// plus the routes or routing blocks each inspected, as
    /// `(carried, production attempts, oracle attempts, inspected, oracle routes)`.
    #[allow(clippy::type_complexity)]
    pub fn certification_cutover_pin_audit(
        &self,
    ) -> Option<(bool, Vec<[u8; 16]>, Vec<[u8; 16]>, u64, u64)> {
        let audit = self
            .parts
            .publication
            .certification_cutover_pin_audit(4_096, 64 << 20)
            .ok()?;
        Some((
            audit.carried,
            audit.pins.iter().map(|pin| pin.attempt()).collect(),
            audit.oracle.iter().map(|pin| pin.attempt()).collect(),
            audit.inspected,
            audit.routes,
        ))
    }
}
