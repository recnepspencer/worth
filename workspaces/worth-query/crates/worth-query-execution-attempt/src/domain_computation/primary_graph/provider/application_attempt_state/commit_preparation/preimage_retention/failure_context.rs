//! Failure-site data stays with the original retention reason until commit reports it.
use crate::domain_computation::application_aftermath::WorthQueryPreImageRetentionDenial;
use worth_query_execution_lineage::facade::ports::{
    CommitAuthorityBindingInternalPort as _, RetainedGovernedInputInternalPort as _,
};
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
    pub(super) fn with_attempt(
        mut self,
        attempt: &super::WorthQueryPrimaryGraphApplicationAttempt,
        member: Option<&super::super::super::CoupledApplicationMemberKind>,
        route: &'static str,
        original_changed_fields: bool,
    ) -> Self {
        if self.denial != WorthQueryPreImageRetentionDenial::MissingDemandedField {
            return self;
        }
        use super::super::super::CoupledApplicationMemberKind as Member;
        let participant = match member {
            None => "root".to_owned(),
            Some(Member::AuthoredStage {
                identity, binding, ..
            }) => {
                format!("authored-stage {} {binding}", identity.ordinal())
            }
            Some(Member::Producer { producer, meaning }) => {
                format!("producer {producer} source={:?}", meaning.source_scope)
            }
        };
        let input = attempt
            .original_authority()
            .retained_governed_input_carrier();
        let binding = input.and_then(|input| input.material_input().map(|(_, binding, _)| binding));
        let correction = input.and_then(|input| input.correction());
        let authored = attempt.authored_correction_identity();
        let detail = self.detail.take().unwrap_or_else(|| {
            "recorded inverse demands an unobserved exact mutated field".to_owned()
        });
        self.detail = Some(format!(
            "{detail}; participant={participant}; binding={binding:?}; retained_input={}; correction_association={}; authored_identity={}; recorded_reverse={}; route={route}; original_changed_fields={original_changed_fields}",
            input.is_some(), correction.is_some(), authored.is_some(), attempt.recorded_reverse().is_some(),
        ));
        self
    }

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
