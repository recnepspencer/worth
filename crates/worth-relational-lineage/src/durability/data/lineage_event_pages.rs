//! The immutable event payload and its node-range routing directory.
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::lineage::data::events::LineageEventRecord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;

pub const LINEAGE_NODE_RANGE_WIDTH: u64 = 256;
pub const LINEAGE_EVENT_PAGE_ROWS: usize = 256;
pub const LINEAGE_EVENT_PAGE_BYTES: u64 = 1 << 20;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalLineageEventPageRoute {
    pub branch_id: BranchId,
    pub node_range: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCheckpointLineageEventPage {
    pub first_node: u64,
    pub last_node: u64,
    pub event_ids: Vec<u64>,
    pub routes: Vec<RelationalLineageEventPageRoute>,
    pub member: RelationalDurableMemberLocator,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCheckpointLineageEventRow {
    pub event: Arc<LineageEventRecord>,
    pub publication_commit_id: CommitId,
}
impl RelationalCheckpointLineageEventPage {
    pub fn from_rows(
        member: RelationalDurableMemberLocator,
        rows: &[RelationalCheckpointLineageEventRow],
    ) -> Option<Self> {
        if rows.is_empty() || rows.len() > LINEAGE_EVENT_PAGE_ROWS {
            return None;
        }
        let mut routes = BTreeSet::new();
        let mut first_node = u64::MAX;
        let mut last_node = 0;
        let mut event_ids = Vec::new();
        for row in rows {
            if row.event.event_id() == 0
                || row.event.event_id() >= u64::MAX - 1
                || event_ids
                    .last()
                    .is_some_and(|previous| *previous >= row.event.event_id())
            {
                return None;
            }
            event_ids.push(row.event.event_id());
            let mut participant = false;
            for node in row.event.sources().iter().chain(row.event.targets()) {
                if node.0 == 0 || node.0 >= u64::MAX - 1 {
                    return None;
                }
                participant = true;
                first_node = first_node.min(node.0);
                last_node = last_node.max(node.0);
                routes.insert(RelationalLineageEventPageRoute {
                    branch_id: row.event.branch_id().clone(),
                    node_range: (node.0 - 1) / LINEAGE_NODE_RANGE_WIDTH,
                });
            }
            if !participant {
                return None;
            }
        }
        Some(Self {
            first_node,
            last_node,
            event_ids,
            routes: routes.into_iter().collect(),
            member,
        })
    }
}
