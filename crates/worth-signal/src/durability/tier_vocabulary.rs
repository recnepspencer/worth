//! Stable program names for generic runtime tier keys.
#[derive(Clone, Debug)]
pub struct SignalDurableTierVocabulary<T: Copy + Ord> {
    entries: Box<[(T, &'static str)]>,
}
#[derive(Debug)]
pub enum SignalDurableTierVocabularyDenial {
    EmptyName,
    DuplicateName(&'static str),
    DuplicateTier,
    MissingTierMapping,
    UnknownTierName(String),
}
impl<T: Copy + Ord> SignalDurableTierVocabulary<T> {
    pub fn declare(
        entries: impl IntoIterator<Item = (T, &'static str)>,
    ) -> Result<Self, SignalDurableTierVocabularyDenial> {
        use SignalDurableTierVocabularyDenial as D;
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_by_key(|(tier, _)| *tier);
        let mut names = std::collections::BTreeSet::new();
        for (index, (tier, name)) in entries.iter().enumerate() {
            if name.is_empty() {
                return Err(D::EmptyName);
            }
            if index > 0 && entries[index - 1].0 == *tier {
                return Err(D::DuplicateTier);
            }
            if !names.insert(*name) {
                return Err(D::DuplicateName(*name));
            }
        }
        Ok(Self {
            entries: entries.into_boxed_slice(),
        })
    }
    pub(crate) fn name(&self, tier: T) -> Result<&'static str, SignalDurableTierVocabularyDenial> {
        self.entries
            .binary_search_by_key(&tier, |(tier, _)| *tier)
            .map(|index| self.entries[index].1)
            .map_err(|_| SignalDurableTierVocabularyDenial::MissingTierMapping)
    }
    pub(crate) fn resolve(&self, name: &str) -> Result<T, SignalDurableTierVocabularyDenial> {
        self.entries
            .iter()
            .find(|(_, known)| *known == name)
            .map(|(tier, _)| *tier)
            .ok_or_else(|| SignalDurableTierVocabularyDenial::UnknownTierName(name.into()))
    }
}
