//! One complete definition supplies the real schema, program and installation projections.
//! Generated configuration retains the original concrete handler type.
//! ```compile_fail,E0308
//! use worth_query_topology_entry::*;
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! pub struct Schema;
//! impl worth_query_decl::facade::application_schema::fragment::Fragment for Schema {
//!  const NAME: &'static str = "Schema";
//!  type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
//!  type Members = (TopologyContribution,);
//! }
//! impl worth_query_decl::facade::application_schema::ApplicationSchema for Schema {
//!  const OWNER: &'static str = "wrong-feature-handler"; const NAME: &'static str = "Schema";
//!  const MAJOR: u32 = 1; const MINOR: u32 = 0;
//!  fn declaration() -> Result<worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>, worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial> { worth_query_decl::facade::application_schema::fragment::declare(Self) }
//!  fn visit_correction<V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>>(source: std::any::TypeId, visitor: V) -> V::Output { worth_query_decl::facade::application_schema::fragment::visit_correction(Self, source, visitor) }
//! }
//! impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition for Schema { type Contributions = (TopologyContribution,); }
//!
//! impl TopologySchemaBinding for Schema { type PrincipalFragment = worth_query_topology_entry::PrincipalRecords; }
//! let mut runtime = CompletePlanarFinalRuntime::<Schema>::new(planar_final_feature_runtime());
//! runtime.lifecycle.initial.as_mut().unwrap().handler = Some(42_u64);
//! ```
use super::*;
use crate::{PlanarFinalOutputFeature, TopologySchemaBinding};

// compiled-example: begin
worth_query_host::facade::worth_query_application_contribution! {
    pub feature PlanarFinalFeatureDefinition<Schema: TopologySchemaBinding>
        for PlanarFinalOutputFeature => CompletePlanarFinalRuntime {
        schema: |schema| { schema };
        provides(crate::PlanarFinalBodyOutput);
        required_pair lifecycle(
            PlanarFinalFeatureRoles<Schema>, FinalPlanarMutationHandler, FinalPlanarPreserveHandler
        ) {
            initial_operation: PublishFinalPlanarOutput::reference::<Schema>().definition()
                .no_external_effect().no_aftermath().finish();
            preserve_operation: PreserveFinalPlanarOutput::reference::<Schema>().definition()
                .no_external_effect().no_aftermath().finish();
        };
    }
}
// compiled-example: end
