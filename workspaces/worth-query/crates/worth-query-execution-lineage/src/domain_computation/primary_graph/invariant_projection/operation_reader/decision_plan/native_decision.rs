//! Admission and tracked field facts are independent of application and member types.
use super::super::super::native_read::NativeIdentity;
use super::*;
use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::application_schema::fragment::ColumnIdentity;
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts as Contracts;

pub struct NativeDecision<'a> {
    layout:
        &'a crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
    bindings:
        &'a worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    authority: u64,
    phase: Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    contracts: Option<&'a Contracts>,
    facts: &'a mut super::super::DecisionReads,
}
impl<'r, S: ApplicationSchema, O>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, 'r, S, O>
{
    #[inline(always)]
    pub(crate) fn native_decision(&mut self) -> NativeDecision<'_> {
        NativeDecision {
            layout: self.reader.layout,
            bindings: self.reader.value_bindings,
            authority: self.reader.authority_identity,
            phase: self
                .reader
                .native
                .candidate_identity()
                .map(|id| id.qualification()),
            contracts: self.admitted_contracts.as_deref(),
            facts: self.decision_facts,
        }
    }
}
impl NativeDecision<'_> {
    pub fn new<'a>(
        layout: &'a crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
        bindings: &'a worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
        authority: u64,
        phase: Option<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification,
        >,
        contracts: Option<&'a Contracts>,
        facts: &'a mut super::super::DecisionReads,
    ) -> NativeDecision<'a> {
        NativeDecision {
            layout,
            bindings,
            authority,
            phase,
            contracts,
            facts,
        }
    }
    pub fn admit_target(
        &self,
        target: &ApplicationOperationDecisionReadTarget,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        admit_target(self.contracts, target)
    }
    pub fn record_outcome(
        &mut self,
        key: WorthQueryApplicationFactKey,
        outcome: super::super::decision_reads::DecisionReadOutcome,
    ) {
        self.facts.record(key, outcome);
    }

    pub fn require_field(
        &mut self,
        identity: NativeIdentity<'_>,
        field: ColumnIdentity,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        field.installed(self.bindings.fragments()).map_err(|_| {
            WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::FieldNotInstalled,
                field.field(),
            )
        })?;
        self.tracked_field(identity, &field.locus()).map(|_| ())
    }
    pub fn tracked_field(
        &mut self,
        identity: NativeIdentity<'_>,
        field: &ApplicationFieldBindingLocus,
    ) -> Result<AspectFieldLocator, WorthQueryInvariantDecisionPlanDenial> {
        admit_target(
            self.contracts,
            &ApplicationOperationDecisionReadTarget::Field {
                entity: field.entity().to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
            },
        )?;
        if !identity.local_to(self.authority, self.phase, field.entity()) {
            return Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                field.entity(),
            ));
        }
        let locator = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
            .ok_or_else(|| {
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::FieldNotInstalled,
                    field.field(),
                )
            })?;
        self.record_field(identity, field, locator.clone());
        Ok(locator)
    }
    /// Records the pre-image of every field of `identity`'s entity that this
    /// operation declares both writable and readable, exactly as a decision
    /// read of each would (the same tracked fact, sealed with the decision).
    ///
    /// A handler that targets an entity may then write those fields without a
    /// discarded read. Each write still finds its sealed observed fact, so a
    /// concurrent change to any observed field still makes the attempt Stale.
    /// A writable field that is not a declared read stays unobserved and still
    /// refuses `UnobservedWrite`; a foreign identity records nothing.
    pub fn observe_declared_writes(
        &mut self,
        identity: NativeIdentity<'_>,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        let Some(contracts) = self.contracts else {
            return Ok(());
        };
        let entity = identity.entity.as_ref();
        if !identity.local_to(self.authority, self.phase, entity) {
            return Ok(());
        }
        let writable = self
            .layout
            .entity_fields(entity)
            .filter(|field| field_presence::may_write(contracts.touches(), entity, field.locator))
            .filter(|field| {
                graph_reads_admit_target(
                    contracts.graph_reads(),
                    &ApplicationOperationDecisionReadTarget::Field {
                        entity: entity.to_owned(),
                        aspect: field.aspect.to_owned(),
                        field: field.field.to_owned(),
                    },
                )
            })
            .map(|field| ApplicationFieldBindingLocus::new(entity, field.aspect, field.field))
            .collect::<Vec<_>>();
        for field in writable {
            self.tracked_field(identity, &field)?;
        }
        Ok(())
    }
    pub fn record_field(
        &mut self,
        identity: NativeIdentity<'_>,
        field: &ApplicationFieldBindingLocus,
        locator: AspectFieldLocator,
    ) {
        let entity = field.entity();
        if let Some(presence) = self.layout.record_presence_locator(field).cloned()
            && presence != locator
            && self.contracts.is_some_and(|contract| {
                field_presence::may_write(contract.touches(), entity, &locator)
            })
        {
            self.facts.record(
                WorthQueryApplicationFactKey::Field {
                    entity: entity.to_owned(),
                    entity_id: identity.entity_id,
                    locator: presence,
                },
                DecisionReadOutcome::Observed,
            );
        }
        if self.layout.field_is_optional(entity, &locator)
            && self.contracts.is_some_and(|contract| {
                field_presence::may_write(contract.touches(), entity, &locator)
            })
        {
            self.facts.record(
                WorthQueryApplicationFactKey::AspectPresence {
                    entity: entity.to_owned(),
                    entity_id: identity.entity_id,
                    locator: locator.clone(),
                },
                DecisionReadOutcome::Observed,
            );
        }
        self.facts.record(
            WorthQueryApplicationFactKey::Field {
                entity: entity.to_owned(),
                entity_id: identity.entity_id,
                locator,
            },
            DecisionReadOutcome::Observed,
        );
    }
}
pub fn admit_target(
    contracts: Option<&Contracts>,
    target: &ApplicationOperationDecisionReadTarget,
) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
    if contracts.is_none_or(|contract| match target {
        ApplicationOperationDecisionReadTarget::ManagedIndex { parent } => {
            contract.managed_index_reads().contains(parent)
        }
        _ => graph_reads_admit_target(contract.graph_reads(), target),
    }) {
        Ok(())
    } else {
        Err(WorthQueryInvariantDecisionPlanDenial::new(
            WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget,
            format!("{target:?}"),
        ))
    }
}
