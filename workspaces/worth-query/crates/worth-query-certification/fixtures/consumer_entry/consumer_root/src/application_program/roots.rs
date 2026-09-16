use worth_query_decl::facade::application_program::{
    ApplicationCompositionInstance, ApplicationConnectionInstanceRef, ApplicationOutputEdge,
    ApplicationOutputGraph, ApplicationOutputLeaf,
};
use worth_query_topology_entry::{
    PlanarBodyInput, PlanarBodyOutput, PlanarOutputFeature, PlanarSourceFeature,
    PlanarSourceToOutputConnection,
};

use super::{
    ConsumerSchema, PlanarAlternateDependentConnection, PlanarAlternateSummaryConnection,
    PlanarConnection, PlanarDependentConnection, PlanarSummaryConnection,
};

pub struct SecondaryPlanarRoot;
pub struct UndeclaredPlanarRoot;

impl ApplicationCompositionInstance for SecondaryPlanarRoot {
    const PATH: &'static str = "certification.secondary-root";
}

impl ApplicationCompositionInstance for UndeclaredPlanarRoot {
    const PATH: &'static str = "certification.undeclared-root";
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
