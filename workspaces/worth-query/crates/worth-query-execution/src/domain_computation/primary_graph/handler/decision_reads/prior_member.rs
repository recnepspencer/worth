use super::*;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputAction;

impl<Schema, Binding> DecisionReader<'_, '_, '_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Resolve one live member of a prior declared family through the admitted
    /// correspondence index, with its declared publication action.
    pub fn prior_output_member<PriorBinding, Family, Action>(
        &mut self,
        suffix: &str,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Family::Entity>, HandlerExecutionDenial>
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Family:
            WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = PriorBinding::Output>,
        Family::Entity: OperationReads<Binding::Operation>,
        Action: WorthQueryApplicationOutputAction,
    {
        self.reader
            .prior_output_member::<PriorBinding, Family, Action>(suffix)
            .map_err(HandlerExecutionDenial::new)
    }

    /// A missing prior binding yields `None`; missing or invalid members in a
    /// present correspondence remain denials and cannot trigger initial fallback.
    pub fn prior_output_member_if_present<PriorBinding, Family, Action>(
        &mut self,
        suffix: &str,
    ) -> Result<
        Option<WorthQueryInvariantEntityIdentity<Schema, Family::Entity>>,
        HandlerExecutionDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Family:
            WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = PriorBinding::Output>,
        Family::Entity: OperationReads<Binding::Operation>,
        Action: WorthQueryApplicationOutputAction,
    {
        self.reader
            .prior_output_member_if_present::<PriorBinding, Family, Action>(suffix)
            .map_err(HandlerExecutionDenial::new)
    }
}
