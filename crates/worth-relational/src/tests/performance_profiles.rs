use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Instant;

use super::domains::fintech::{
    perf_capture_baseline_observability, perf_capture_intraday_risk_probe,
    perf_capture_post_mutation_observability, perf_capture_trade_correction_probe,
    perf_correct_trade_correction, perf_emit_trade_correction_audit, perf_open_analysis_branch,
    perf_stress_intraday_risk, setup_intraday_risk_perf_world, setup_trade_correction_perf_world,
};
use super::performance_support::*;
use crate::capabilities::DurabilityRead;
use crate::facade::config::{AdjacencyBackend, RelationalRuntimeProfile};
use crate::facade::history::BranchId;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::facade::inspection::{
    ConnectivityInspectionBudget, ConnectivityInspectionRequest, InspectionRecordClass,
    InspectionScope, KindInspectionRequest, RecentCommitInspectionRequest,
    StructuralIdentityQueryRequest,
};
use crate::facade::lineage::{
    HistoricalResolutionBoundednessBasis, HistoricalResolutionRequest, LineageDivergenceRequest,
    LineageDivergenceTraversalBasis,
};
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::facade::query::{
    DeterministicQueryPlanKey, IndexParityMode, PlannedQueryPacket, QueryAccessContract,
    QueryExecutionShape, QueryLocalityClass, QueryOrderingContract, QueryScope,
    ReductionDiscipline,
};
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};
use crate::facade::runtime::{CompiledArtifactAuthorityStatus, EntityRecordProjection};
use crate::facade::symbols::Symbol;
use crate::replay::data::digest_diagnostics_surface;
use crate::tests::support::*;
use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet,
};

mod artifact_recoverability_matrix;
mod cad_topology_matrix;
mod chip_simulator_matrix;
mod commit_delta_matrix;
mod downstream_runtime_mock_matrix;
mod downstream_runtime_support;
mod durability_append_matrix;
mod game_engine_matrix;
mod geometry_artifact_decomposition_matrix;
mod geometry_kernel_matrix;
mod hot_cold_path_matrix;
mod index_parity_matrix;
mod inspection_budget_matrix;
mod invariant_materialization_matrix;
mod invariant_support;
mod measurement_support;
mod merge_lineage_matrix;
mod mixed_load_matrix;
mod performance_harness_matrix;
mod profile_matrix;
mod query_packet_matrix;
mod recoverability_policy_matrix;
mod replay_recovery_matrix;
mod retention_reclaim_matrix;
mod rocketship_bulk_intents;
mod rocketship_layout;
mod rocketship_pseudorealistic;
mod rocketship_scale_matrix;
mod snapshot_materialization_matrix;
mod sustained_load_matrix;
mod workflow_matrix;

use downstream_runtime_support::*;
use invariant_support::*;
use measurement_support::*;
use rocketship_bulk_intents::*;
use rocketship_layout::*;
use rocketship_pseudorealistic::*;
