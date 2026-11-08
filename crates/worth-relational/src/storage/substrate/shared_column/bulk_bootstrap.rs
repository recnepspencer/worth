//! Dense construction retains each node bill independently of temporary levels.
use super::node::{ColumnNode, ColumnStorage};
use super::{SharedColumn, StoredValue, PAGE_LEN};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::storage::substrate::StorageAllocation as Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

struct Level<T: StoredValue> {
    values: Vec<Arc<ColumnNode<T>>>,
    memory: Option<ExecutionMemoryReservation>,
}
impl<T: StoredValue> Level<T> {
    fn allocate(count: usize, request: Option<&ExecutionRequest<'_, '_>>) -> Result<Self, Cause> {
        charge(request, bytes::<Arc<ColumnNode<T>>>(count)?)?;
        let memory = crate::storage::substrate::node_memory::reserve(
            request,
            crate::storage::substrate::node_memory::array::<Arc<ColumnNode<T>>>(count),
        )?;
        Ok(Self {
            values: Vec::with_capacity(count),
            memory,
        })
    }
}
impl<T: StoredValue> SharedColumn<T> {
    /// The incoming owned values/input Vec retain their caller's original
    /// custody. This operation admits only newly constructed nodes and levels.
    pub(crate) fn from_dense_values_for_request(
        values: Vec<T::Value>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        Self::dense_using(values, Some(request))
    }
    pub(super) fn from_dense_values(values: Vec<T::Value>) -> Self {
        Self::dense_using(values, None).expect("ordinary dense construction has no request refusal")
    }
    fn dense_using(
        values: Vec<T::Value>,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Self, Cause> {
        charge(request, bytes::<T::Value>(values.len())?)?;
        Self::dense_carriers_using(values.into_iter().map(T::from_value), request)
    }
    pub(crate) fn from_dense_carriers(values: Vec<T>) -> Self {
        Self::dense_carriers_using(values.into_iter(), None)
            .expect("ordinary carrier construction has no request refusal")
    }
    pub(crate) fn from_dense_carriers_for_request(
        values: Vec<T>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        Self::dense_carriers_using(values.into_iter(), Some(request))
    }
    fn dense_carriers_using(
        values: impl ExactSizeIterator<Item = T>,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Self, Cause> {
        if let Some(request) = request {
            request.consult()?;
        }
        let len = values.len();
        if len == 0 {
            return Ok(Self::default());
        }
        let mut input = values;
        let mut level = Level::allocate(len.div_ceil(PAGE_LEN), request)?;
        while input.len() != 0 {
            let work = Arc::<ColumnNode<T>>::layout_bytes()
                .checked_add(bytes::<Option<T>>(PAGE_LEN)?)
                .and_then(|n| n.checked_add(bytes::<T>(input.len().min(PAGE_LEN)).ok()?))
                .ok_or(Cause::ChargedBytesOverflow)?;
            charge(request, work)?;
            let mut page = ColumnNode::allocate(0, request)?;
            let node = Arc::get_mut(&mut page).expect("new page is uniquely owned");
            let ColumnStorage::Page(rows) = &mut node.storage else {
                unreachable!("leaf allocator produced a page")
            };
            for (index, value) in input.by_ref().take(PAGE_LEN).enumerate() {
                rows[index] = Some(value);
                node.value_count += 1;
            }
            level.values.push(page);
        }
        drop(input);
        let mut height = 0;
        while level.values.len() > 1 {
            let mut parents = Level::allocate(level.values.len().div_ceil(2), request)?;
            let Level { values, memory } = level;
            let mut children = values.into_iter();
            while let Some(left) = children.next() {
                // The initial branch and its final counters/children both initialize.
                charge(
                    request,
                    Arc::<ColumnNode<T>>::layout_bytes()
                        .checked_add(bytes::<ColumnNode<T>>(1)?)
                        .ok_or(Cause::ChargedBytesOverflow)?,
                )?;
                let mut parent = ColumnNode::allocated_branch(Some(left), request)?;
                let node = Arc::get_mut(&mut parent).expect("new parent is uniquely owned");
                let right = children.next();
                let ordinary = ColumnNode::branch(
                    match &mut node.storage {
                        ColumnStorage::Branch(children) => children[0].take(),
                        _ => unreachable!("branch allocator"),
                    },
                    right,
                );
                node.storage = ordinary.storage;
                node.node_count = ordinary.node_count;
                node.page_count = ordinary.page_count;
                node.value_count = ordinary.value_count;
                parents.values.push(parent);
            }
            drop(children);
            drop(memory);
            level = parents;
            height += 1;
        }
        let root = level.values.pop();
        drop(level);
        Ok(Self {
            root,
            height,
            len,
            default: None,
        })
    }
}

fn bytes<T>(count: usize) -> Result<u64, Cause> {
    crate::storage::substrate::node_memory::array::<T>(count).ok_or(Cause::ChargedBytesOverflow)
}
fn charge(request: Option<&ExecutionRequest<'_, '_>>, units: u64) -> Result<(), Cause> {
    let Some(request) = request else {
        return Ok(());
    };
    let mut outcome = None;
    let scope = request.run_native_with_checkpoint(|context| {
        outcome = Some(crate::execution::request_stage::checkpoint(context, units));
    });
    match (scope, outcome) {
        (_, Some(Err(cause))) => Err(worth_execution::WorkCeilingDenial::Stopped(cause).into()),
        (Err(cause), _) => Err(cause.into()),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed dense work admission executes its callback"),
    }
}
