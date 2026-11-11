//! Selecting the installed producer's own publication is distinct from running it.
use super::*;

#[derive(Debug)]
pub struct AdmittedHistoricalOutputSelection {
    selected: Vec<RequiredOutputPublicationAuthority>,
}

impl InstalledApplicationPublicationOwner {
    pub fn admit_historical_output_selection<'a>(
        &self,
        selected: impl IntoIterator<Item = &'a RequiredOutputPublicationAuthority>,
    ) -> Option<AdmittedHistoricalOutputSelection> {
        let selected: Vec<_> = selected.into_iter().cloned().collect();
        if selected.is_empty()
            || selected.iter().enumerate().any(|(index, publication)| {
                !Arc::ptr_eq(&self.occurrence, &publication.occurrence)
                    || selected[..index].iter().any(|prior| {
                        prior.publication == publication.publication
                            && prior.ordinal == publication.ordinal
                    })
            })
        {
            return None;
        }
        Some(AdmittedHistoricalOutputSelection { selected })
    }
}

impl AdmittedHistoricalOutputSelection {
    pub fn publication_count(&self) -> usize {
        self.selected.len()
    }
}

impl InstalledApplicationPublicationAuthority {
    pub fn admits_historical_output_selection(
        &self,
        admission: &AdmittedHistoricalOutputSelection,
        selected: &RequiredOutputPublicationAuthority,
    ) -> bool {
        Arc::ptr_eq(&self.occurrence, &selected.occurrence)
            && admission.selected.iter().any(|prior| {
                Arc::ptr_eq(&self.occurrence, &prior.occurrence)
                    && prior.publication == selected.publication
                    && prior.ordinal == selected.ordinal
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_selection_requires_the_original_installed_owner_and_exact_publication_ordinal() {
        let owner = InstalledApplicationPublicationOwner::fresh();
        let other = InstalledApplicationPublicationOwner::fresh();
        let first = owner.authority().seal_required_output_publication_at(17, 0);
        let second = owner.authority().seal_required_output_publication_at(17, 1);
        let admission = owner
            .admit_historical_output_selection([&first, &second])
            .unwrap();
        assert_eq!(admission.publication_count(), 2);
        assert!(owner
            .authority()
            .admits_historical_output_selection(&admission, &first));
        assert!(owner
            .authority()
            .admits_historical_output_selection(&admission, &second));
        assert!(!other
            .authority()
            .admits_historical_output_selection(&admission, &first));
        assert!(!owner.authority().admits_historical_output_selection(
            &admission,
            &owner.authority().seal_required_output_publication_at(17, 2)
        ));
        assert!(other.admit_historical_output_selection([&first]).is_none());
        assert!(owner
            .admit_historical_output_selection([&first, &first])
            .is_none());
        assert!(owner.admit_historical_output_selection([]).is_none());
    }
}
