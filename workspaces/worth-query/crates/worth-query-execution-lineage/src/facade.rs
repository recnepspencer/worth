//! Crate facade for primary-graph lineage, output demand, invariant projection,
//! partitioned computation, handlers, checkpoints, and domain semantics.

pub mod domain_computation {
    pub use crate::domain_computation::{
        application_aftermath, authorization, execution_observation, primary_graph,
        product_publication,
    };
}
pub use crate::domain_computation::{application_aftermath, authorization, primary_graph};
pub use worth_query_execution_obligations::facade::aftermath::correction_material;
pub mod ports {
    pub use crate::ports::*;
}
pub mod test_read_stops {
    pub use crate::test_read_stops::*;
}
