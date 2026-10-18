//! Independent child cells run concurrently; their comparisons keep declared order.
use super::{cell, Cell};

pub(super) type CellResult = Result<(String, (u64, u64)), String>;

pub(super) fn parallel(rows: &[Cell]) -> Vec<CellResult> {
    if rows.is_empty() {
        return Vec::new();
    }
    // Each child provisions its declared native worker width. The gate gives
    // this complete matrix its own lane, so that width bounds its CPU demand.
    let largest_width = rows.iter().map(|row| row.workers).max().unwrap().max(1);
    let processors = std::thread::available_parallelism().map_or(1, usize::from);
    let workers = (processors / largest_width).max(1).min(rows.len());
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|lane| {
                scope.spawn(move || {
                    rows.iter()
                        .copied()
                        .enumerate()
                        .skip(lane)
                        .step_by(workers)
                        .map(|(index, row)| (index, cell(row)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let mut ordered: Vec<_> = handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("matrix child launcher completes"))
            .collect();
        ordered.sort_unstable_by_key(|(index, _)| *index);
        assert_eq!(ordered.len(), rows.len(), "every runnable row completes");
        ordered
            .into_iter()
            .enumerate()
            .map(|(expected, (index, observation))| {
                assert_eq!(index, expected, "each declared row appears exactly once");
                observation
            })
            .collect()
    })
}
