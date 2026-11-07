//! Native exact equality shares the genuine candidate Source and original read loans.
use super::super::{prefix, schema::*};
use worth_query_decl::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationManagedIndexColdSource},
    application_schema::*,
};
use worth_query_host::facade::application_invariants::*;

pub(in super::super) struct ExactName;
impl ApplicationInvariantMarkerIdentity<Catalog> for ExactName {
    const IDENTIFIER: &'static str = "AtMostOneExactShaName";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(in super::super) fn definition() -> ApplicationInvariantDefinition<Catalog, ExactName> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Directory".into()),
        ApplicationInvariantScopeTarget::Entity("Item".into()),
        ApplicationInvariantScopeTarget::Relation("Contains".into()),
    ];
    ApplicationInvariantDefinition::new(
        ExactName::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            closure.clone(),
            closure,
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
    .managed_index_source(prefix::PrefixIndex::reference())
    .managed_index_cold_sources(ApplicationManagedIndexColdSource::AdmitRebuild)
}

#[derive(Clone, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
pub struct ExactNameRefusal {
    pub scope: [u64; 3],
    pub committed: Option<u64>,
    pub proposed: u64,
}
worth_query_decl::facade::worth_query_value_binding!(pub ExactNameRefusalBinding for ExactNameRefusal {
    identity:"maintained-catalog.exact-name-refusal.v1",scalar:String,
    canonical:ValueDecodeBounds::new(256,128,16,8),
});
pub(in super::super) struct Rule {
    index: WorthQueryApplicationInvariantManagedIndexBinding<Catalog, prefix::PrefixIndex>,
}
pub(in super::super) fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Catalog>,
) -> Result<Rule, String> {
    Ok(Rule {
        index: resolver
            .typed_managed_index(prefix::PrefixIndex::reference())
            .ok_or("missing exact-name Source")?,
    })
}
impl WorthQueryApplicationInvariantRule<Catalog> for Rule {
    type Scope = WorthQueryPreparedInvariantManagedIndex<Catalog, prefix::PrefixIndex>;
    type Refusal = ExactNameRefusal;
    type RefusalBinding = ExactNameRefusalBinding;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantError<ExactNameRefusal>> {
        planner
            .retain_managed_index(&self.index)
            .map_err(WorthQueryApplicationInvariantError::from)
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Catalog>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantError<ExactNameRefusal>>> {
        Some(if budget.checkpoint(1) {
            self.prepare_scope(planner)
        } else {
            Err(WorthQueryApplicationInvariantError::Stopped)
        })
    }
    fn evaluate(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        _: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Self::Refusal>,
        WorthQueryApplicationInvariantError<ExactNameRefusal>,
    > {
        Err(WorthQueryApplicationInvariantError::CheckedHookRequired)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Catalog>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<Self::Refusal>,
            WorthQueryApplicationInvariantError<ExactNameRefusal>,
        >,
    > {
        Some((|| {
            if !budget.checkpoint(1) {
                return Err(WorthQueryApplicationInvariantError::Stopped);
            }
            let pair = context.refined_managed_index(scope).map_err(execution)?;
            let probe = super::super::query::Probe {
                prefix: "sha".into(),
            };
            let after = pair.proposed().query(&probe).map_err(execution)?;
            let committed = match pair.committed().query(&probe) {
                Ok(rows) => Some(rows.len() as u64),
                Err(cause) if cause.source_unavailable() => None,
                Err(cause) => return Err(execution(cause)),
            };
            Ok(if after.len() > 1 {
                WorthQueryApplicationInvariantVerdict::Violation(ExactNameRefusal {
                    scope: {
                        let id = scope.scope_identity();
                        [
                            u64::from(id.partition_value()),
                            id.local_slot_value(),
                            u64::from(id.generation_value()),
                        ]
                    },
                    committed,
                    proposed: after.len() as u64,
                })
            } else {
                WorthQueryApplicationInvariantVerdict::Pass
            })
        })())
    }
}
fn execution(
    cause: impl Into<WorthQueryApplicationInvariantError<ExactNameRefusal>>,
) -> WorthQueryApplicationInvariantError<ExactNameRefusal> {
    cause.into()
}
