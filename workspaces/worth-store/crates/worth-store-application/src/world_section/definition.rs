//! Authored owner configuration before Home binds its actual Native source.
use worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder;
use worth_runtime_world::facade::RuntimeWorldHomeBuilder;

pub struct ApplicationWorldDefinition<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) world: RuntimeWorldHomeBuilder<D, I, E, Ctx, T>,
    pub(super) bridge: UnattachedRelationalBridgeBuilder,
    pub(super) graph_role: String,
}
impl<D, I, E, Ctx, T> ApplicationWorldDefinition<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn new(
        world: RuntimeWorldHomeBuilder<D, I, E, Ctx, T>,
        bridge: UnattachedRelationalBridgeBuilder,
        graph_role: impl Into<String>,
    ) -> Self {
        Self {
            world,
            bridge,
            graph_role: graph_role.into(),
        }
    }
}
