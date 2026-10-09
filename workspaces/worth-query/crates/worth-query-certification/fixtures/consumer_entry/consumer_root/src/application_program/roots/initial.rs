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
