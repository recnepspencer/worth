//! Failure-site data stays with the original retention reason until commit reports it.
use crate::domain_computation::application_aftermath::WorthQueryPreImageRetentionDenial;
use worth_query_installation::facade::InstalledPreImageLocus;
use worth_relational::facade::transactions::RecordRef;

#[derive(Debug)]
pub(super) struct FieldRetentionFailure {
    pub(super) denial: WorthQueryPreImageRetentionDenial,
    pub(super) detail: Option<String>,
}

impl From<WorthQueryPreImageRetentionDenial> for FieldRetentionFailure {
    fn from(denial: WorthQueryPreImageRetentionDenial) -> Self {
        Self {
            denial,
            detail: None,
        }
    }
}

impl FieldRetentionFailure {
    pub(super) fn missing(
        site: &'static str,
        target: Option<&RecordRef>,
        locus: Option<&InstalledPreImageLocus>,
    ) -> Self {
        Self {
            denial: WorthQueryPreImageRetentionDenial::MissingDemandedField,
            detail: Some(format!(
                "recorded inverse demands an unobserved exact mutated field; site={site}; target={target:?}; locus={locus:?}"
            )),
        }
    }

    pub(super) fn missing_output(role: &str, entity: &str) -> Self {
        Self {
            denial: WorthQueryPreImageRetentionDenial::MissingDemandedField,
            detail: Some(format!(
                "recorded inverse demands an unobserved exact mutated field; site=output-role-target; role={role}; entity={entity}"
            )),
        }
    }
}
