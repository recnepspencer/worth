use crate::application_schema::{ApplicationOperationMarkerIdentity, ApplicationSchema};

pub trait ApplicationExternalInputProvider<Schema, Operation>: Sized + 'static
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
{
    const IDENTITY: &'static str;
    type Selection: Clone;
    type Values: Clone;
    type Revision: Clone + Eq;
    type Provenance: Clone;
    type Denial;

    fn resolve(
        &self,
        selection: &Self::Selection,
    ) -> Result<
        ApplicationExternalInputResolution<Self::Values, Self::Revision, Self::Provenance>,
        Self::Denial,
    >;

    fn validate_revision(
        &self,
        selection: &Self::Selection,
        revision: &Self::Revision,
    ) -> Result<(), Self::Denial>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationExternalInputResolution<Values, Revision, Provenance> {
    values: Values,
    revision: Revision,
    provenance: Provenance,
}

impl<Values, Revision, Provenance>
    ApplicationExternalInputResolution<Values, Revision, Provenance>
{
    pub const fn new(values: Values, revision: Revision, provenance: Provenance) -> Self {
        Self {
            values,
            revision,
            provenance,
        }
    }

    pub const fn values(&self) -> &Values {
        &self.values
    }
    pub const fn revision(&self) -> &Revision {
        &self.revision
    }
    pub const fn provenance(&self) -> &Provenance {
        &self.provenance
    }
}
