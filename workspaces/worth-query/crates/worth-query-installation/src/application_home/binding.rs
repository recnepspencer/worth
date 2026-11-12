use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId};
use worth_proof::{AdmittedApplicationHomeBinding, AuthorityMarker};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use crate::facade::{WorthQueryInstalledApplicationSchema, WorthQueryProgramSupportRoster};

/// Owner-sealed result of checking descriptive binding against installed
/// schema and a closed support roster. Digests alone cannot construct it.
///
/// ```compile_fail,E0451
/// use worth_foundational::facade::ApplicationHomeBinding;
/// use worth_query_installation::facade::WorthQueryHomeBindingAdmission;
/// fn forge(description: ApplicationHomeBinding) -> WorthQueryHomeBindingAdmission {
///     WorthQueryHomeBindingAdmission { description }
/// }
/// ```
#[derive(Debug)]
pub struct WorthQueryHomeBindingAdmission {
    description: ApplicationHomeBinding,
}

impl AuthorityMarker for WorthQueryHomeBindingAdmission {}

impl WorthQueryHomeBindingAdmission {
    pub fn into_description(self) -> ApplicationHomeBinding {
        self.description
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHomeBindingDenial {
    ForeignInstalledRoster,
    SelectedProgramNotRostered,
    DescriptionMismatch,
}

impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationSchema<Schema> {
    /// Admit caller-owned descriptive storage without allocating another copy.
    /// The caller's preparation owns construction/accounting of the data;
    /// this port checks it against authentic installed and validated meaning.
    pub fn admit_home_binding(
        &self,
        roster: &WorthQueryProgramSupportRoster<Schema>,
        selected: &ApplicationProgramRevision,
        description: ApplicationHomeBinding,
    ) -> Result<
        AdmittedApplicationHomeBinding<WorthQueryHomeBindingAdmission>,
        WorthQueryHomeBindingDenial,
    > {
        use WorthQueryHomeBindingDenial as Denial;
        let installed = self.binding_identity();
        if roster.schema_binding() != &installed {
            return Err(Denial::ForeignInstalledRoster);
        }
        let selected = roster
            .entry(selected)
            .ok_or(Denial::SelectedProgramNotRostered)?;
        if description.package() != *installed.package_identity()
            || description.schema() != *installed.schema_identity()
            || description.selected().identity() != selected.identity().as_str()
            || description.selected().revision()
                != CanonicalDigestId::new(*selected.revision().as_bytes())
            || description.roster().len() != roster.entries().len()
            || roster.entries().iter().any(|entry| {
                description
                    .program(CanonicalDigestId::new(*entry.revision().as_bytes()))
                    .is_none_or(|recorded| recorded.identity() != entry.identity().as_str())
            })
        {
            return Err(Denial::DescriptionMismatch);
        }
        Ok(AdmittedApplicationHomeBinding::from_owner(
            WorthQueryHomeBindingAdmission { description },
        ))
    }
}
