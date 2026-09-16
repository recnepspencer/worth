use worth_query_decl::facade::application_program::{
    ApplicationCompositionInstance, ApplicationConnectionInstanceRef,
    ApplicationDiscoveredOutputGraph, ApplicationOutputEdge, ApplicationOutputGraph,
    ApplicationOutputLeaf,
};
use worth_query_topology_entry::{
    PlanarBodyInput, PlanarBodyOutput, PlanarDerivedBodyInput, PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature, PlanarOutputFeature, PlanarOutputToLateFinalConnection,
    PlanarSourceFeature, PlanarSourceToOutputConnection, PlanarSourceToRemoteOutputConnection,
};

use super::{
    ConsumerSchema, PlanarAlternateDependentConnection, PlanarAlternateSummaryConnection,
    PlanarConnection, PlanarDependentConnection, PlanarSummaryConnection,
};

pub struct SecondaryPlanarRoot;
pub struct DiscoveredPlanarRoot;
pub struct UndeclaredPlanarRoot;
pub struct RequiredSharedPlanarRoot;

impl ApplicationCompositionInstance for SecondaryPlanarRoot {
    const PATH: &'static str = "certification.secondary-root";
}

impl ApplicationCompositionInstance for DiscoveredPlanarRoot {
    const PATH: &'static str = "certification.discovered-root";
}

impl ApplicationCompositionInstance for UndeclaredPlanarRoot {
    const PATH: &'static str = "certification.undeclared-root";
}

impl ApplicationCompositionInstance for RequiredSharedPlanarRoot {
    const PATH: &'static str = "certification.required-shared-root";
}

pub type ConsumerProgramRoot = ApplicationOutputGraph<
    PlanarConnection,
    (
        ApplicationOutputEdge<
            PlanarDependentConnection,
            ApplicationOutputEdge<PlanarSummaryConnection, ApplicationOutputLeaf>,
        >,
        ApplicationOutputEdge<
            PlanarAlternateDependentConnection,
            ApplicationOutputEdge<PlanarAlternateSummaryConnection, ApplicationOutputLeaf>,
        >,
    ),
>;

pub type ConsumerTruncatedProgramRoot =
    ApplicationOutputGraph<PlanarConnection, ApplicationOutputLeaf>;

type SecondaryPlanarConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    SecondaryPlanarRoot,
    PlanarSourceFeature,
    PlanarBodyOutput,
    SecondaryPlanarRoot,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;

pub type ConsumerSecondaryProgramRoot =
    ApplicationOutputGraph<SecondaryPlanarConnection, ApplicationOutputLeaf>;

type DiscoveredPlanarConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    DiscoveredPlanarRoot,
    PlanarSourceFeature,
    PlanarBodyOutput,
    DiscoveredPlanarRoot,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;

type DiscoveredPlanarDependentConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    DiscoveredPlanarRoot,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    DiscoveredPlanarRoot,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToLateFinalConnection,
>;

pub type ConsumerDiscoveredProgramRoot = ApplicationDiscoveredOutputGraph<
    DiscoveredPlanarConnection,
    ApplicationOutputEdge<DiscoveredPlanarDependentConnection, ApplicationOutputLeaf>,
>;

type RequiredRemoteConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    RequiredSharedPlanarRoot,
    PlanarSourceFeature,
    PlanarBodyOutput,
    RequiredSharedPlanarRoot,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToRemoteOutputConnection,
>;

type RequiredLateDependentConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    RequiredSharedPlanarRoot,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    RequiredSharedPlanarRoot,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToLateFinalConnection,
>;

pub type ConsumerRequiredSharedRoot = ApplicationOutputGraph<
    RequiredRemoteConnection,
    ApplicationOutputEdge<RequiredLateDependentConnection, ApplicationOutputLeaf>,
>;

type UndeclaredPlanarConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    UndeclaredPlanarRoot,
    PlanarSourceFeature,
    PlanarBodyOutput,
    UndeclaredPlanarRoot,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;

pub type ConsumerUndeclaredProgramRoot =
    ApplicationOutputGraph<UndeclaredPlanarConnection, ApplicationOutputLeaf>;
