//! One prepared performed-source link serves ordinary and restored admission.
use super::*;
use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding as Scope;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind as Kind;
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductBranchIncarnation};

pub(super) struct PreparedPerformedJoin {
    commit: CompositeCommitIdentity,
    epoch: SourceEpoch,
    source: WorthQueryPerformedOutputDemandSource,
    obligations: Vec<PerformedOutputObligation>,
    growth: Option<record_preparation::PreparedSourceCommitGrowth>,
}
impl DemandRegistryState {
    pub(super) fn prepare_performed_join(
        &self,
        requested: &WorthQueryOutputDemandKey,
        selected: &WorthQueryOutputDemandKey,
        commit: &CompositeCommitIdentity,
        scope: Scope,
        occurrence: ProductBranchIncarnation,
    ) -> Result<PreparedPerformedJoin, WorthQueryOutputDemandDenial> {
        let custody = self
            .source_custody
            .get(commit)
            .ok_or_else(|| refusal(Kind::DuplicatePerformedSource))?;
        if let Some(cause) = &custody.retired {
            return Err(cause.clone());
        }
        if let Some(cause) = custody.source_denial(&requested.source) {
            return Err(cause);
        }
        if !custody.available(&requested.source, scope) || custody.occurrence != occurrence {
            return Err(refusal(if custody.bound_sources.is_some() {
                Kind::ForeignSource
            } else {
                Kind::DuplicatePerformedSource
            }));
        }
        if let Some(record) = self.records.get(selected) {
            if record.product_occurrence != occurrence || record.source_scope != Some(scope) {
                return Err(refusal(Kind::ForeignSource));
            }
            if let DemandState::Failed(cause) = &record.state {
                return Err(cause.clone());
            }
        }
        let obligations = admission::prepare_performed_obligation_slots(self, selected)?;
        let growth = self
            .records
            .get(selected)
            .and_then(|record| (!record.source_commits.contains(commit)).then_some(record))
            .map(|record| self.prepare_source_commit_growth(record))
            .transpose()?
            .flatten();
        let mut source = custody
            .source
            .as_ref()
            .expect("available custody retains its original source")
            .clone();
        source.output_source_identity = Some(requested.source.clone());
        Ok(PreparedPerformedJoin {
            commit: commit.clone(),
            epoch: requested.source.clone(),
            source,
            obligations,
            growth,
        })
    }
    pub(super) fn install_performed_join(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        join: PreparedPerformedJoin,
    ) {
        let PreparedPerformedJoin {
            commit,
            epoch,
            source,
            obligations,
            growth,
        } = join;
        self.source_custody
            .get_mut(&commit)
            .expect("prepared source remains locked")
            .finish_admission(epoch.clone());
        let mut record = self
            .records
            .get_mut(key)
            .expect("prepared row is installed before its source link");
        if !record.source_commits.contains(&commit) {
            if let Some(growth) = growth {
                growth.install(&mut record);
            }
            record.source_commits.push(commit.clone());
        }
        let previous = std::mem::replace(&mut record.performed_obligations, obligations);
        record.performed_obligations.extend(previous);
        record
            .performed_obligations
            .push(PerformedOutputObligation {
                source_commit: commit,
                source: epoch,
            });
        if matches!(record.state, DemandState::Admitted) && record.performed_source.is_none() {
            record.performed_source = Some(source);
        }
        record.finish();
    }
}
fn refusal(kind: Kind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        kind,
        "performed source differs from its prepared publication",
    )
}
