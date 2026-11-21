//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn unlink_observed_for_test<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationReads<Operation> + OperationUnlinks<Operation>,
    {
        Self::unlink_observed::<Relation, From, To>(self, relation, from, to)
    }
}
