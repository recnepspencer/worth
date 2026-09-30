use std::marker::PhantomData;

use worth_query_declaration::facade::application_operation::{
    WorthQueryApplicationOutputAction, WorthQueryApplicationOutputCardinality,
    WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily,
};

use super::{
    OutputRoleUse, WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputEntity,
    WorthQueryApplicationOutputFamilyEntry, WorthQueryApplicationOutputProjectionDenial,
};

/// One commit's output correspondence, read as the output contract
/// `Contract`.
///
/// A receipt or settlement stores its correspondence erased.
/// `outputs_of::<Contract>()` checks once, at runtime, that the commit was
/// made under `Contract`, and returns this view. Every read on the view names a role or family marker of
/// `Contract` itself: a marker of another contract, or one `Contract` does not
/// declare exactly as read, fails to compile.
pub struct WorthQueryApplicationTypedOutputCorrespondence<'correspondence, Contract> {
    correspondence: &'correspondence WorthQueryApplicationOutputCorrespondence,
    _contract: PhantomData<fn() -> Contract>,
}

impl<Contract> Copy for WorthQueryApplicationTypedOutputCorrespondence<'_, Contract> {}

impl<Contract> Clone for WorthQueryApplicationTypedOutputCorrespondence<'_, Contract> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Contract> std::fmt::Debug for WorthQueryApplicationTypedOutputCorrespondence<'_, Contract> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("WorthQueryApplicationTypedOutputCorrespondence")
            .field(self.correspondence)
            .finish()
    }
}

impl<'correspondence, Contract: 'static>
    WorthQueryApplicationTypedOutputCorrespondence<'correspondence, Contract>
{
    /// The view of `correspondence`, which the caller has checked was
    /// committed under `Contract`.
    pub(super) const fn new(
        correspondence: &'correspondence WorthQueryApplicationOutputCorrespondence,
    ) -> Self {
        Self {
            correspondence,
            _contract: PhantomData,
        }
    }

    /// Project the entity committed under one fixed role. An exactly-one role
    /// yields the entity; an at-most-one role yields `Option`, `None` when the
    /// commit left the role unbound.
    #[allow(clippy::type_complexity)]
    pub fn entity<Role>(
        &self,
    ) -> Result<
        <Role::Cardinality as WorthQueryApplicationOutputCardinality>::Read<
            WorthQueryApplicationOutputEntity<Role::Entity, Role::Action>,
        >,
        WorthQueryApplicationOutputProjectionDenial,
    >
    where
        Role: WorthQueryApplicationOutputRole<Contract = Contract>,
    {
        let bound = self
            .correspondence
            .bound_entity(&OutputRoleUse::fixed::<Role>())?
            .map(WorthQueryApplicationOutputEntity::new);
        Role::Cardinality::read(bound)
            .ok_or(WorthQueryApplicationOutputProjectionDenial::MissingRole)
    }

    /// Project the entity committed under the member of `Family` named by
    /// `suffix`, which the commit bound with `Action`.
    pub fn member<Family, Action>(
        &self,
        suffix: &str,
    ) -> Result<
        WorthQueryApplicationOutputEntity<Family::Entity, Action>,
        WorthQueryApplicationOutputProjectionDenial,
    >
    where
        Family: WorthQueryApplicationOutputRoleFamily<Contract = Contract>,
        Action: WorthQueryApplicationOutputAction,
    {
        let role = OutputRoleUse::member::<Family, Action>(suffix)
            .map_err(WorthQueryApplicationOutputProjectionDenial::InvalidMemberSuffix)?;
        self.correspondence
            .bound_entity(&role)?
            .map(WorthQueryApplicationOutputEntity::new)
            .ok_or(WorthQueryApplicationOutputProjectionDenial::MissingRole)
    }

    /// Every member of `Family` the commit bound, in role-name order, with its
    /// posture.
    pub fn family_entries<Family>(
        &self,
    ) -> Result<
        Vec<WorthQueryApplicationOutputFamilyEntry<'correspondence, Family::Entity>>,
        WorthQueryApplicationOutputProjectionDenial,
    >
    where
        Family: WorthQueryApplicationOutputRoleFamily<Contract = Contract>,
    {
        self.correspondence
            .family_members::<Family>()?
            .map(|entry| {
                entry.map(
                    |(role, posture, entity_id)| WorthQueryApplicationOutputFamilyEntry {
                        role,
                        posture,
                        entity_id,
                        _marker: PhantomData,
                    },
                )
            })
            .collect()
    }
}
