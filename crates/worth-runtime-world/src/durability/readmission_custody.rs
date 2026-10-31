//! Actual acquired Native admissions survive every later readmission refusal.
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::durability::{
    RecoveredRelationalBranchBasis, RelationalRecoveredPublicationCustody,
};

#[derive(Debug)]
pub struct WorldNativeReadmissionCustody {
    pub(crate) root: Option<RecoveredRelationalBranchBasis>,
    pub(crate) publications:
        Vec<Option<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>>,
}

impl WorldNativeReadmissionCustody {
    pub(crate) fn empty() -> Self {
        Self {
            root: None,
            publications: Vec::new(),
        }
    }
    pub(crate) fn refuse(
        self,
        cause: super::WorldReadmissionDenial,
    ) -> super::WorldReadmissionDenial {
        super::WorldReadmissionDenial::NativeCustody {
            cause: Box::new(cause),
            custody: Box::new(self),
        }
    }

    pub fn retains_root_admission(&self) -> bool {
        self.root.is_some()
    }
    pub fn retained_publication_admissions(&self) -> usize {
        self.publications
            .iter()
            .filter(|publication| publication.is_some())
            .count()
    }
}
