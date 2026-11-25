use worth_foundational::facade::AspectKey;

pub fn ordered_aspect_keys(aspects: impl IntoIterator<Item = AspectKey>) -> Vec<AspectKey> {
    let mut aspects = aspects.into_iter().collect::<Vec<_>>();
    if !aspects.windows(2).all(|window| window[0] < window[1]) {
        aspects.sort();
    }
    aspects.dedup();
    aspects
}
