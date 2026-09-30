use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationOutputCardinality,
    WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily,
};
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationOperationDecisionReadTarget,
};
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_relational::facade::storage::RecordLifecycleState;

use super::*;
use crate::domain_computation::primary_graph::application_attempt::OutputRoleUse;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputPosture,
    WorthQueryPriorOutputDenial, WorthQueryPriorOutputDenialKind,
};

/// The read of a prior fixed role: the identity for an exactly-one role, an
/// `Option` of it for an at-most-one role.
pub(in crate::domain_computation::primary_graph) type WorthQueryPriorOutputRead<Schema, Role> =
    <<Role as WorthQueryApplicationOutputRole>::Cardinality as WorthQueryApplicationOutputCardinality>::Read<
        WorthQueryInvariantEntityIdentity<Schema, <Role as WorthQueryApplicationOutputRole>::Entity>,
    >;

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
    /// Read the fixed role `Role` that `PriorBinding` last committed. `Role`
    /// must belong to the output contract of `PriorBinding`, and the contract
    /// must declare it; otherwise the read fails to compile.
    pub fn prior_output<PriorBinding, Role>(
        &mut self,
    ) -> Result<WorthQueryPriorOutputRead<Schema, Role>, WorthQueryPriorOutputDenial>
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = PriorBinding::Output>,
        Role::Entity: OperationReads<Operation>,
    {
        self.prior_output_if_present::<PriorBinding, Role>()?
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::Unavailable,
                    Role::NAME,
                )
            })
    }

    /// A missing correspondence for this exact binding is absence, not a denial.
    /// Within a present correspondence, an unbound at-most-one role reads as
    /// `None` and an unbound exactly-one role is refused. Missing scope or
    /// product context and invalid or stale correspondence remain denials.
    pub fn prior_output_if_present<PriorBinding, Role>(
        &mut self,
    ) -> Result<Option<WorthQueryPriorOutputRead<Schema, Role>>, WorthQueryPriorOutputDenial>
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = PriorBinding::Output>,
        Role::Entity: OperationReads<Operation>,
    {
        let role = OutputRoleUse::fixed::<Role>();
        self.admit_prior_entity::<Role::Entity>(Role::NAME)?;
        let Some(correspondence) = self.select_prior_correspondence::<PriorBinding>(Role::NAME)?
        else {
            return Ok(None);
        };
        self.charge_role_work(Role::NAME)?;
        let bound = correspondence
            .bound_entity(&role)
            .map_err(|denial| WorthQueryPriorOutputDenial::projection(Role::NAME, denial))?;
        let identity = bound
            .map(|entity_id| self.live_prior_identity::<Role::Entity>(Role::NAME, entity_id))
            .transpose()?;
        <Role::Cardinality as WorthQueryApplicationOutputCardinality>::read(identity)
            .map(Some)
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::MissingRole,
                    Role::NAME,
                )
            })
    }

    /// Read every currently visible member of the family `Family` that
    /// `PriorBinding` last committed.
    ///
    /// Results are ordered by semantic role name. Retired correspondence is
    /// excluded; a create/preserve member that is not visible is an integrity
    /// denial rather than an incomplete inventory. A family that the prior
    /// contract does not declare fails to compile.
    #[allow(clippy::type_complexity)]
    pub fn prior_output_family<PriorBinding, Family>(
        &mut self,
    ) -> Result<
        Vec<WorthQueryPriorOutputFamilyMember<Schema, PriorBinding, Family::Entity>>,
        WorthQueryPriorOutputDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Family:
            WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = PriorBinding::Output>,
        Family::Entity: OperationReads<Operation>,
    {
        self.prior_output_family_if_present::<PriorBinding, Family>()?
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::Unavailable,
                    Family::PREFIX,
                )
            })
    }

    /// Read a generated-role family when this exact prior binding has
    /// correspondence.
    ///
    /// `None` means no correspondence exists for `PriorBinding` at the
    /// selected product occurrence and generation. Entity, work, and
    /// live-identity failures remain denials.
    #[allow(clippy::type_complexity)]
    pub fn prior_output_family_if_present<PriorBinding, Family>(
        &mut self,
    ) -> Result<
        Option<Vec<WorthQueryPriorOutputFamilyMember<Schema, PriorBinding, Family::Entity>>>,
        WorthQueryPriorOutputDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Family:
            WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = PriorBinding::Output>,
        Family::Entity: OperationReads<Operation>,
    {
        let prefix = Family::PREFIX;
        self.admit_prior_entity::<Family::Entity>(prefix)?;
        let Some(correspondence) = self.select_prior_correspondence::<PriorBinding>(prefix)? else {
            return Ok(None);
        };
        let project = |denial| WorthQueryPriorOutputDenial::projection(prefix, denial);
        let mut examined = 0_usize;
        let mut live = 0_usize;
        for entry in correspondence.family_members::<Family>().map_err(project)? {
            self.charge_role_work(prefix)?;
            let (_, posture, _) = entry.map_err(project)?;
            examined += 1;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                continue;
            }
            live += 1;
        }
        self.require_role_budget(examined.saturating_add(live), prefix)?;
        let mut members = Vec::with_capacity(live);
        for entry in correspondence.family_members::<Family>().map_err(project)? {
            self.consume_admitted_role_work();
            let (role, posture, entity) = entry.map_err(project)?;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                continue;
            }
            self.consume_admitted_role_work();
            members.push(WorthQueryPriorOutputFamilyMember {
                identity: self.live_prior_identity::<Family::Entity>(role, entity)?,
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
