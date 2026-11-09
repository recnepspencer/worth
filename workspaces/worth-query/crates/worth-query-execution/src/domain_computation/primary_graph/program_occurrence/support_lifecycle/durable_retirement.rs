//! Durable retirement reservation and acknowledged terminal posture.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQueryProgramSupportRetirementFailure as Failure;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::CanonicalDigestId;

impl WorthQueryProgramSupportLifecycle {
    pub(in crate::domain_computation::primary_graph) fn begin_retirement_in_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        revision: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramSupportRetirementAttempt, Failure> {
        let Some(medium) = self.medium.get() else {
            return self.begin_retirement(revision).map_err(Failure::Policy);
        };
        let driver = self.lock_publication().map_err(Failure::Durability)?;
        driver
            .finish_pending(medium.as_ref(), request)
            .map_err(Failure::Durability)?;
        let before = {
            let mut state = self.state();
            let entry = state.entries.get_mut(revision).ok_or(Failure::Policy(
                WorthQueryProgramSupportRetirementDenial::UnrosteredProgram {
                    revision: *revision,
                },
            ))?;
            if entry.retirement_claimed {
                return Err(Failure::Policy(
                    WorthQueryProgramSupportRetirementDenial::RetirementInProgress {
                        revision: *revision,
                    },
                ));
            }
            if entry.status == ProgramSupportStatus::Retired {
                return Err(Failure::Policy(
                    WorthQueryProgramSupportRetirementDenial::AlreadyRetired {
                        revision: *revision,
                    },
                ));
            }
            entry.retirement_claimed = true;
            entry.status
        };
        let publication = match before {
            ProgramSupportStatus::Active => driver.publish(
                medium.as_ref(),
                QueryProgramSupportRecord::Status {
                    revision: CanonicalDigestId::new(*revision.as_bytes()),
                    before: ProgramSupportStatus::Active,
                    after: ProgramSupportStatus::Retiring,
                },
                request,
            ),
            ProgramSupportStatus::Retiring => Ok(()),
            ProgramSupportStatus::Retired => {
                unreachable!("retired support refused before reservation")
            }
        };
        if let Err(cause) = publication {
            self.clear_retirement_claim(revision);
            return Err(Failure::Durability(cause));
        }
        Ok(WorthQueryProgramSupportRetirementAttempt {
            state: Arc::clone(&self.state),
            revision: *revision,
            finished: false,
        })
    }
    fn clear_retirement_claim(&self, revision: &ApplicationProgramRevision) {
        self.state()
            .entries
            .get_mut(revision)
            .expect("actual retiring entry")
            .retirement_claimed = false;
    }
    pub(in crate::domain_computation::primary_graph) fn acknowledge_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        revision: &ApplicationProgramRevision,
        retired: bool,
    ) -> Result<(), Failure> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let result = (|| {
            let driver = self.lock_publication()?;
            driver.publish(
                medium.as_ref(),
                QueryProgramSupportRecord::Status {
                    revision: CanonicalDigestId::new(*revision.as_bytes()),
                    before: ProgramSupportStatus::Retiring,
                    after: if retired {
                        ProgramSupportStatus::Retired
                    } else {
                        ProgramSupportStatus::Active
                    },
                },
                request,
            )
        })();
        self.clear_retirement_claim(revision);
        result.map_err(Failure::Durability)
    }
}
