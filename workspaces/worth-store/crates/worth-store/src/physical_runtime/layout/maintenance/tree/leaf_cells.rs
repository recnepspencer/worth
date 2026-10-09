use worth_store_physical_format::BTreeNodeCellV1;

use super::PhysicalLayoutMaintenanceFailure;

/// Whether a leaf write changed the cell set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LeafCellWrite {
    Unchanged,
    Written,
}

/// Writes `key -> value` into sorted leaf cells. An existing equal value is a
/// no-op; an existing value is replaced only when it is exactly the
/// `superseded` value the caller read. Any other existing value conflicts.
pub(super) fn write_leaf_cell(
    cells: &mut Vec<BTreeNodeCellV1>,
    key: &[u8],
    value: &[u8],
    superseded: Option<&[u8]>,
) -> Result<LeafCellWrite, PhysicalLayoutMaintenanceFailure> {
    match cells.binary_search_by(|cell| cell.key().cmp(key)) {
        Ok(index) if cells[index].leaf_value() == Some(value) => Ok(LeafCellWrite::Unchanged),
        Ok(index) if cells[index].leaf_value() == superseded => {
            cells[index] = BTreeNodeCellV1::leaf(key.to_vec(), value.to_vec());
            Ok(LeafCellWrite::Written)
        }
        Ok(_) => Err(PhysicalLayoutMaintenanceFailure::ConflictingKey),
        Err(index) => {
            cells.insert(index, BTreeNodeCellV1::leaf(key.to_vec(), value.to_vec()));
            Ok(LeafCellWrite::Written)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells() -> Vec<BTreeNodeCellV1> {
        vec![
            BTreeNodeCellV1::leaf(vec![1], vec![10]),
            BTreeNodeCellV1::leaf(vec![3], vec![30]),
        ]
    }

    #[test]
    fn existing_value_other_than_superseded_conflicts_and_leaves_cells_unchanged() {
        let mut written = cells();
        assert!(matches!(
            write_leaf_cell(&mut written, &[3], &[31], Some(&[29])),
            Err(PhysicalLayoutMaintenanceFailure::ConflictingKey)
        ));
        assert!(matches!(
            write_leaf_cell(&mut written, &[3], &[31], None),
            Err(PhysicalLayoutMaintenanceFailure::ConflictingKey)
        ));
        assert_eq!(written, cells());
    }

    #[test]
    fn exact_superseded_value_is_replaced_and_equal_value_is_a_no_op() {
        let mut written = cells();
        assert_eq!(
            write_leaf_cell(&mut written, &[3], &[31], Some(&[30])).unwrap(),
            LeafCellWrite::Written
        );
        assert_eq!(written[1].leaf_value(), Some(&[31][..]));
        assert_eq!(
            write_leaf_cell(&mut written, &[3], &[31], Some(&[30])).unwrap(),
            LeafCellWrite::Unchanged
        );
        assert_eq!(
            write_leaf_cell(&mut written, &[2], &[20], None).unwrap(),
            LeafCellWrite::Written
        );
        assert_eq!(written.len(), 3);
        assert_eq!(written[1].key(), &[2]);
    }
}
