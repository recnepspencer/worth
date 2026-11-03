//! Scheduling interruption preserves registry custody, not output currentness.

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::recovery_posture;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::interest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputSchedulingResult;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refresh_predecessor::OutputRefreshPredecessor;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputAdvancement;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture as Posture;
use worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::fixture::Fixture;
#[derive(Clone, Copy, Debug)]
pub enum Finish {
    Ordinary,
    Selected,
}
impl Finish {
    pub fn stop(
        self,
        fixture: &Fixture,
        kind: WorthQueryOutputDemandDenialKind,
    ) -> WorthQueryOutputDemandDenial {
        let mut denial = WorthQueryOutputDemandDenial::new_for_test(kind, "scheduling stopped".to_owned());
        (*denial.readmission_failure_mut_for_test()) = Some("original scheduling refusal");
        let mut result = Err(denial);
        match self {
            Self::Ordinary => {
                let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) =
                    fixture.registry.begin_for_test(&fixture.interest)
                else {
                    panic!("the ordinary caller must claim scheduling");
                };
                fixture
                    .registry
                    .finish_scheduling_for_test(&fixture.interest, source, &mut result);
            }
            Self::Selected => {
                let (admission, finish) =
                    fixture.registry.begin_admitted_for_test(&fixture.interest).unwrap();
                let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) = admission else {
                    panic!("the selected caller must claim scheduling");
                };
                finish
                    .expect("selected scheduling prepares its real finish")
                    .finish_for_test(source, &mut result);
            }
        }
        result
            .err()
            .expect("scheduling keeps the full caller denial")
    }
}

#[path = "bridge_deferral/mod.rs"]
pub mod bridge_deferral;
#[path = "fixture/mod.rs"]
pub mod fixture;
