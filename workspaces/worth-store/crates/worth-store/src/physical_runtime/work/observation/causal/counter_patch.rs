use super::super::PhysicalWorkCounterSnapshot;

const FAMILY_COUNT: usize = 9;
const PRESSURE_COUNT: usize = 8;
const STAGE_COUNT: usize = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PhysicalWorkCounterPatch {
    changes: Box<[PhysicalWorkCounterChange]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhysicalWorkCounterChange {
    cell: u16,
    value: u64,
}

impl PhysicalWorkCounterPatch {
    pub(super) fn between(
        before: PhysicalWorkCounterSnapshot,
        after: PhysicalWorkCounterSnapshot,
    ) -> Self {
        let mut changes = Vec::new();
        for family in 0..FAMILY_COUNT {
            for pressure in 0..PRESSURE_COUNT {
                for stage in 0..STAGE_COUNT {
                    let value = after.by_family_and_pressure[family][pressure][stage];
                    if value != before.by_family_and_pressure[family][pressure][stage] {
                        changes.push(PhysicalWorkCounterChange {
                            cell: encode_cell(family, pressure, stage),
                            value,
                        });
                    }
                }
            }
        }
        Self {
            changes: changes.into_boxed_slice(),
        }
    }

    pub(super) fn apply_to(&self, snapshot: &mut PhysicalWorkCounterSnapshot) {
        for change in &self.changes {
            let (family, pressure, stage) = decode_cell(change.cell);
            snapshot.by_family_and_pressure[family][pressure][stage] = change.value;
        }
    }
}

const fn encode_cell(family: usize, pressure: usize, stage: usize) -> u16 {
    ((family * PRESSURE_COUNT * STAGE_COUNT) + (pressure * STAGE_COUNT) + stage) as u16
}

const fn decode_cell(cell: u16) -> (usize, usize, usize) {
    let cell = cell as usize;
    let family = cell / (PRESSURE_COUNT * STAGE_COUNT);
    let within_family = cell % (PRESSURE_COUNT * STAGE_COUNT);
    let pressure = within_family / STAGE_COUNT;
    let stage = within_family % STAGE_COUNT;
    (family, pressure, stage)
}

#[cfg(test)]
mod tests {
    use super::{PhysicalWorkCounterPatch, PhysicalWorkCounterSnapshot};

    #[test]
    fn sparse_absolute_patch_reconstructs_the_exact_observed_snapshot() {
        let mut before_counts = [[[0_u64; 7]; 8]; 9];
        before_counts[1][2][3] = 8;
        before_counts[8][6][6] = 13;
        let before = PhysicalWorkCounterSnapshot::from_counts(before_counts);

        let mut after_counts = before_counts;
        after_counts[1][2][3] = 5;
        after_counts[4][5][6] = 21;
        after_counts[1][7][6] = 34;
        let after = PhysicalWorkCounterSnapshot::from_counts(after_counts);

        let patch = PhysicalWorkCounterPatch::between(before, after);
        let mut reconstructed = before;
        patch.apply_to(&mut reconstructed);

        assert_eq!(reconstructed, after);
        assert_eq!(patch.changes.len(), 3);
    }

    #[test]
    fn evicted_patch_advances_the_base_for_retained_history() {
        let empty = PhysicalWorkCounterSnapshot::default();
        let first = snapshot_with_values(2, 0, 0);
        let second = snapshot_with_values(2, 3, 0);
        let third = snapshot_with_values(2, 3, 5);
        let first_patch = PhysicalWorkCounterPatch::between(empty, first);
        let second_patch = PhysicalWorkCounterPatch::between(first, second);
        let third_patch = PhysicalWorkCounterPatch::between(second, third);

        let mut rolling_base = empty;
        first_patch.apply_to(&mut rolling_base);
        assert_eq!(rolling_base, first);

        let mut retained = rolling_base;
        second_patch.apply_to(&mut retained);
        assert_eq!(retained, second);
        third_patch.apply_to(&mut retained);
        assert_eq!(retained, third);
    }

    fn snapshot_with_values(first: u64, second: u64, third: u64) -> PhysicalWorkCounterSnapshot {
        let mut counts = [[[0_u64; 7]; 8]; 9];
        counts[0][0][0] = first;
        counts[4][5][6] = second;
        counts[8][6][6] = third;
        PhysicalWorkCounterSnapshot::from_counts(counts)
    }
}
