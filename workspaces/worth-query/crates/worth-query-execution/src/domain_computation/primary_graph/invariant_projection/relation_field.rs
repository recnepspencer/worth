//! Privileged pinned relation payload reads. This door is not forwarded to operation readers.
use super::{WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantRelation};
use worth_query_declaration::facade::application_schema::fragment::RelationSelection;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Read one actual field under bootstrap-minted broad invariant authority.
    /// This supplies neither ordinary query disclosure nor an operation's
    /// recorded relation-field decision read. Those readers expose no forwarding door.
    pub fn relation_field<Relation, From, To, Field>(
        &mut self,
        identity: &WorthQueryInvariantRelation<Schema, Relation, From, To>,
        field: Field,
    ) -> Option<Field::Value>
    where
        Field: RelationSelection<Entity = Relation, From = From, To = To>,
    {
        let mut decoded = super::super::value_decode::ScalarFieldDecoder::<Field::Binding>::new();
        self.native_read().relation_field(
            identity.relation_id(),
            identity.from().native_identity(),
            identity.to().native_identity(),
            field.identity(),
            (field.from(), field.to()),
            &mut decoded,
        )?;
        decoded.value
    }
}
