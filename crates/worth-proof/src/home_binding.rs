use crate::AuthorityMarker;

/// Move-only admission of the declared binding of an application home.
///
/// The concrete owner payload has private construction after installed schema
/// and closed program-roster checks. A data description cannot replace it.
///
/// ```compile_fail,E0308
/// use worth_proof::{AdmittedApplicationHomeBinding, AuthorityMarker};
/// struct OwnerAdmission;
/// impl AuthorityMarker for OwnerAdmission {}
/// struct Description;
/// impl AuthorityMarker for Description {}
/// fn open(_: AdmittedApplicationHomeBinding<OwnerAdmission>) {}
/// open(AdmittedApplicationHomeBinding::from_owner(Description));
/// ```
#[derive(Debug)]
pub struct AdmittedApplicationHomeBinding<Admission: AuthorityMarker> {
    admission: Admission,
}

impl<Admission: AuthorityMarker> AdmittedApplicationHomeBinding<Admission> {
    /// Possession of the exact owner-sealed admission is the minting authority.
    pub fn from_owner(admission: Admission) -> Self {
        Self { admission }
    }

    pub fn consume<Output>(self, admit: impl FnOnce(Admission) -> Output) -> Output {
        admit(self.admission)
    }
}
