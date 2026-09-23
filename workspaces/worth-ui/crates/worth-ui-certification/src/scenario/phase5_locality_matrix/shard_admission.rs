use super::shard_contract::{is_large, SHARD_COUNT};

pub(super) const MAX_TOTAL: usize = 8;
pub(super) const MAX_LARGE: usize = 8;

pub(super) fn next_wave(pending: &[usize]) -> Vec<usize> {
    let mut wave = pending
        .iter()
        .copied()
        .filter(|shard| is_large(*shard))
        .take(MAX_LARGE)
        .collect::<Vec<_>>();
    wave.extend(
        pending
            .iter()
            .copied()
            .filter(|shard| !is_large(*shard))
            .take(MAX_TOTAL - wave.len()),
    );
    if wave.is_empty() {
        pending.iter().copied().take(MAX_TOTAL).collect()
    } else {
        wave
    }
}

pub(super) fn validate_wave(wave: &[usize]) -> Result<(), String> {
    if wave.is_empty() || wave.len() > MAX_TOTAL {
        return Err(format!("invalid shard admission size {}", wave.len()));
    }
    if wave.iter().filter(|shard| is_large(**shard)).count() > MAX_LARGE {
        return Err("shard admission exceeded large-world cap".to_owned());
    }
    if wave.iter().any(|shard| *shard >= SHARD_COUNT) {
        return Err("shard admission contained an unknown shard".to_owned());
    }
    if wave
        .iter()
        .enumerate()
        .any(|(index, shard)| wave[..index].contains(shard))
    {
        return Err("shard admission contains a duplicate".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_large, next_wave, validate_wave, MAX_LARGE, MAX_TOTAL, SHARD_COUNT};

    #[test]
    fn deterministic_admission_caps_native_load() {
        let pending = (0..SHARD_COUNT).collect::<Vec<_>>();
        let wave = next_wave(&pending);
        validate_wave(&wave).unwrap();
        assert_eq!(wave.len(), MAX_TOTAL.min(SHARD_COUNT));
        let large = wave.iter().filter(|shard| is_large(**shard)).count();
        assert!(large <= MAX_LARGE);
        assert_eq!(large, (SHARD_COUNT - SHARD_COUNT / 2).min(MAX_LARGE));
        assert!(wave[..large].iter().all(|shard| is_large(*shard)));
        assert!(wave[large..].iter().all(|shard| !is_large(*shard)));
    }

    #[test]
    fn deterministic_admission_completes_large_first_in_two_waves() {
        let mut pending = (0..SHARD_COUNT).collect::<Vec<_>>();
        let first = next_wave(&pending);
        validate_wave(&first).unwrap();
        assert!((SHARD_COUNT / 2..SHARD_COUNT).all(|shard| first.contains(&shard)));
        pending.retain(|shard| !first.contains(shard));
        let second = next_wave(&pending);
        validate_wave(&second).unwrap();
        assert!(second.iter().all(|shard| !is_large(*shard)));
        pending.retain(|shard| !second.contains(shard));
        assert!(pending.is_empty());
    }
}
