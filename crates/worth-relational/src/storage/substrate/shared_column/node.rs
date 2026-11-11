//! Actual column node/page backing; element carriers have independent ownership.
use super::PAGE_LEN;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::storage::substrate::{StorageAllocation as Arc, StoredValue};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug, Clone)]
pub(super) enum ColumnStorage<T: StoredValue> {
    Page(Vec<Option<T>>),
    Branch([Option<Arc<ColumnNode<T>>>; 2]),
}
#[derive(Debug)]
pub(super) struct ColumnNode<T: StoredValue> {
    pub(super) storage: ColumnStorage<T>,
    pub(super) node_count: usize,
    pub(super) page_count: usize,
    pub(super) value_count: usize,
    pub(super) memory: Option<ExecutionMemoryReservation>,
}
impl<T: StoredValue> Clone for ColumnNode<T> {
    fn clone(&self) -> Self {
        // Ordinary image/maintenance COW makes an independent unqualified node.
        Self {
            storage: self.storage.clone(),
            node_count: self.node_count,
            page_count: self.page_count,
            value_count: self.value_count,
            memory: None,
        }
    }
}
impl<T: StoredValue> ColumnNode<T> {
    fn bytes(page: bool) -> Option<u64> {
        Arc::<Self>::layout_bytes().checked_add(if page {
            crate::storage::substrate::node_memory::array::<Option<T>>(PAGE_LEN)?
        } else {
            0
        })
    }
    pub(super) fn allocate(
        height: usize,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Arc<Self>, Cause> {
        let memory =
            crate::storage::substrate::node_memory::reserve(request, Self::bytes(height == 0))?;
        let mut node = if height == 0 {
            Self {
                storage: ColumnStorage::Page(vec![None; PAGE_LEN]),
                node_count: 1,
                page_count: 1,
                value_count: 0,
                memory: None,
            }
        } else {
            Self::branch(None, None)
        };
        node.memory = memory;
        Ok(Arc::new(node))
    }
    pub(super) fn allocated_branch(
        left: Option<Arc<Self>>,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Arc<Self>, Cause> {
        let memory = crate::storage::substrate::node_memory::reserve(request, Self::bytes(false))?;
        let mut node = Self::branch(left, None);
        node.memory = memory;
        Ok(Arc::new(node))
    }
    pub(super) fn branch(left: Option<Arc<Self>>, right: Option<Arc<Self>>) -> Self {
        let children = [left, right];
        Self {
            node_count: 1 + children
                .iter()
                .flatten()
                .map(|node| node.node_count)
                .sum::<usize>(),
            page_count: children.iter().flatten().map(|node| node.page_count).sum(),
            value_count: children.iter().flatten().map(|node| node.value_count).sum(),
            storage: ColumnStorage::Branch(children),
            memory: None,
        }
    }
    pub(super) fn get(&self, index: usize, height: usize) -> Option<&T> {
        match &self.storage {
            ColumnStorage::Page(values) => values[index % PAGE_LEN].as_ref(),
            ColumnStorage::Branch(children) => children[branch_index(index, height)]
                .as_ref()?
                .get(index, height - 1),
        }
    }
    pub(super) fn detach<'a>(
        this: &'a mut Arc<Self>,
        copied_bytes: &mut u64,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<&'a mut Self, Cause> {
        if let Some(request) = request {
            request.consult()?;
        }
        if Arc::get_mut(this).is_none() {
            let page = matches!(this.storage, ColumnStorage::Page(_));
            let memory =
                crate::storage::substrate::node_memory::reserve(request, Self::bytes(page))?;
            let page_bytes = if page {
                crate::storage::substrate::node_memory::array::<Option<T>>(PAGE_LEN)
                    .ok_or(Cause::ChargedBytesOverflow)?
            } else {
                0
            };
            let node = Self {
                storage: this.storage.clone(),
                node_count: this.node_count,
                page_count: this.page_count,
                value_count: this.value_count,
                memory,
            };
            *copied_bytes =
                copied_bytes.saturating_add(std::mem::size_of::<Self>() as u64 + page_bytes);
            *this = Arc::new(node);
        }
        Ok(Arc::get_mut(this)
            .expect("the actual unique node or newly allocated node remains exclusive"))
    }
    pub(super) fn carrier_mut<'a>(
        this: &'a mut Arc<Self>,
        index: usize,
        height: usize,
        copied: &mut u64,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<&'a mut T, Cause> {
        let node = Self::detach(this, copied, request)?;
        match &mut node.storage {
            ColumnStorage::Page(values) => Ok(values[index % PAGE_LEN]
                .as_mut()
                .expect("the selected carrier was materialized")),
            ColumnStorage::Branch(children) => Self::carrier_mut(
                children[branch_index(index, height)]
                    .as_mut()
                    .expect("the selected child was materialized"),
                index,
                height - 1,
                copied,
                request,
            ),
        }
    }
    pub(super) fn set(
        &mut self,
        index: usize,
        height: usize,
        value: T,
        copied: &mut u64,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<(), Cause> {
        match &mut self.storage {
            ColumnStorage::Page(values) => {
                if values[index % PAGE_LEN].is_none() {
                    self.value_count += 1;
                }
                values[index % PAGE_LEN] = Some(value);
            }
            ColumnStorage::Branch(children) => {
                let selected = &mut children[branch_index(index, height)];
                if selected.is_none() {
                    *selected = Some(Self::allocate(height - 1, request)?);
                }
                Self::detach(selected.as_mut().expect("installed child"), copied, request)?.set(
                    index,
                    height - 1,
                    value,
                    copied,
                    request,
                )?;
                self.node_count = 1 + children
                    .iter()
                    .flatten()
                    .map(|node| node.node_count)
                    .sum::<usize>();
                self.page_count = children.iter().flatten().map(|node| node.page_count).sum();
                self.value_count = children.iter().flatten().map(|node| node.value_count).sum();
            }
        }
        Ok(())
    }
    pub(super) fn into_values(node: Arc<Self>, output: &mut Vec<T::Value>) {
        let node = Arc::unwrap_or_clone(node);
        match node.storage {
            ColumnStorage::Page(values) => {
                output.extend(values.into_iter().flatten().map(T::into_value))
            }
            ColumnStorage::Branch(children) => {
                for child in children.into_iter().flatten() {
                    Self::into_values(child, output);
                }
            }
        }
    }
}
fn branch_index(index: usize, height: usize) -> usize {
    ((index / PAGE_LEN) >> (height - 1)) & 1
}
