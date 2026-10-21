//! Pin the selected Query head; native route admission decides whether cold custody is required.
use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
use worth_relational::facade::mvcc::RelationalRequiredOutputPublication;

#[cfg(test)]
mod tests;

pub(in crate::domain_computation::primary_graph) struct SelectedRequiredPublicationHead {
    selected: Arc<OnceLock<RecordedOutput>>,
}

pub(in crate::domain_computation::primary_graph) struct RequiredPublicationPredecessor {
    pub(in crate::domain_computation::primary_graph) correspondence:
        Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(in crate::domain_computation::primary_graph) producer: String,
    pub(in crate::domain_computation::primary_graph) dependency: [u8; 32],
    pub(in crate::domain_computation::primary_graph) source_epoch: [u8; 32],
}

impl SelectedRequiredPublicationHead {
    pub(super) fn retain(selected: &Arc<OnceLock<RecordedOutput>>) -> Result<Self, Stop> {
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
        if (claims_native && !has_locator)
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
        self.origin().required_publication.clone()
    }

    pub(in crate::domain_computation::primary_graph) fn cold_predecessor(
        &self,
    ) -> Result<RequiredPublicationPredecessor, Stop> {
        let origin = self.origin();
        let locator = origin
            .native_prior_checkpoint
            .as_ref()
            .ok_or(Stop::SelectedSourceMismatch)?;
        let dependency = origin
            .producer_dependency_identity
            .ok_or(Stop::SelectedSourceMismatch)?;

        Ok(RequiredPublicationPredecessor {
            correspondence: Arc::clone(&origin.correspondence),
            producer: locator.producer().to_owned(),
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
