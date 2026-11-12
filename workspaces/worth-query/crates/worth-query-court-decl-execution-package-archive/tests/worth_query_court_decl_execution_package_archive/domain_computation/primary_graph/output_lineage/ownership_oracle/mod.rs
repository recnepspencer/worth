//! Active native ownership court after the real MODE2 Query publication.

use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::base_model::TopologyModelBinding;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::output_contract::PlanarAnchorOutput;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::output_contract::PlanarCreatedOutputs;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::output_contract::PlanarMutationScope;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::output_contract::PlanarOutputs;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_islands::PlanarIslands;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_read::planar_query_definition;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_read::PlanarQuery;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_read::PlanarReadBinding;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_read::PlanarReadResult;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_topology::planar_turn_invariant;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_topology::PlanarSuccessor;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_topology::PositivePlanarTurn;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_topology::UndeclaredPlanarRelation;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::principal::ConsumerPrincipalBinding;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::principal::MappingTarget;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::region_output::RegionOutputBinding;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::region_output::RegionOutputHandler;
use worth_query_decl::facade::application_operation::ApplicationMutationOutputContract;
use worth_query_decl::facade::application_operation::ApplicationMutationOutputRoleDescriptor;
use worth_query_decl::facade::application_operation::WorthQueryApplicationDeclaredOutputRole;
use worth_query_decl::facade::application_operation::WorthQueryApplicationOutputRole;
use worth_query_decl::facade::application_operation::WorthQueryExactlyOneOutput;
use worth_query_decl::facade::application_operation::WorthQueryPreserveOutput;
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_decl::facade::application_schema::ApplicationRelationRef;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph;
use worth_query_execution::facade as runtime_api;
use worth_query_execution::facade::domain_computation::CandidateWriter;
use worth_query_execution::facade::domain_computation::DecisionReader;
use worth_query_execution::facade::domain_computation::HandlerExecutionDenial;
use worth_query_execution::facade::domain_computation::HandlerResult;
use worth_query_execution::facade::domain_computation::MutationHandlerExecutionDenial;
use worth_query_execution::facade::domain_computation::OperationHandler;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationInvariantProjectionAuthority;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationPrincipalKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationReadObservation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationRelationSeed;
use worth_query_execution::facade::domain_computation::WorthQueryConditionalApplicationRuntimeInstallation;
use worth_query_execution::facade::domain_computation::WorthQueryConditionalRuntimeInstallationDenial;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionPlan;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantMutationTarget;
use worth_query_execution::facade::application_invariants::WorthQueryInvariantProjectionWork;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationProjectionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandAdvance;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandSettlement;
use worth_query_execution::facade::domain_computation::WorthQueryOutputSettlementPosture;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenial;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::fixture_values::positive_length::PositiveLength;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::length_binding::TopologyLengthBinding;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationConditionalBinding;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationConditionalPackageContract;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationConditionalProducerAccess;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationContribution;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationContributionContracts;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationContributionSetup;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationProducerBinding;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationProducerProvider;
use worth_query_execution::facade::application_contribution::WorthQueryDecisionContextDependencies;
use worth_query_execution::facade::application_contribution::WorthQueryOutputReadinessContractBuilder;
use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
use worth_query_execution::facade::application_contribution::WorthQueryProducerInputReuseContract;
use worth_query_execution::facade::application_contribution::WorthQueryProducerInvariantRequirement;
use worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture;
use worth_query_execution::facade::application_contribution::WorthQueryProducerOutputFamily;
use worth_query_execution::facade::application_contribution::WorthQueryUndeclaredProducerDenial;
use worth_query_installation::facade as runtime_domain;
use worth_query_installation::worth_query_conditional_node as conditional_node;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::base_model::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::output_contract::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::planar_islands::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::region_output::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::output_producer::*;
#[path = "model.rs"]
mod storage_model;
include!("../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/producer/resources.rs");

/// Reuse the real installed MODE2 producer when another owner needs its evidence.
pub(in crate::domain_computation::primary_graph) fn with_genuine_native_observation<T>(
    inspect: impl FnOnce(
        &dyn Fn(
            worth_runtime_bridge::facade::BridgeConditionalCausalEnvelopeRequest,
        ) -> Result<
            worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope,
            worth_query_execution::facade::causal_inspection::WorthQueryBridgeCausalReportingDenial,
        >,
        &worth_runtime_bridge::facade::BridgeConditionalCausalObservation,
    ) -> T,
) -> T {
    let runtime = installation::install();
    let settled = journey::complete(&runtime);
    let observation = settled
        .readiness_delivery()
        .unwrap()
        .causal_observation_for_test()
        .unwrap();
    let report = |request| {
        runtime
            .runtime_for_test()
            .bridge_for_test()
            .assemble_native_causal_report(request)
    };
    inspect(&report, observation)
}

#[path = "admission/mod.rs"]
pub mod admission;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/base_model.rs"]
pub mod base_model;
#[path = "birth/mod.rs"]
pub mod birth;
#[path = "commit_interruption/mod.rs"]
pub mod commit_interruption;
#[path = "facts/mod.rs"]
pub mod facts;
#[path = "fixture_values/mod.rs"]
pub mod fixture_values;
#[path = "installation/mod.rs"]
pub mod installation;
#[path = "journey/mod.rs"]
pub mod journey;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/length_binding.rs"]
pub mod length_binding;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/mutation/output_contract.rs"]
pub mod output_contract;
#[path = "output_producer/mod.rs"]
pub mod output_producer;
#[path = "own_write/mod.rs"]
pub mod own_write;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_invariant.rs"]
pub mod planar_invariant;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_islands.rs"]
pub mod planar_islands;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_read.rs"]
pub mod planar_read;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_topology.rs"]
pub mod planar_topology;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/principal.rs"]
pub mod principal;
#[path = "readiness/mod.rs"]
pub mod readiness;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/region_output.rs"]
pub mod region_output;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_values/src/integer_orientation.rs"]
pub mod worth_query_consumer_values;

#[path = "optional_aspects/mod.rs"]
pub mod optional_aspects;
#[path = "producer_observation/mod.rs"]
pub mod producer_observation;
