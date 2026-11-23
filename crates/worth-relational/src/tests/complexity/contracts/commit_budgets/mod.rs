use crate::facade::history::BranchId;
use crate::facade::transactions::BulkRelationCreateIntent;
use crate::facade::transactions::CommitTopology;
use crate::tests::support::*;

mod bulk_mutation_admission;
mod contract_registry;
mod fixtures;
mod partition_local_commit;
mod preparation_packetization;
mod relation_identity_validation;
mod relation_integrity_counters;
mod replay_verification;
mod schema_continuity_boundaries;
mod unique_invariant_lookup;

use fixtures::*;
