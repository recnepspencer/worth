use super::{WorthQueryGraphReadMaterial, WorthQueryGraphReadRow};

struct WorthQueryGraphReadMaterialNode {
    material: WorthQueryGraphReadMaterial,
    next: Option<Box<WorthQueryGraphReadMaterialNode>>,
}

#[derive(Default)]
pub(super) struct WorthQueryGraphReadMaterialization {
    head: Option<Box<WorthQueryGraphReadMaterialNode>>,
    chunk_count: u64,
    row_count: usize,
    retained_bytes: usize,
}

impl WorthQueryGraphReadMaterialization {
    pub(super) const fn node_allocation_bytes() -> usize {
        std::mem::size_of::<WorthQueryGraphReadMaterialNode>()
    }

    pub(super) fn push(&mut self, material: WorthQueryGraphReadMaterial) -> usize {
        let retained_bytes = material
            .owned_allocation_capacity_bytes()
            .saturating_add(Self::node_allocation_bytes());
        self.row_count = self.row_count.saturating_add(material.rows().len());
        self.chunk_count = self.chunk_count.saturating_add(1);
        self.retained_bytes = self.retained_bytes.saturating_add(retained_bytes);
        self.head = Some(Box::new(WorthQueryGraphReadMaterialNode {
            material,
            next: self.head.take(),
        }));
        retained_bytes
    }

    pub(super) fn restore_emission_order(mut self) -> Self {
        let mut ordered = None;
        while let Some(mut node) = self.head.take() {
            self.head = node.next.take();
            node.next = ordered;
            ordered = Some(node);
        }
        self.head = ordered;
        self
    }

    pub(super) const fn chunk_count(&self) -> u64 {
        self.chunk_count
    }

    pub(super) const fn row_count(&self) -> usize {
        self.row_count
    }

    pub(super) const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(super) fn rows(&self) -> WorthQueryGraphReadRows<'_> {
        WorthQueryGraphReadRows {
            node: self.head.as_deref(),
            row_index: 0,
            remaining: self.row_count,
        }
    }
}

pub struct WorthQueryGraphReadRows<'a> {
    node: Option<&'a WorthQueryGraphReadMaterialNode>,
    row_index: usize,
    remaining: usize,
}

impl<'a> Iterator for WorthQueryGraphReadRows<'a> {
    type Item = &'a WorthQueryGraphReadRow;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = self.node?;
            if let Some(row) = node.material.rows().get(self.row_index) {
                self.row_index = self.row_index.saturating_add(1);
                self.remaining = self.remaining.saturating_sub(1);
                return Some(row);
            }
            self.node = node.next.as_deref();
            self.row_index = 0;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for WorthQueryGraphReadRows<'_> {}

impl std::fmt::Debug for WorthQueryGraphReadMaterialization {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryGraphReadMaterialization")
            .field("chunk_count", &self.chunk_count)
            .field("row_count", &self.row_count)
            .field("retained_bytes", &self.retained_bytes)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryGraphReadMaterialization {
    fn eq(&self, other: &Self) -> bool {
        if self.chunk_count != other.chunk_count
            || self.row_count != other.row_count
            || self.retained_bytes != other.retained_bytes
        {
            return false;
        }
        let mut left = self.head.as_deref();
        let mut right = other.head.as_deref();
        loop {
            match (left, right) {
                (Some(left_node), Some(right_node)) => {
                    if left_node.material != right_node.material {
                        return false;
                    }
                    left = left_node.next.as_deref();
                    right = right_node.next.as_deref();
                }
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}

impl Drop for WorthQueryGraphReadMaterialization {
    fn drop(&mut self) {
        while let Some(mut node) = self.head.take() {
            self.head = node.next.take();
        }
    }
}
