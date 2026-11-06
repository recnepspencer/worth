//! The final output's two roles have one public program/install declaration.
use super::*;
use crate::PlanarFinalOutputFeature;
use worth_query_host::facade::application_contribution::{
    WorthQueryFeatureRoleRuntime, WorthQueryFeatureRuntime, WorthQueryRequiredFeaturePair,
};

/// Both producer/readiness slots are mandatory in the typed declaration.
/// ```compile_fail,E0107
/// use worth_query_host::facade::application_contribution::WorthQueryRequiredFeaturePair;
/// use worth_query_topology_entry::*;
/// type MissingPreserve<S> = WorthQueryRequiredFeaturePair<S, PlanarFinalOutputFeature,
///     PlanarFinalOutputProducer<S>, PlanarFinalOutputReadiness<S>>;
/// ```
/// The Initial readiness must target the Initial producer's actual operation.
/// ```compile_fail,E0271
/// use worth_query_host::facade::application_contribution::*;
/// use worth_query_topology_entry::*;
/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// pub struct Schema;
/// impl worth_query_decl::facade::application_schema::fragment::Fragment for Schema {
///  const NAME: &'static str = "Schema";
///  type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
///  type Members = (TopologyContribution,);
/// }
/// impl worth_query_decl::facade::application_schema::ApplicationSchema for Schema {
///  const OWNER: &'static str = "wrong-readiness"; const NAME: &'static str = "Schema";
///  const MAJOR: u32 = 1; const MINOR: u32 = 0;
///  fn declaration() -> Result<worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>, worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial> { worth_query_decl::facade::application_schema::fragment::declare(Self) }
///  fn visit_correction<V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>>(source: std::any::TypeId, visitor: V) -> V::Output { worth_query_decl::facade::application_schema::fragment::visit_correction(Self, source, visitor) }
/// }
/// impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition for Schema { type Contributions = (TopologyContribution,); }
///
/// impl TopologySchemaBinding for Schema { type PrincipalFragment = worth_query_topology_entry::PrincipalRecords; }
/// type Wrong = WorthQueryRequiredFeaturePair<Schema, PlanarFinalOutputFeature,
///     PlanarFinalOutputProducer<Schema>, PlanarFinalPreserveReadiness<Schema>,
///     PlanarFinalPreserveProducer<Schema>, PlanarFinalPreserveReadiness<Schema>>;
/// fn require<C: WorthQueryRequiredFeatureContract<Schema>>() {}
/// require::<Wrong>();
/// ```
// compiled-example: begin
pub type PlanarFinalFeatureRoles<Schema, Feature = PlanarFinalOutputFeature> =
    WorthQueryRequiredFeaturePair<
        Schema,
        Feature,
        PlanarFinalOutputProducer<Schema>,
        PlanarFinalOutputReadiness<Schema>,
        PlanarFinalPreserveProducer<Schema>,
        PlanarFinalPreserveReadiness<Schema>,
    >;
// compiled-example: end
/// A runtime literal cannot silently omit its Preserve slot.
/// ```compile_fail,E0063
/// use worth_query_host::facade::application_contribution::WorthQueryFeatureRuntime;
/// use worth_query_topology_entry::{PlanarFinalFeatureRuntime, planar_final_feature_runtime};
/// let complete = planar_final_feature_runtime();
/// let missing: PlanarFinalFeatureRuntime = WorthQueryFeatureRuntime { initial: complete.initial };
/// ```
pub type PlanarFinalFeatureRuntime = WorthQueryFeatureRuntime<
    WorthQueryFeatureRoleRuntime<FinalPlanarMutationHandler, PlanarFinalOutputProvider, ()>,
    WorthQueryFeatureRoleRuntime<FinalPlanarPreserveHandler, PlanarFinalOutputProvider, ()>,
>;

pub fn planar_final_feature_runtime() -> PlanarFinalFeatureRuntime {
    WorthQueryFeatureRuntime::new(
        WorthQueryFeatureRoleRuntime::new(
            FinalPlanarMutationHandler,
            PlanarFinalOutputProvider::default(),
            (),
        ),
        WorthQueryFeatureRoleRuntime::new(
            FinalPlanarPreserveHandler,
            PlanarFinalOutputProvider::default(),
            (),
        ),
    )
}
