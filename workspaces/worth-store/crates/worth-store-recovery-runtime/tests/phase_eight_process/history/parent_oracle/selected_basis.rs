use super::observe_artifact_at_path;

#[path = "selected_basis/wal_tail_suffix.rs"]
mod wal_tail_suffix;

const CHECKPOINT_PATH: &str = "families/checkpoint.current";
const WAL_PREFIX: &str = "families/wal/";

/// Return only the independently verified checkpoint and WAL tail that the
/// production recovery selector can consume. Other files remain observer
/// evidence, never fate evidence.
pub(crate) fn select(files: &[(String, Vec<u8>)]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let checkpoint = files
        .iter()
        .find(|(path, _)| path == CHECKPOINT_PATH)
        .ok_or_else(|| "selected-basis oracle cannot find current checkpoint".to_owned())?;
    let checkpoint_facts = observe_artifact_at_path(&checkpoint.0, &checkpoint.1)
        .checkpoint
        .ok_or_else(|| "selected-basis oracle rejected current checkpoint".to_owned())?;
    let frontier = checkpoint_facts.covered.1;
    let wal = select_wal(files, frontier)?;
    let mut selected = vec![(checkpoint.0.clone(), checkpoint.1.clone())];
    selected.extend(wal);
    Ok(selected)
}

fn select_wal(
    files: &[(String, Vec<u8>)],
    frontier: u64,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut candidates = files
        .iter()
        .filter_map(|(path, bytes)| {
            let name = path.strip_prefix(WAL_PREFIX)?.to_owned();
            let identity = parse_segment_name(&name)?;
            let facts = observe_artifact_at_path(path, bytes).wal?;
            (facts.frames > 0 && facts.last.is_some_and(|last| last > frontier))
                .then_some((identity, path, bytes, facts))
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable_by_key(|(identity, ..)| *identity);
    let Some((_, _, _, first_facts)) = candidates.first() else {
        return Ok(Vec::new());
    };
    let generation = first_facts
        .generation
        .ok_or_else(|| "selected-basis WAL omitted generation".to_owned())?;
    let mut selected = Vec::new();
    let mut previous_end = frontier;
    let mut previous_segment: Option<u64> = None;
    for (identity, path, bytes, facts) in candidates {
        if identity.1 != generation {
            return Err(
                "selected-basis WAL generation does not match the selected tail".to_owned(),
            );
        }
        let Some(wal) = wal_tail_suffix::select(bytes, facts, frontier)? else {
            continue;
        };
        let first = wal.first_lsn;
        let last = wal.last_lsn;
        validate_wal_continuation(
            identity,
            generation,
            frontier,
            first,
            last,
            previous_end,
            previous_segment,
        )?;
        selected.push((path.to_owned(), wal.bytes));
        previous_end = last;
        previous_segment = Some(identity.0);
    }
    Ok(selected)
}

fn validate_wal_continuation(
    identity: (u64, u64),
    generation: u64,
    frontier: u64,
    first: u64,
    last: u64,
    previous_end: u64,
    previous_segment: Option<u64>,
) -> Result<(), String> {
    if identity.1 != generation {
        return Err("selected-basis WAL generation does not match the selected tail".to_owned());
    }
    if first < frontier || first < previous_end {
        return Err("selected-basis WAL tail overlaps the checkpoint or prior segment".to_owned());
    }
    if first > previous_end
        || previous_segment.is_some_and(|segment| identity.0 != segment.saturating_add(1))
    {
        return Err("selected-basis WAL tail is not contiguous".to_owned());
    }
    if last <= first {
        return Err("selected-basis WAL tail has an invalid LSN range".to_owned());
    }
    Ok(())
}

fn parse_segment_name(name: &str) -> Option<(u64, u64)> {
    let body = name.strip_prefix("segment-")?.strip_suffix(".wal")?;
    let (segment, generation) = body.split_once("-generation-")?;
    let segment = segment.parse::<u64>().ok()?;
    let generation = generation.parse::<u64>().ok()?;
    (segment > 0
        && generation > 0
        && format!("segment-{segment}-generation-{generation}.wal") == name)
        .then_some((segment, generation))
}
