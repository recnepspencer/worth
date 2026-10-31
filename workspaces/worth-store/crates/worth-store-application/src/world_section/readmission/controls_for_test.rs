//! Borrow the same Home-owned physical control without extracting lifecycle.
impl<D, I, E, Ctx, T> super::WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn physical_serving_for_test(
        &self,
    ) -> &worth_store::physical_runtime::ServingPhysicalRuntime {
        self.home.physical_serving_for_test()
    }
}
