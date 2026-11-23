//! Bind publication services to the installed runtime or its preparation snapshot.
use super::RelationalRuntime;
use crate::mvcc::publication::authority::{
    RelationalPreparationPublicationAuthority, RelationalPublicationAuthority,
};
use crate::publication::{PublicationAuthority, PublicationPreparationAuthority};

impl RelationalRuntime {
    pub fn publication_companion_port(&self) -> crate::mvcc::PublicationCompanionRegistrationPort {
        crate::mvcc::PublicationCompanionRegistrationPort::new(
            self.runtime_instance_id(),
            self.owner_binding(),
            self.publication_binding(),
        )
    }
}

impl RelationalRuntime {
    pub(crate) fn publication_authority(&self) -> PublicationAuthority<'_> {
        PublicationAuthority::new(self)
    }
}

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn publication_preparation_authority(&self) -> PublicationPreparationAuthority<'_> {
        PublicationPreparationAuthority::new(self)
    }
}

impl RelationalRuntime {
    pub(crate) fn mvcc_publication_authority(&self) -> RelationalPublicationAuthority<'_> {
        RelationalPublicationAuthority { runtime: self }
    }
}

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn mvcc_publication_authority(
        &self,
    ) -> RelationalPreparationPublicationAuthority<'_> {
        RelationalPreparationPublicationAuthority { runtime: self }
    }
}
