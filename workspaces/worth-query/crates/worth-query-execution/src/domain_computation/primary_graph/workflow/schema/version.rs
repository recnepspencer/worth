//! Stable interpretation version for authoritative workflow graph facts.
//!
//! Expression conditions add the `condition-operands` node field without a
//! version change: every version-1 record still reads exactly as before (a
//! condition with a binding and no operands is the migrated form), expression
//! records use a disjoint field shape, and adoption requires the exact
//! version to read a lineage, so a bump would make every definition already
//! published unreadable.

pub(in crate::domain_computation::primary_graph) const WORKFLOW_FACT_PROTOCOL_VERSION: u64 = 1;
pub(in crate::domain_computation::primary_graph) const WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION:
    u64 = 2;
