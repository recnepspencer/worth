use super::WorthQueryInstalledPackageIndex;

/// Installation-owned relationship between two installed package indexes.
///
/// The comparison uses the private installation runtime identity and
/// generation. Reporting digests alone cannot establish any relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledPackageIndexRelation {
    EquivalentGeneration,
    SameGenerationMeaningChanged,
    ExactSuccessor,
    ForeignRuntime,
    NonSuccessorGeneration,
}

impl WorthQueryInstalledPackageIndex {
    pub fn relation_to(&self, candidate: &Self) -> WorthQueryInstalledPackageIndexRelation {
        use WorthQueryInstalledPackageIndexRelation as Relation;

        if self.runtime != candidate.runtime
            || self.authority_lineage != candidate.authority_lineage
        {
            return Relation::ForeignRuntime;
        }
        if self.generation == candidate.generation {
            return if self.identity == candidate.identity {
                Relation::EquivalentGeneration
            } else {
                Relation::SameGenerationMeaningChanged
            };
        }
        let expected_successor = self.generation.ordinal().checked_add(1);
        if expected_successor == Some(candidate.generation.ordinal()) {
            Relation::ExactSuccessor
        } else {
            Relation::NonSuccessorGeneration
        }
    }
}
