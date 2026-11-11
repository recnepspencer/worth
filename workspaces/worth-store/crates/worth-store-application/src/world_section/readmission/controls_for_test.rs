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

    pub fn replace_relational_runtime_for_test(
        &mut self,
        runtime: worth_relational::facade::runtime::RelationalRuntime,
    ) {
        self.home.relational.replace_runtime_for_test(runtime);
    }

    pub fn publish_captured_relational_checkpoint_for_test(
        &self,
        checkpoint: Option<
            worth_relational::facade::durability::CompleteRelationalNativeCheckpoint,
        >,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observe_capture: Option<
            &dyn Fn(&worth_relational::facade::durability::RelationalNativeCheckpoint),
        >,
        before_native_acknowledgement: Option<&(dyn Fn() + Send + Sync)>,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        let world = self
            .world
            .checkpoint_port()
            .ok_or(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)?;
        crate::checkpoint::journey::checkpoint(
            &self.home.context.serving,
            &self.home.context.checkpoint,
            Some(&world),
            Some(&self.signal_checkpoint),
            execution,
            |home, through, digest| {
                self.home
                    .relational()
                    .prepare_captured_complete_checkpoint_for_test(
                        checkpoint,
                        home,
                        through,
                        digest,
                        execution,
                        observe_capture,
                    )
            },
            None,
            before_native_acknowledgement,
            None,
        )
    }
}
