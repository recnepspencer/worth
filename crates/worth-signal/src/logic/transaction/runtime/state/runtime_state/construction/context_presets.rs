use crate::data::graph::SignalGraph;

use crate::schema::data::SignalSchemaRegistry;

use super::super::super::builder::SignalRuntimeBuilder;

use super::super::SignalRuntime;

impl SignalRuntime<(), (), (), (), ()> {
    /// Create a runtime builder from a graph.
    ///
    /// Use this when you need abnormal setup, not for the normal path.
    pub fn builder(
        graph: SignalGraph,
    ) -> SignalRuntimeBuilder<
        super::super::super::builder::Missing,
        super::super::super::builder::Missing,
        (),
        (),
        (),
        (),
        (),
    > {
        SignalRuntimeBuilder::new(graph)
    }

    /// Create a runtime builder while preserving a heap-owned composition root.
    pub fn builder_from_boxed_graph(
        graph: Box<SignalGraph>,
    ) -> SignalRuntimeBuilder<
        super::super::super::builder::Missing,
        super::super::super::builder::Missing,
        (),
        (),
        (),
        (),
        (),
    > {
        SignalRuntimeBuilder::new_boxed(graph)
    }

    /// Build a runtime with the recommended default setup for a typed app context.
    pub fn build_for<Ctx>(graph: SignalGraph) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::build(graph)
    }

    /// Build a runtime with the recommended default setup and a first-class schema registry.
    pub fn build_for_with_schema<Ctx>(
        graph: SignalGraph,
        schema_registry: SignalSchemaRegistry,
    ) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::build_with_schema(graph, schema_registry)
    }

    /// Build a runtime with the richer development diagnostics preset for a typed app context.
    pub fn development_for<Ctx>(graph: SignalGraph) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::development(graph)
    }

    /// Build a runtime with the richer development diagnostics preset and a first-class schema registry.
    pub fn development_for_with_schema<Ctx>(
        graph: SignalGraph,
        schema_registry: SignalSchemaRegistry,
    ) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::development_with_schema(graph, schema_registry)
    }

    /// Build a runtime with the lean operational diagnostics preset for a typed app context.
    pub fn operational_for<Ctx>(graph: SignalGraph) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::operational(graph)
    }

    /// Build a runtime with the lean operational diagnostics preset and a first-class schema registry.
    pub fn operational_for_with_schema<Ctx>(
        graph: SignalGraph,
        schema_registry: SignalSchemaRegistry,
    ) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::operational_with_schema(graph, schema_registry)
    }

    /// Build a runtime with the web-development preset for a typed app context.
    pub fn web_development_for<Ctx>(graph: SignalGraph) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::web_development(graph)
    }

    /// Build a runtime with the web-development preset and a first-class schema registry.
    pub fn web_development_for_with_schema<Ctx>(
        graph: SignalGraph,
        schema_registry: SignalSchemaRegistry,
    ) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::web_development_with_schema(graph, schema_registry)
    }

    /// Build a runtime with the fintech preset for a typed app context.
    pub fn fintech_for<Ctx>(graph: SignalGraph) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::fintech(graph)
    }

    /// Build a runtime with the fintech preset and a first-class schema registry.
    pub fn fintech_for_with_schema<Ctx>(
        graph: SignalGraph,
        schema_registry: SignalSchemaRegistry,
    ) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::fintech_with_schema(graph, schema_registry)
    }

    /// Build a runtime with the heaviest forensic preset for a typed app context.
    pub fn forensic_for<Ctx>(graph: SignalGraph) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::forensic(graph)
    }

    /// Build a runtime with the heaviest forensic preset and a first-class schema registry.
    pub fn forensic_for_with_schema<Ctx>(
        graph: SignalGraph,
        schema_registry: SignalSchemaRegistry,
    ) -> SignalRuntime<(), (), (), Ctx, ()> {
        SignalRuntime::<(), (), (), Ctx, ()>::forensic_with_schema(graph, schema_registry)
    }
}
