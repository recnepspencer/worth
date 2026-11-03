//! Lifecycle order does not collapse exact rows or cross output families.

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use std::cmp::Ordering;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
use worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture;
pub fn family_declarations() -> [[(TypeId, &'static str); 2]; 2] {
    struct A;
    struct B;
    [
        [
            (TypeId::of::<A>(), "alpha-output"),
            (TypeId::of::<B>(), "zulu-output"),
        ],
        [
            (TypeId::of::<B>(), "alpha-output"),
            (TypeId::of::<A>(), "zulu-output"),
        ],
    ]
}
pub fn initial() -> WorthQueryOutputDemandKey {
    let mut key = crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("initial", 7, 1);
    (*key.applicability_mut_for_test()) = WorthQueryProducerApplicability::new(
        "registry-fixture",
        WorthQueryProducerLifecyclePosture::Initial,
    );
    key
}
