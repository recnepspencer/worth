//! Lend a live staging footprint and the schema selected for this scope.
use super::BranchBoundRelationalTransaction;
use crate::authority::commit::phases::prepare::{
    retained_entity_selection, working_state_inputs::SelectionTransaction,
};

impl<'a> From<&'a mut BranchBoundRelationalTransaction>
    for retained_entity_selection::SelectionInputs<'a>
{
    fn from(transaction: &'a mut BranchBoundRelationalTransaction) -> Self {
        Self(SelectionTransaction {
            transaction_id: transaction.transaction_id,
            schema: transaction.schema_authority.clone(),
            footprint: &mut transaction.footprint,
        })
    }
}
