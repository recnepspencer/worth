//! Pin the selected Query head; native route admission decides whether cold custody is required.
use super::*;
pub(crate) mod readmission;
pub(in crate::domain_computation::primary_graph) use readmission::{
    ReadmittedRequiredPublication, RequiredPublicationReadmissionStop,
};
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
use worth_relational::facade::mvcc::RelationalRequiredOutputPublication;

pub struct SelectedRequiredPublicationHead {
    selected: Arc<OnceLock<RecordedOutput>>,
}

pub struct RequiredPublicationPredecessor<'head> {
    pub(in crate::domain_computation::primary_graph) implementation:
        worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
    pub(in crate::domain_computation::primary_graph) correspondence:
        Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(in crate::domain_computation::primary_graph) producer: &'head str,
    pub(in crate::domain_computation::primary_graph) dependency: [u8; 32],
    pub(in crate::domain_computation::primary_graph) source_epoch: [u8; 32],
}

impl SelectedRequiredPublicationHead {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn retain(
        selected: &Arc<OnceLock<RecordedOutput>>,
    ) -> Result<Self, Stop> {
        let row = selected.get().ok_or(Stop::SelectedSourceMismatch)?;
        let origin = match &row.performed_origin {
            Some(alias) => {
                let origin = alias.get().ok_or(Stop::SelectedSourceMismatch)?;
                if origin.performed_origin.is_some() {
                    return Err(Stop::SelectedSourceMismatch);
                }
                origin
            }
            None => row,
        };
        // A factory-produced ordinary restoration may have no native locator.
        // A partial native ownership description cannot become that ordinary case.
        let claims_native = origin.required_publication.is_some();
        let has_locator = origin.native_prior_checkpoint.is_some();
        let native_restoration = origin.source_identity.is_none()
            && origin
                .required_publication
                .as_ref()
                .is_some_and(|publication| publication.is_authored_restoration());
        if (claims_native && !has_locator && !native_restoration)
            || (has_locator && origin.producer_dependency_identity.is_none())
        {
            return Err(Stop::SelectedSourceMismatch);
        }
        Ok(Self {
            selected: Arc::clone(selected),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn warm_publication(
        &self,
    ) -> Option<RelationalRequiredOutputPublication> {
        self.selected
            .get()?
            .required_publication
            .clone()
            .or_else(|| self.origin().required_publication.clone())
    }

    pub(in crate::domain_computation::primary_graph) fn cold_predecessor(
        &self,
    ) -> Result<RequiredPublicationPredecessor<'_>, Stop> {
        let origin = self.origin();
        let locator = origin
            .native_prior_checkpoint
            .as_ref()
            .ok_or(Stop::SelectedSourceMismatch)?;
        let dependency = origin
            .producer_dependency_identity
            .ok_or(Stop::SelectedSourceMismatch)?;

        Ok(RequiredPublicationPredecessor {
            implementation: origin
                .producing_implementation
                .ok_or(Stop::SelectedSourceMismatch)?,
            correspondence: Arc::clone(&origin.correspondence),
            producer: locator.producer(),
            dependency,
            source_epoch: locator.source(),
        })
    }

    fn origin(&self) -> &RecordedOutput {
        let row = self
            .selected
            .get()
            .expect("selection pins a filled lineage head");
        match &row.performed_origin {
            Some(alias) => alias
                .get()
                .expect("selection resolved the performed origin"),
            None => row,
        }
    }
}

mod court_doors_for_test;
