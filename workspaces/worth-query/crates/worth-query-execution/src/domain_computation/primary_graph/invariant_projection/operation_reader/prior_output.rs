use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationOutputContract,
};
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationOperationDecisionReadTarget,
};
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_relational::facade::storage::RecordLifecycleState;

use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputAction, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputPosture, WorthQueryApplicationOutputRole,
    WorthQueryApplicationOutputRoleFamily, WorthQueryPriorOutputDenial,
    WorthQueryPriorOutputDenialKind,
};

/// One currently visible member of a prior generated-output family.
pub struct WorthQueryPriorOutputFamilyMember<Schema, Binding, Entity> {
    role: String,
    published_posture: WorthQueryApplicationOutputPosture,
    identity: WorthQueryInvariantEntityIdentity<Schema, Entity>,
    _binding: PhantomData<fn() -> Binding>,
}

impl<Schema, Binding, Entity> WorthQueryPriorOutputFamilyMember<Schema, Binding, Entity> {
    pub fn role(&self) -> &str {
        &self.role
    }

    pub const fn published_posture(&self) -> WorthQueryApplicationOutputPosture {
        self.published_posture
    }

    pub const fn identity(&self) -> &WorthQueryInvariantEntityIdentity<Schema, Entity> {
        &self.identity
    }

