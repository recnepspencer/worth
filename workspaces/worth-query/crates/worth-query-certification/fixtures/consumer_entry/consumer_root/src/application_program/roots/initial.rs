use super::*;
use worth_query_topology_entry::PlanarInitialToOutputConnection;

pub struct InitialDiscoveredPlanarRoot;

impl ApplicationCompositionInstance for InitialDiscoveredPlanarRoot {
    const PATH: &'static str = "certification.initial-discovered-root";
}

type InitialConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    InitialDiscoveredPlanarRoot,
    PlanarSourceFeature,
    PlanarBodyOutput,
    InitialDiscoveredPlanarRoot,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarInitialToOutputConnection,
>;

type InitialDependentConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    InitialDiscoveredPlanarRoot,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    InitialDiscoveredPlanarRoot,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToLateFinalConnection,
>;

pub type ConsumerInitialDiscoveredProgramRoot = ApplicationDiscoveredOutputGraph<
    InitialConnection,
    ApplicationOutputEdge<InitialDependentConnection, ApplicationOutputLeaf>,
>;

// This fixed graph uses the same lawful no-source connection and real dependent delivery.
#[cfg(test)]
pub type ConsumerInitialFixedProgramRoot = ApplicationOutputGraph<
    InitialConnection,
    ApplicationOutputEdge<InitialDependentConnection, ApplicationOutputLeaf>,
>;
