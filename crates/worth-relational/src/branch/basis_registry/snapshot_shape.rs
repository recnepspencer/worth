//! Exact initialized slots and the separately paid physical hash traversal.
#[derive(Clone, Copy)]
pub(super) struct RetainedBasisSnapshotShape {
    pub(super) elements: usize,
    pub(super) buckets: usize,
}
impl RetainedBasisSnapshotShape {
    pub(super) fn covers(self, actual: Self) -> bool {
        self.elements == actual.elements && actual.buckets <= self.buckets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_snapshot_rejects_cardinality_changes_and_unpaid_bucket_growth() {
        let paid = RetainedBasisSnapshotShape {
            elements: 3,
            buckets: 14,
        };
        for actual in [
            RetainedBasisSnapshotShape {
                elements: 2,
                buckets: 14,
            },
            RetainedBasisSnapshotShape {
                elements: 4,
                buckets: 14,
            },
            RetainedBasisSnapshotShape {
                elements: 3,
                buckets: 28,
            },
        ] {
            assert!(!paid.covers(actual));
        }
        assert!(paid.covers(paid));
        assert!(paid.covers(RetainedBasisSnapshotShape {
            elements: 3,
            buckets: 7
        }));
    }
}
