//! Persistent-name reads on an installed invariant's read view.
//!
//! The view's posture decides what is recorded. `committed()` reads the
//! branch's names at the proposal's basis and never sees the proposal's own
//! births. `proposed()` also reads the names the commit under evaluation
//! mints, keeps and ends, as its own naming authority validated and minted
//! them: a pending birth answers with the identity and canonical parts
//! finalization records, and a record whose name the commit ends refuses.
//! When the commit's naming plan is refused, the read refuses
//! [`WorthQueryInvariantNameDenial::Names`] and the commit refuses with that
//! exact naming denial, whatever the rule decides.

use worth_query_declaration::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_execution_naming::facade::{
    invariant_entity_birth_of, invariant_entity_name_of, ExecutionNameReadDenial, InvariantNameRead,
};
use worth_relational::facade::runtime::CustomInvariantNameReadDenial;

use super::{
    WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantReadView,
    WorthQueryInvariantAccessDenial,
};

/// Why an invariant's persistent-name read refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantNameDenial {
    /// The view's own controls refused the entity: another binding or view,
    /// another kind, outside the prepared scope, or not readable here.
    Access(WorthQueryInvariantAccessDenial),
    /// The name read refused: no name, an ended name, an unrecorded name, or
    /// a name another kind's scheme governs.
    Read(ExecutionNameReadDenial),
    /// The commit's names are unreadable: none supplied, the index is
    /// unavailable, the work budget is spent, or the naming plan is refused.
    Names(CustomInvariantNameReadDenial),
}

impl<Schema> WorthQueryApplicationInvariantReadView<'_, Schema> {
    /// The admitted `WorthQueryName<Entity>` `entity` carries in this view.
    ///
    /// The entity must be readable here: this view's binding and posture,
    /// its declared kind, and inside the prepared scope. A record that
    /// carries no name refuses `RecordNotNamed`, typed, as an unnamed kind's
    /// records always do.
    ///
    /// Complexity: one entity read plus O(1) expected name probes.
    pub fn name_of<Entity>(
        &self,
        entity: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
    ) -> Result<WorthQueryName<Entity>, WorthQueryInvariantNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        self.check_entity(entity, entity.kind)
            .map_err(WorthQueryInvariantNameDenial::Access)?;
        settle(invariant_entity_name_of::<Schema, Entity>(
            &self.names,
            entity.entity_id,
        ))
    }

    /// The recorded birth of a held entity name in this view: decode it with
    /// `RecordedNameBirth::decode_discriminator::<D>()`. Its parents and the
    /// names its discriminator references are admitted in this same view.
    ///
    /// Complexity: O(1) expected probes per parent and referenced name, plus
    /// O(B) for the B discriminator bytes.
    pub fn birth_of<Entity>(
        &self,
        name: &WorthQueryName<Entity>,
    ) -> Result<RecordedNameBirth<Entity>, WorthQueryInvariantNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        settle(invariant_entity_birth_of::<Schema, Entity>(
            &self.names,
            name,
        ))
    }
}

fn settle<T>(read: InvariantNameRead<T>) -> Result<T, WorthQueryInvariantNameDenial> {
    read.map_err(WorthQueryInvariantNameDenial::Names)?
        .map_err(WorthQueryInvariantNameDenial::Read)
}
