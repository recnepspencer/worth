use super::ServingPhysicalRuntime;

impl ServingPhysicalRuntime {
    /// Exercises the one-shot Store-owned epoch transaction and returns its
    /// arena frontier without exposing production caller tier authority.
    #[doc(hidden)]
    pub fn certification_activate_tier_epoch(
        &self,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
    ) -> Result<u64, String> {
        self.parts
            .publication
            .activate_tier_epoch(placement)
            .map_err(|denial| format!("{denial:?}"))
    }
}
