//! Sealed static compute transport; owner completion is a separate value.
use std::any::Any;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::PreparedCompute;

impl PreparedCompute {
    pub(in crate::domain_computation::primary_graph) fn declared_test_request(
        rank: usize,
        fails: bool,
    ) -> Self {
        Self::new(move |member| {
            let (_, poll) = member.into_parts();
            poll().unwrap();
            (rank, fails)
        })
    }
}

mod court_doors_for_test;
