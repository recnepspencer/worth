use std::collections::BTreeMap;

pub(crate) fn duplicate_groups<Identity: Ord>(
    identities: impl IntoIterator<Item = (usize, Identity)>,
) -> Vec<Vec<usize>> {
    let mut by_identity: BTreeMap<Identity, Vec<usize>> = BTreeMap::new();
    for (index, identity) in identities {
        by_identity.entry(identity).or_default().push(index);
    }
    by_identity
        .into_values()
        .filter(|group| group.len() > 1)
        .collect()
}
