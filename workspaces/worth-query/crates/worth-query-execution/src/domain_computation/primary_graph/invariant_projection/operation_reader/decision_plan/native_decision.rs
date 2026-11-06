//! Admission and tracked field facts are independent of application and member types.
use super::super::super::native_read::NativeIdentity;
use super::*;
use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::application_schema::fragment::ColumnIdentity;
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts as Contracts;

pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) struct NativeDecision<
    'a,
> {
    layout:
        &'a crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
    bindings:
        &'a worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    authority: u64,
    contracts: Option<&'a Contracts>,
    facts: &'a mut super::super::DecisionReads,
}
impl<'r, S: ApplicationSchema, O>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, 'r, S, O>
{
    #[inline(always)]
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) fn native_decision(
        &mut self,
    ) -> NativeDecision<'_> {
        NativeDecision {
            layout: self.reader.layout,
            bindings: self.reader.value_bindings,
            authority: self.reader.authority_identity,
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
        contracts: Option<&'a Contracts>,
        facts: &'a mut super::super::DecisionReads,
    ) -> NativeDecision<'a> {
        NativeDecision {
            layout,
            bindings,
            authority,
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
        if identity.authority != self.authority || identity.entity.as_ref() != field.entity() {
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
pub(super) fn admit_target(
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
