//! Intrinsic custody of the actual RelationalCheckpointLineageEventPage directory backing.
use super::RelationalCheckpointLineageEventPage;
use crate::execution::RelationalExecutionDenialCause as Cause;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug)]
struct Backing {
    rows: Vec<RelationalCheckpointLineageEventPage>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct CheckpointLineageEventPages(Option<Arc<Backing>>);
pub(crate) struct CheckpointLineageEventPagesBuilder {
    rows: Vec<RelationalCheckpointLineageEventPage>,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CheckpointLineageEventPages {
    pub(crate) fn from_ordinary(rows: Vec<RelationalCheckpointLineageEventPage>) -> Self {
        if rows.is_empty() {
            Self::default()
        } else {
            Self(Some(Arc::new(Backing {
                rows,
                _memory: None,
            })))
        }
    }
    pub(crate) fn ordinary_owned_copy(&self) -> Vec<RelationalCheckpointLineageEventPage> {
        self.rows().to_vec()
    }
    pub(crate) fn allocate_for_request(
        capacity: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CheckpointLineageEventPagesBuilder, Cause> {
        crate::execution::check_request(request)?;
        if capacity == 0 {
            return Ok(CheckpointLineageEventPagesBuilder {
                rows: Vec::new(),
                bytes: 0,
                _memory: None,
            });
        }
        let control = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .0
            .pad_to_align()
            .size();
        let rows = Layout::array::<RelationalCheckpointLineageEventPage>(capacity)
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .size();
        let bytes = control
            .checked_add(rows)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        Ok(CheckpointLineageEventPagesBuilder {
            rows: Vec::with_capacity(capacity),
            bytes,
            _memory: memory,
        })
    }
    fn rows(&self) -> &[RelationalCheckpointLineageEventPage] {
        self.0.as_ref().map_or(&[], |b| b.rows.as_slice())
    }
}
impl CheckpointLineageEventPagesBuilder {
    fn admit_nested(
        &mut self,
        bytes: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        crate::execution::check_request(request)?;
        let total = self
            .bytes
            .checked_add(u64::try_from(bytes).map_err(|_| Cause::ChargedBytesOverflow)?)
            .ok_or(Cause::ChargedBytesOverflow)?;
        if let Some(memory) = &mut self._memory {
            memory.resize(total).map_err(|cause| {
                crate::execution::denial::lease_denial(
                    worth_execution::LeaseDenial::MemoryExhausted(cause),
                )
            })?;
        } else {
            self._memory = request
                .reserve_retained_memory(total)
                .map_err(Cause::from)?;
        }
        self.bytes = total;
        Ok(())
    }
    fn room(&self) -> Result<(), Cause> {
        if self.rows.len() == self.rows.capacity() {
            Err(Cause::ChargedBytesOverflow)
        } else {
            Ok(())
        }
    }
    pub(crate) fn seal(self) -> CheckpointLineageEventPages {
        if self.rows.is_empty() {
            return CheckpointLineageEventPages::default();
        }
        CheckpointLineageEventPages(Some(Arc::new(Backing {
            rows: self.rows,
            _memory: self._memory,
        })))
    }
}
impl std::ops::Deref for CheckpointLineageEventPages {
    type Target = [RelationalCheckpointLineageEventPage];
    fn deref(&self) -> &Self::Target {
        self.rows()
    }
}
impl PartialEq for CheckpointLineageEventPages {
    fn eq(&self, other: &Self) -> bool {
        self.rows() == other.rows()
    }
}
impl Eq for CheckpointLineageEventPages {}
impl Serialize for CheckpointLineageEventPages {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.rows().serialize(s)
    }
}
impl<'de> Deserialize<'de> for CheckpointLineageEventPages {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Vec::<RelationalCheckpointLineageEventPage>::deserialize(d).map(Self::from_ordinary)
    }
}
impl<'a> IntoIterator for &'a CheckpointLineageEventPages {
    type Item = &'a RelationalCheckpointLineageEventPage;
    type IntoIter = std::slice::Iter<'a, RelationalCheckpointLineageEventPage>;
    fn into_iter(self) -> Self::IntoIter {
        self.rows().iter()
    }
}

impl CheckpointLineageEventPagesBuilder {
    /// Carry an already selected immutable page, funding this directory's copy.
    pub(crate) fn push_carried(
        &mut self,
        page: &RelationalCheckpointLineageEventPage,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), super::RelationalNativeCheckpointCaptureDenial> {
        use super::RelationalNativeCheckpointCaptureDenial as Denial;
        self.room().map_err(Denial::Request)?;
        let ids = Layout::array::<u64>(page.event_ids.len())
            .map_err(|_| Denial::Request(Cause::ChargedBytesOverflow))?
            .size();
        let routes = Layout::array::<super::RelationalLineageEventPageRoute>(page.routes.len())
            .map_err(|_| Denial::Request(Cause::ChargedBytesOverflow))?
            .size();
        let mut bytes = ids
            .checked_add(routes)
            .ok_or(Denial::Request(Cause::ChargedBytesOverflow))?;
        for route in &page.routes {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            bytes = bytes
                .checked_add(route.branch_id.0.len())
                .ok_or(Denial::Request(Cause::ChargedBytesOverflow))?;
        }
        self.admit_nested(bytes, request).map_err(Denial::Request)?;
        self.rows.push(page.clone());
        Ok(())
    }
    pub(crate) fn push<'a>(
        &mut self,
        member: super::super::medium::RelationalDurableMemberLocator,
        events: impl Clone + Iterator<Item = &'a crate::lineage::data::LineageEventRecord>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), super::RelationalNativeCheckpointCaptureDenial> {
        use super::{
            RelationalLineageEventPageRoute, RelationalNativeCheckpointCaptureDenial as Denial,
            LINEAGE_NODE_RANGE_WIDTH,
        };
        self.room().map_err(Denial::Request)?;
        let mut routes = worth_execution::ExecutionOrderedMap::<
            (&crate::history::data::BranchId, u64),
            (),
        >::default();
        let (mut first_node, mut last_node, mut count, mut last_id) =
            (u64::MAX, 0u64, 0usize, 0u64);
        for event in events.clone() {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            if event.event_id() <= last_id || event.event_id() >= u64::MAX - 1 {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            last_id = event.event_id();
            count = count
                .checked_add(1)
                .ok_or(Denial::Request(Cause::ChargedBytesOverflow))?;
            let mut participant = false;
            for node in event.sources().iter().chain(event.targets()) {
                if node.0 == 0 || node.0 >= u64::MAX - 1 {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                participant = true;
                first_node = first_node.min(node.0);
                last_node = last_node.max(node.0);
                let key = (event.branch_id(), (node.0 - 1) / LINEAGE_NODE_RANGE_WIDTH);
                if routes.get(&key).is_none() {
                    routes
                        .insert_cloned_native_for_request(&key, &(), *request)
                        .map_err(|cause| {
                            crate::durability::checkpoint_history::capture::capture_stop(
                                crate::durability::checkpoint_history::capture::map_stop(cause),
                            )
                        })?;
                }
            }
            if !participant {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
        }
        if count == 0 || count > super::LINEAGE_EVENT_PAGE_ROWS {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        let ids = Layout::array::<u64>(count)
            .map_err(|_| Denial::Request(Cause::ChargedBytesOverflow))?
            .size();
        let rows = Layout::array::<RelationalLineageEventPageRoute>(routes.len())
            .map_err(|_| Denial::Request(Cause::ChargedBytesOverflow))?
            .size();
        let mut bytes = ids
            .checked_add(rows)
            .ok_or(Denial::Request(Cause::ChargedBytesOverflow))?;
        for ((branch, _), _) in routes.iter() {
            bytes = bytes
                .checked_add(branch.0.len())
                .ok_or(Denial::Request(Cause::ChargedBytesOverflow))?;
        }
        self.admit_nested(bytes, request).map_err(Denial::Request)?;
        let mut event_ids = Vec::with_capacity(count);
        for event in events {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            event_ids.push(event.event_id());
        }
        let mut owned_routes = Vec::with_capacity(routes.len());
        for ((branch, node_range), _) in routes.iter() {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            owned_routes.push(RelationalLineageEventPageRoute {
                branch_id: (**branch).clone(),
                node_range: *node_range,
            });
        }
        self.rows.push(RelationalCheckpointLineageEventPage {
            first_node,
            last_node,
            event_ids,
            routes: owned_routes,
            member,
        });
        Ok(())
    }
}
