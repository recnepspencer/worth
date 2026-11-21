pub use worth_query_execution_query::facade::domain_computation::primary_graph::product_activation::{
    inspect_selected_program, WorthQuerySelectedProgramInspection,
    WorthQuerySelectedProgramInspectionDenial,
};
mod publication;
pub(in crate::domain_computation::primary_graph) use publication::WorthQueryProductConditionalPublicationDenial;
