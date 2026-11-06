//! Derived retirement permission stays on the authentic producer succession issuer.
use super::*;

#[derive(Debug)]
pub struct AdmittedProducerOutputRetirement {
    generator: RequiredOutputPublicationAuthority,
    publications: Vec<RequiredOutputPublicationAuthority>,
}
impl InstalledApplicationPublicationAuthority {
    pub fn admit_source_scope_retirements(
        &self,
        succession: &AdmittedRequiredOutputSuccession,
        publications: Vec<RequiredOutputPublicationAuthority>,
    ) -> Option<AdmittedProducerOutputRetirement> {
        if !self.admits_required_output_succession(succession, &succession.predecessor)
            || publications.is_empty()
            || publications.iter().enumerate().any(|(index, publication)| {
                !Arc::ptr_eq(&self.occurrence, &publication.occurrence)
                    || same_position(publication, &succession.predecessor)
                    || publications[..index]
                        .iter()
                        .any(|prior| same_position(prior, publication))
            })
        {
            return None;
        }
        Some(AdmittedProducerOutputRetirement {
            generator: succession.predecessor.clone(),
            publications,
        })
    }
    pub fn admits_producer_output_retirement(
        &self,
        proof: &AdmittedProducerOutputRetirement,
        generator: &RequiredOutputPublicationAuthority,
        publication: &RequiredOutputPublicationAuthority,
    ) -> bool {
        Arc::ptr_eq(&self.occurrence, &generator.occurrence)
            && same_position(&proof.generator, generator)
            && proof.publications.iter().any(|prior| {
                Arc::ptr_eq(&self.occurrence, &prior.occurrence)
                    && same_position(prior, publication)
            })
    }
}
fn same_position(
    left: &RequiredOutputPublicationAuthority,
    right: &RequiredOutputPublicationAuthority,
) -> bool {
    Arc::ptr_eq(&left.occurrence, &right.occurrence)
        && left.publication == right.publication
        && left.ordinal == right.ordinal
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn producer_retirement_projects_only_exact_distinct_owned_publications() {
        let owner = InstalledApplicationPublicationOwner::fresh();
        let authority = owner.authority();
        let generator = authority.seal_required_output_publication_at(7, 0);
        let child = authority.seal_required_output_publication_at(8, 0);
        let leaf = authority.seal_required_output_publication_at(8, 1);
        let stranger = InstalledApplicationPublicationOwner::fresh()
            .authority()
            .seal_required_output_publication_at(8, 0);
        let succession = owner
            .admit_required_output_succession(
                owner.admit_required_output("producer", [1; 32]),
                &generator,
            )
            .unwrap();
        assert!(authority
            .admit_source_scope_retirements(&succession, vec![child.clone(), stranger.clone()])
            .is_none());
        assert!(authority
            .admit_source_scope_retirements(&succession, vec![child.clone(), child.clone()])
            .is_none());
        assert!(authority
            .admit_source_scope_retirements(&succession, vec![generator.clone()])
            .is_none());
        let proof = authority
            .admit_source_scope_retirements(&succession, vec![child.clone(), leaf.clone()])
            .unwrap();
        assert!(authority.admits_producer_output_retirement(&proof, &generator, &leaf));
        assert!(!authority.admits_producer_output_retirement(&proof, &child, &leaf));
        assert!(!authority.admits_producer_output_retirement(&proof, &generator, &stranger));
    }
}
