use super::{
    seed::apply_initial_seed, WorthQueryInMemoryTestRuntimeBuilder, WorthQueryTestBackendError,
    WorthQueryTestBackendErrorKind, WorthQueryTestBackendSchema, WorthQueryTestSeedReceipt,
    WorthQueryTestSeedRow,
};

impl WorthQueryTestBackendSchema {
    /// Commits a fixture's initial rows into its real source owner before a
    /// public backend and Bridge are assembled from that same owner.
    pub fn seeded_relational_source(
        &self,
        identity_touch: crate::runtime::WorthQueryAspectTouch,
        rows: Vec<WorthQueryTestSeedRow>,
    ) -> Result<
        (
            worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
            WorthQueryTestSeedReceipt,
        ),
        WorthQueryTestBackendError,
    > {
        let count = rows.len();
        let specification = WorthQueryInMemoryTestRuntimeBuilder::default()
            .seed_collection_rows(identity_touch, rows)?
            .initial_seed;
        let mut memory = crate::memory_workspace::WorthQueryMemoryWorkspace::collection_with_native_contracts_for_initial_seed(
            self.collection(), self.memory_aspects()?, self.contracts().cloned(),
            worth_relational::facade::runtime::InvariantCatalog::default(), [], count,
        ).map_err(|error| WorthQueryTestBackendError::new(
            WorthQueryTestBackendErrorKind::WorkspaceBuildFailed, error.to_string(),
        ))?;
        let receipt = apply_initial_seed(&mut memory, self.collection(), specification)?;
        Ok((memory.relational_source_owner(), receipt))
    }
}
