//! One complete definition supplies the real schema, program and installation projections.
//! Generated configuration retains the original concrete handler type.
//! ```compile_fail,E0308
//! use worth_query_topology_entry::*;
//! worth_query_decl::facade::worth_query_application! {
//!     pub Schema { owner: "wrong-feature-handler", version: (1, 0), contributions: [TopologyContribution], }
//! }
//! impl TopologySchemaBinding for Schema {}
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
