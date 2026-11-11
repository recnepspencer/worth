use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Instant;

use super::performance_support::*;
use crate::facade::config::RelationalRuntimeProfile;
use crate::facade::history::BranchId;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::facade::inspection::{
    ConnectivityInspectionBudget, ConnectivityInspectionRequest, InspectionRecordClass,
    InspectionScope, KindInspectionRequest, RecentCommitInspectionRequest,
    StructuralIdentityQueryRequest,
};
use crate::facade::query::{
    DeterministicQueryPlanKey, IndexParityMode, PlannedQueryPacket, QueryAccessContract,
    QueryExecutionShape, QueryLocalityClass, QueryOrderingContract, QueryScope,
    ReductionDiscipline,
};
use crate::facade::runtime::EntityRecordProjection;
use crate::facade::symbols::Symbol;
use crate::tests::support::*;
use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet,
};

mod cad_topology_matrix;
mod downstream_runtime_mock_matrix;
mod downstream_runtime_support;
mod game_engine_matrix;
mod geometry_artifact_decomposition_matrix;
mod inspection_budget_matrix;
mod invariant_materialization_matrix;
mod invariant_support;
mod mixed_load_matrix;
mod performance_harness_matrix;
mod profile_matrix;
mod query_packet_matrix;
mod retention_reclaim_matrix;
mod rocketship_bulk_intents;
mod rocketship_layout;
mod rocketship_pseudorealistic;
mod rocketship_scale_matrix;
mod snapshot_materialization_matrix;

use downstream_runtime_support::*;
use invariant_support::*;
use rocketship_bulk_intents::*;
use rocketship_layout::*;
use rocketship_pseudorealistic::*;