    pub fn into_identity(self) -> WorthQueryInvariantEntityIdentity<Schema, Entity> {
        self.identity
    }
}

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub fn prior_output<Binding, Entity, Action>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryPriorOutputDenial>
    where
        Binding: 'static,
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        self.admit_prior_entity::<Entity>(role.name())?;
        let correspondence = self.prior_correspondence::<Binding>(role.name())?;
        self.require_role_budget(1, role.name())?;
        self.reader.work_budget.consume(1);
        self.reader.work.record_output_lineage_role_lookup();
        let role_name = role.name().to_owned();
        let output = correspondence
            .entity(role)
            .map_err(|denial| WorthQueryPriorOutputDenial::projection(&role_name, denial))?;
        self.live_prior_identity::<Entity>(&role_name, output.entity_id())
    }

    /// Read every currently visible member of one declared generated-role family.
    ///
    /// Results are ordered by semantic role name. Retired correspondence is
    /// excluded; a create/preserve member that is not visible is an integrity
    /// denial rather than an incomplete inventory.
    pub fn prior_output_family<Binding, Entity>(
        &mut self,
        family: WorthQueryApplicationOutputRoleFamily<Binding, Entity>,
    ) -> Result<
        Vec<WorthQueryPriorOutputFamilyMember<Schema, Binding, Entity>>,
        WorthQueryPriorOutputDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
    {
        let subject = family.prefix();
        self.prior_output_family_if_present(family)?.ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(WorthQueryPriorOutputDenialKind::Unavailable, subject)
        })
    }

    /// Read a generated-role family when this exact prior binding has correspondence.
    ///
    /// `None` means no correspondence exists for `Binding` at the selected product
    /// occurrence and generation. Declaration, entity, work, and live-identity
    /// failures remain denials.
    pub fn prior_output_family_if_present<Binding, Entity>(
        &mut self,
        family: WorthQueryApplicationOutputRoleFamily<Binding, Entity>,
    ) -> Result<
        Option<Vec<WorthQueryPriorOutputFamilyMember<Schema, Binding, Entity>>>,
        WorthQueryPriorOutputDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
    {
        self.admit_prior_entity::<Entity>(family.prefix())?;
        let declaration =
            <Binding::Output as ApplicationMutationOutputContract<Schema>>::ROLE_FAMILIES
                .iter()
                .find(|candidate| candidate.prefix() == family.prefix())
                .ok_or_else(|| {
                    WorthQueryPriorOutputDenial::new(
                        WorthQueryPriorOutputDenialKind::UndeclaredFamily,
                        family.prefix(),
                    )
                })?;
        if declaration.entity() != Entity::IDENTIFIER {
            return Err(WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::EntityMismatch,
                family.prefix(),
            ));
        }
        let Some(correspondence) = self.select_prior_correspondence::<Binding>(family.prefix())?
        else {
            return Ok(None);
        };
        let mut examined = 0_usize;
        let mut live = 0_usize;
        for entry in correspondence
            .binding_family_entries::<Binding, Entity>(family.prefix())
            .map_err(|denial| WorthQueryPriorOutputDenial::projection(family.prefix(), denial))?
        {
            self.charge_role_work(family.prefix())?;
            let (_, posture, _) = entry.map_err(|denial| {
                WorthQueryPriorOutputDenial::projection(family.prefix(), denial)
            })?;
            examined += 1;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                continue;
            }
            live += 1;
        }
        self.require_role_budget(examined.saturating_add(live), family.prefix())?;
        let mut members = Vec::with_capacity(live);
        for entry in correspondence
            .binding_family_entries::<Binding, Entity>(family.prefix())
            .map_err(|denial| WorthQueryPriorOutputDenial::projection(family.prefix(), denial))?
        {
            self.consume_admitted_role_work();
            let (role, posture, entity) = entry.map_err(|denial| {
                WorthQueryPriorOutputDenial::projection(family.prefix(), denial)
            })?;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                continue;
            }
            self.consume_admitted_role_work();
            members.push(WorthQueryPriorOutputFamilyMember {
                identity: self.live_prior_identity::<Entity>(role, entity)?,
                role: role.to_owned(),
                published_posture: posture,
                _binding: PhantomData,
            });
        }
        Ok(Some(members))
    }

    fn admit_prior_entity<Entity>(
        &mut self,
        subject: &str,
    ) -> Result<(), WorthQueryPriorOutputDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Entity {
            entity: Entity::IDENTIFIER.to_owned(),
        })
        .map_err(|_| {
            WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::UndeclaredDecisionTarget,
                subject,
            )
        })
    }

    fn prior_correspondence<Binding: 'static>(
        &mut self,
        subject: &str,
    ) -> Result<
        std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
        WorthQueryPriorOutputDenial,
    > {
        self.select_prior_correspondence::<Binding>(subject)?
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::Unavailable,
                    subject,
                )
            })
    }

    fn select_prior_correspondence<Binding: 'static>(
        &mut self,
        subject: &str,
    ) -> Result<
        Option<std::sync::Arc<WorthQueryApplicationOutputCorrespondence>>,
        WorthQueryPriorOutputDenial,
    > {
        let binding_type = std::any::TypeId::of::<Binding>();
        if let Some(correspondence) = self.reader.prior_output_bindings.get(&binding_type) {
            return Ok(Some(std::sync::Arc::clone(correspondence)));
        }
        let scope = self.operation_scope.clone().ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(WorthQueryPriorOutputDenialKind::Unavailable, subject)
        })?;
        let occurrence = self.reader.selected_product_occurrence.ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(WorthQueryPriorOutputDenialKind::Unavailable, subject)
        })?;
        let generation = self.reader.selected_product_generation.ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(WorthQueryPriorOutputDenialKind::Unavailable, subject)
        })?;
        let partition = self
            .reader
            .selected_source_partition_identity
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::Unavailable,
                    subject,
                )
            })?;
        self.require_role_budget(1, subject)?;
        let selection = self
            .reader
            .output_lineage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resolve_binding::<Binding>(
                &scope,
                occurrence,
                generation,
                partition,
                self.reader.work_budget.remaining().saturating_sub(1),
            )
            .map_err(|_| {
                self.reader.work_budget.mark_exceeded();
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::WorkBudgetExceeded,
                    subject,
                )
            })?;
        self.reader.work_budget.consume(selection.source_lookups);
        self.reader
            .work
            .record_output_lineage_selection(selection.source_lookups);
        let Some(correspondence) = selection.correspondence else {
            return Ok(None);
        };
        self.reader
            .prior_output_bindings
            .insert(binding_type, std::sync::Arc::clone(&correspondence));
        Ok(Some(correspondence))
    }

    fn require_role_budget(
        &mut self,
        required: usize,
        subject: &str,
    ) -> Result<(), WorthQueryPriorOutputDenial> {
        self.reader
            .work_budget
            .can_afford(required)
            .then_some(())
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::WorkBudgetExceeded,
                    subject,
                )
            })
    }

    fn charge_role_work(&mut self, subject: &str) -> Result<(), WorthQueryPriorOutputDenial> {
        self.require_role_budget(1, subject)?;
        self.consume_admitted_role_work();
        Ok(())
    }

    fn consume_admitted_role_work(&mut self) {
        self.reader.work_budget.consume(1);
        self.reader.work.record_output_lineage_role_lookup();
    }

    fn live_prior_identity<Entity>(
        &mut self,
        role: &str,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryPriorOutputDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        let projected = self
            .reader
            .runtime
            .read_truth()
            .project_snapshot(self.reader.snapshot)
            .and_then(|view| {
                view.entity_record_with_projection_scope(
                    entity_id,
                    ProjectionAspectScope::empty(),
                    |record| Some((record.kind_id(), record.lifecycle())),
                )
            })
            .filter(|(_, lifecycle)| *lifecycle == RecordLifecycleState::Live)
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::OutputUnavailable,
                    role,
                )
            })?;
        let entity = self.reader.layout.entity_name(projected.0).ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(WorthQueryPriorOutputDenialKind::EntityMismatch, role)
        })?;
        if entity != Entity::IDENTIFIER {
            return Err(WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::EntityMismatch,
                role,
            ));
        }
        self.reader.realized_scope.record(entity_id);
        let identity = WorthQueryInvariantEntityIdentity {
            entity_id,
            kind: projected.0,
            entity: Arc::from(entity),
            authority_identity: self.reader.authority_identity,
            _marker: PhantomData,
        };
        self.require_decision_entity(
            &identity,
            ApplicationEntityRef::from_schema_identifier(Entity::IDENTIFIER),
        )
        .map_err(|_| {
            WorthQueryPriorOutputDenial::new(WorthQueryPriorOutputDenialKind::Unavailable, role)
        })?;
        Ok(identity)
    }
}
