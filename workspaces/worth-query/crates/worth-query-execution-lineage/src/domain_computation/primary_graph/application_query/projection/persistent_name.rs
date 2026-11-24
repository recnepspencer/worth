//! A row's persistent name, projected from the names its read observed.
//!
//! A name is not a declared result slot: it is the identity the row's own
//! record carries, read in the same pass that materialized the row. A
//! record's name changes only when its lineage ends, which ends the record,
//! so the row's own source footprint already tracks every projected name.

use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_execution_naming::facade::ExecutionNameReadDenial;

use super::{
    projection_denial, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionDenialKind as Kind, WorthQueryApplicationProjectionRow,
};

impl<Schema, Query> WorthQueryApplicationProjectionRow<'_, Schema, Query> {
    /// The admitted `WorthQueryName<Entity>` this row's record carries.
    ///
    /// Refuses `PersistentNameMissing` when the record carries no name,
    /// `PersistentNameKindMismatch` when its name belongs to another kind
    /// than `Entity`, and `PersistentNamesUnavailable` when the read did not
    /// observe names (see
    /// `WorthQueryApplicationProjection::PROJECTS_PERSISTENT_NAMES`).
    ///
    /// Complexity: O(log observed records); no further read.
    pub fn name<Entity>(
        &self,
    ) -> Result<WorthQueryName<Entity>, WorthQueryApplicationProjectionDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        self.optional_name::<Entity>()?
            .ok_or_else(|| self.name_denial(Kind::PersistentNameMissing))
    }

    /// The admitted name this row's record carries, or `None` when the record
    /// carries no name. Every other refusal of [`Self::name`] still refuses.
    ///
    /// Complexity: O(log observed records); no further read.
    pub fn optional_name<Entity>(
        &self,
    ) -> Result<Option<WorthQueryName<Entity>>, WorthQueryApplicationProjectionDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let names = self
            .names
            .ok_or_else(|| self.name_denial(Kind::PersistentNamesUnavailable))?;
        names
            .name_of::<Schema, Entity>(self.entity_id())
            .map_err(|denial| {
                self.name_denial(match denial {
                    ExecutionNameReadDenial::KindIdentityMismatch { .. }
                    | ExecutionNameReadDenial::SubjectKindMismatch { .. } => {
                        Kind::PersistentNameKindMismatch
                    }
                    ExecutionNameReadDenial::RecordNoLongerNamed { .. } => {
                        Kind::PersistentNameMissing
                    }
                    _ => Kind::PersistentNamesUnavailable,
                })
            })
    }

    fn name_denial(&self, kind: Kind) -> WorthQueryApplicationProjectionDenial {
        projection_denial(kind, "persistent-name")
    }
}
