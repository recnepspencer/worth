use super::*;

#[test]
fn prefix_count_matches_an_independent_scan_after_mixed_edits_and_shape_changes() {
    for seed in 1_u64..=8 {
        let mut random = seed;
        let mut values = vec![None; 64];
        let mut previous: SharedColumn<StorageAllocation<Option<usize>>> =
            SharedColumn::from(values.clone());
        let mut presence = 0;
        for step in 0..256 {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let mut current = previous.clone();
            if step % 17 == 0 {
                values.push(Some(step));
                current.push(Some(step));
            } else if step % 31 == 0 {
                values.truncate(values.len() / 2);
                current = SharedColumn::from(values.clone());
            } else {
                let index = (random as usize) % values.len();
                let value = (random & 4 != 0).then_some(step);
                values[index] = value;
                current[index] = value;
            }
            presence = adjust(presence, &previous, &current);
            assert_eq!(
                presence,
                values.iter().filter(|value| value.is_some()).count()
            );
            previous = current;
        }
    }
}

// The count primitive above is additional equality evidence. Growth exercises
// PreparedResidentRegion::incremental itself, including its residency choice.
