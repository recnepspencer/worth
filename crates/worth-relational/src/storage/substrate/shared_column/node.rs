use super::PAGE_LEN;
use crate::storage::substrate::StorageAllocation as Arc;

#[derive(Debug)]
pub(super) enum ColumnStorage<T: Clone> {
    Page(Vec<Option<Arc<T>>>),
    Branch([Option<Arc<ColumnNode<T>>>; 2]),
}

impl<T: Clone> Clone for ColumnStorage<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Page(values) => {
                let mut copied = Vec::with_capacity(PAGE_LEN);
                copied.extend(values.iter().cloned());
                Self::Page(copied)
            }
            Self::Branch(children) => Self::Branch(children.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ColumnNode<T: Clone> {
    pub(super) storage: ColumnStorage<T>,
    pub(super) node_count: usize,
    pub(super) page_count: usize,
    pub(super) value_count: usize,
}

impl<T: Clone> ColumnNode<T> {
    pub(super) fn into_values(node: Arc<Self>, output: &mut Vec<T>) {
        match Arc::unwrap_or_clone(node).storage {
            ColumnStorage::Page(values) => {
                output.extend(values.into_iter().flatten().map(Arc::unwrap_or_clone))
            }
            ColumnStorage::Branch(children) => {
                for child in children.into_iter().flatten() {
                    Self::into_values(child, output);
                }
            }
        }
    }

    pub(super) fn empty(height: usize) -> Self {
        if height == 0 {
            Self {
                storage: ColumnStorage::Page(vec![None; PAGE_LEN]),
                node_count: 1,
                page_count: 1,
                value_count: 0,
            }
        } else {
            Self::branch(None, None)
        }
    }

    pub(super) fn branch(left: Option<Arc<Self>>, right: Option<Arc<Self>>) -> Self {
        let children = [left, right];
        let node_count = 1 + children
            .iter()
            .flatten()
            .map(|node| node.node_count)
            .sum::<usize>();
        let page_count = children.iter().flatten().map(|node| node.page_count).sum();
        let value_count = children.iter().flatten().map(|node| node.value_count).sum();
        Self {
            storage: ColumnStorage::Branch(children),
            node_count,
            page_count,
            value_count,
        }
    }

    pub(super) fn get(&self, index: usize, height: usize) -> Option<&Arc<T>> {
        match &self.storage {
            ColumnStorage::Page(values) => values[index % PAGE_LEN].as_ref(),
            ColumnStorage::Branch(children) => children[branch_index(index, height)]
                .as_ref()?
                .get(index, height - 1),
        }
    }

    pub(super) fn get_mut(&mut self, index: usize, height: usize) -> &mut T {
        match &mut self.storage {
            ColumnStorage::Page(values) => {
                Arc::make_mut(values[index % PAGE_LEN].as_mut().unwrap())
            }
            ColumnStorage::Branch(children) => {
                Arc::make_mut(children[branch_index(index, height)].as_mut().unwrap())
                    .get_mut(index, height - 1)
            }
        }
    }

    pub(super) fn detach<'a>(this: &'a mut Arc<Self>, copied_bytes: &mut u64) -> &'a mut Self {
        if !this.is_unique() {
            let page_bytes = match &this.storage {
                ColumnStorage::Page(values) => values.len() * std::mem::size_of::<Option<Arc<T>>>(),
                ColumnStorage::Branch(_) => 0,
            };
            *copied_bytes =
                copied_bytes.saturating_add((std::mem::size_of::<Self>() + page_bytes) as u64);
        }
        Arc::make_mut(this)
    }

    pub(super) fn set(
        &mut self,
        index: usize,
        height: usize,
        value: Arc<T>,
        copied_bytes: &mut u64,
    ) {
        match &mut self.storage {
            ColumnStorage::Page(values) => {
                if values[index % PAGE_LEN].is_none() {
                    self.value_count += 1;
                }
                values[index % PAGE_LEN] = Some(value);
            }
            ColumnStorage::Branch(children) => {
                let child = children[branch_index(index, height)]
                    .get_or_insert_with(|| Arc::new(Self::empty(height - 1)));
                Self::detach(child, copied_bytes).set(index, height - 1, value, copied_bytes);
                self.node_count = 1 + children
                    .iter()
                    .flatten()
                    .map(|node| node.node_count)
                    .sum::<usize>();
                self.page_count = children.iter().flatten().map(|node| node.page_count).sum();
                self.value_count = children.iter().flatten().map(|node| node.value_count).sum();
            }
        }
    }
}

fn branch_index(index: usize, height: usize) -> usize {
    (index / PAGE_LEN >> (height - 1)) & 1
}
