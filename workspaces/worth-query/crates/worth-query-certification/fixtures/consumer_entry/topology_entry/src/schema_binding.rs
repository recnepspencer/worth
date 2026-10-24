//! The consumer fixture's schema policy: what each installed fixture schema
//! declares about the planar producers and operations it installs. Every
//! fixture schema is also a shared topology model schema, with the invariant
//! work its policy declares.
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_host::facade::application_contribution::{
    WorthQueryDecisionContextDependencies, WorthQueryProducerInputReuseContract,
};

use super::base_model::TopologyModelBinding;

pub trait TopologySchemaBinding: ApplicationSchema {
    /// Isolated consumer variants can require the real planar producer binding
    /// to belong to their application program; existing schemas keep their policy.
    const PLANAR_REQUIRES_APPLICATION_PROGRAM: bool = false;
    const PLANAR_EDIT_PROJECTION_WORK: Option<usize> = Some(8_192);
    fn planar_invariant_work() -> u64 {
        super::planar_topology::PLANAR_CHECKED_WORK
    }
    const ROOT_INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
    /// The final producer's decision-reuse declaration is part of each installed
    /// fixture schema, so opt-out journeys use the ordinary production installer.
    const FINAL_INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}

impl<Schema: TopologySchemaBinding> TopologyModelBinding for Schema {
    fn planar_invariant_work() -> u64 {
        <Schema as TopologySchemaBinding>::planar_invariant_work()
    }
}
