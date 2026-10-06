//! Exact prior-family-member reads through the admitted correspondence index.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputProjectionDenial;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputAction;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    /// Read one live member of the family that `PriorBinding` committed.
    /// The family and action are checked against its output contract at compile
    /// time. The suffix selects a member, never a contract or entity kind.
    pub fn prior_output_member<PriorBinding, Family, Action>(
        &mut self,
        suffix: &str,
    ) -> Result<
        WorthQueryInvariantEntityIdentity<Schema, Family::Entity>,
        WorthQueryPriorOutputDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Family:
            WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = PriorBinding::Output>,
        Family::Entity: OperationReads<Operation>,
        Action: WorthQueryApplicationOutputAction,
    {
        self.prior_output_member_if_present::<PriorBinding, Family, Action>(suffix)?
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::Unavailable,
                    Family::PREFIX,
                )
            })
    }

    /// `None` means this exact binding has no prior correspondence. A present
    /// correspondence without the named member, or with a wrong action, entity,
    /// contract or unavailable identity, is a denial. Reads use the role index
    /// and charge one role lookup without enumerating the family's inventory.
    pub fn prior_output_member_if_present<PriorBinding, Family, Action>(
        &mut self,
        suffix: &str,
    ) -> Result<
        Option<WorthQueryInvariantEntityIdentity<Schema, Family::Entity>>,
        WorthQueryPriorOutputDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Family:
            WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = PriorBinding::Output>,
        Family::Entity: OperationReads<Operation>,
        Action: WorthQueryApplicationOutputAction,
    {
        let role = OutputRoleUse::member::<Family, Action>(suffix).map_err(|denial| {
            WorthQueryPriorOutputDenial::projection(
                Family::PREFIX,
                WorthQueryApplicationOutputProjectionDenial::InvalidMemberSuffix(denial),
            )
        })?;
        self.admit_prior_entity::<Family::Entity>(&role.name)?;
        let Some(correspondence) = self.select_prior_correspondence::<PriorBinding>(&role.name)?
        else {
            return Ok(None);
        };
        self.charge_role_work(&role.name)?;
        let entity = correspondence
            .bound_entity(&role)
            .map_err(|denial| WorthQueryPriorOutputDenial::projection(&role.name, denial))?
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::MissingRole,
                    &role.name,
                )
            })?;
        self.live_prior_identity::<Family::Entity>(&role.name, entity)
            .map(Some)
    }
}
