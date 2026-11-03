//! Actual Query admission and fresh disclosure for one real MODE2 completion.

use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::journey::complete;

use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::Application;
use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::OwnerSchema;
use worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::ANCHOR;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;
use std::sync::Arc;
use std::time::Duration;
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
#[test]
fn real_mode2_mixed_query_publication_attests_neither_whole_nor_created_subset() {
    let runtime = worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::install();
    let settled = complete(&runtime);
    worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::admission::refuse_mixed_and_created_subset(&runtime, &settled);
    worth_query_testkit_execution_decl::domain_computation::primary_graph::output_lineage::ownership_oracle::birth::admit_genuine_all_create(&runtime);
}
