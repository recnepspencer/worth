//! Only recorded owner release admits logical discharge.
use super::super::QueryObligationDenial as Denial;
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::HomeIdentity;

#[derive(Clone, Debug, Default)]
pub(in crate::domain_computation::primary_graph::query_obligations) struct ManagedCleanupFold {
    rows: BTreeMap<ManagedCleanupObligationIdentity, QueryManagedCleanupObligation>,
}
impl ManagedCleanupFold {
    pub(super) fn rows(&self) -> impl Iterator<Item = &QueryManagedCleanupObligation> {
        self.rows.values()
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn validate(
        &self,
        record: &QueryManagedCleanupRecord,
        home: HomeIdentity,
        position: u64,
    ) -> Result<(), Denial> {
        match record {
            QueryManagedCleanupRecord::Retain(row) => {
                if row.identity.home() != home
                    || row.identity.acquire_position().get() != position
                    || row.original.is_some_and(|original| original.home() != home)
                {
                    return Err(Denial::ObligationIdentityMismatch);
                }
                if row.release.is_some() || row.run.is_empty() || row.run.len() > 4096 {
                    return Err(Denial::MalformedFrame);
                }
                if self.rows.contains_key(&row.identity) {
                    return Err(Denial::DuplicateObligation);
                }
            }
            QueryManagedCleanupRecord::Released { identity, .. }
            | QueryManagedCleanupRecord::Failed { identity, .. }
            | QueryManagedCleanupRecord::Discharge { identity } => {
                if identity.home() != home {
                    return Err(Denial::ForeignHome);
                }
                let row = self.rows.get(identity).ok_or(Denial::UnknownObligation)?;
                if matches!(
                    row.failure,
                    WorthQueryMutationCleanupFailureKind::AffinityInconsistent
                        | WorthQueryMutationCleanupFailureKind::CompletionInconsistent
                ) {
                    return Err(Denial::CleanupInconsistent {
                        identity: *identity,
                    });
                }
                match record {
                    QueryManagedCleanupRecord::Released { .. }
                    | QueryManagedCleanupRecord::Failed { .. }
                        if row.release.is_some() =>
                    {
                        return Err(Denial::DuplicateObligation);
                    }
                    QueryManagedCleanupRecord::Discharge { .. } if row.release.is_none() => {
                        return Err(Denial::CleanupReleaseUnrecorded {
                            identity: *identity,
                        });
                    }
                    QueryManagedCleanupRecord::Released { release, .. }
                        if release.disposition
                            == WorthQueryManagedRunCleanupDisposition::CleanupPending =>
                    {
                        return Err(Denial::CleanupReleaseUnrecorded {
                            identity: *identity,
                        });
                    }
                    QueryManagedCleanupRecord::Released { .. }
                    | QueryManagedCleanupRecord::Failed { .. }
                    | QueryManagedCleanupRecord::Discharge { .. } => {}
                    QueryManagedCleanupRecord::Retain(_) => unreachable!(),
                }
            }
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn apply(
        &mut self,
        record: QueryManagedCleanupRecord,
    ) {
        match record {
            QueryManagedCleanupRecord::Retain(row) => {
                self.rows.insert(row.identity, row);
            }
            QueryManagedCleanupRecord::Released { identity, release } => {
                self.rows
                    .get_mut(&identity)
                    .expect("validated original")
                    .release = Some(release);
            }
            QueryManagedCleanupRecord::Failed { identity, failure } => {
                self.rows
                    .get_mut(&identity)
                    .expect("validated original")
                    .failure = failure;
            }
            QueryManagedCleanupRecord::Discharge { identity } => {
                self.rows.remove(&identity);
            }
        }
    }
}

impl super::super::QueryObligationLedger {
    pub fn managed_cleanup(&self) -> impl Iterator<Item = &QueryManagedCleanupObligation> {
        self.cleanup.rows()
    }
}
