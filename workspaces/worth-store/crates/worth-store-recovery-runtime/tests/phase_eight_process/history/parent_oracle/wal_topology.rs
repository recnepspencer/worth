use std::cmp::Ordering;

use super::WalFacts;

pub(super) fn validate(wal_files: &[WalFacts]) -> Result<(), String> {
    let mut chain = wal_files
        .iter()
        .filter(|facts| facts.frames > 0)
        .collect::<Vec<_>>();
    chain.sort_by_key(|facts| (facts.segment, facts.generation));
    let Some(first) = chain.first() else {
        return Ok(());
    };
    let generation = first
        .generation
        .ok_or_else(|| "parent oracle WAL frame set omitted its generation".to_owned())?;
    let mut previous = *first;
    let mut previous_segment = first
        .segment
        .ok_or_else(|| "parent oracle WAL frame set omitted its segment".to_owned())?;
    for current in chain.into_iter().skip(1) {
        let segment = current
            .segment
            .ok_or_else(|| "parent oracle WAL frame set omitted its segment".to_owned())?;
        let current_generation = current
            .generation
            .ok_or_else(|| "parent oracle WAL frame set omitted its generation".to_owned())?;
        if current_generation != generation {
            return Err(format!(
                "parent oracle WAL generation changed from {generation} to {current_generation}"
            ));
        }
        if segment != previous_segment.saturating_add(1) {
            return Err(format!(
                "parent oracle WAL segment chain has a gap or duplicate at {segment} after {previous_segment}"
            ));
        }
        let previous_last = previous
            .last
            .ok_or_else(|| "parent oracle WAL predecessor omitted its last LSN".to_owned())?;
        let current_first = current
            .first
            .ok_or_else(|| "parent oracle WAL successor omitted its first LSN".to_owned())?;
        match current_first.cmp(&previous_last) {
            Ordering::Equal => {}
            Ordering::Less => {
                return Err(format!(
                    "parent oracle WAL segment overlap at segment {segment}: first LSN {current_first} precedes {previous_last}"
                ));
            }
            Ordering::Greater => {
                return Err(format!(
                    "parent oracle WAL segment gap at segment {segment}: first LSN {current_first} follows {previous_last}"
                ));
            }
        }
        previous_segment = segment;
        previous = current;
    }
    Ok(())
}
