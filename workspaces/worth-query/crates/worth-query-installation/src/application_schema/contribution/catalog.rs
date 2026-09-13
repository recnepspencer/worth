use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaContributionIdentity, ApplicationSchemaContributionProvenance,
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};

/// Read-only contribution ownership retained by one installed schema.
///
/// This view borrows declaration-owned provenance and grants no installation
/// or execution authority.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryInstalledApplicationContribution<'a> {
    provenance: &'a ApplicationSchemaContributionProvenance,
    members: &'a [ApplicationSchemaMember],
}

impl<'a> WorthQueryInstalledApplicationContribution<'a> {
    pub const fn identity(&self) -> &ApplicationSchemaContributionIdentity {
        self.provenance.identity()
    }

    pub fn member_ordinals(&self) -> &[u32] {
        self.provenance.member_ordinals()
    }

    pub fn members(&self) -> impl ExactSizeIterator<Item = &'a ApplicationSchemaMember> + use<'a> {
        let members = self.members;
        self.provenance
            .member_ordinals()
            .iter()
            .map(move |ordinal| {
                members
                    .get(*ordinal as usize)
                    .expect("installed contribution ordinals were validated during compilation")
            })
    }
}

/// Borrowing catalog over the exact contribution provenance in an installed
/// declaration.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryInstalledApplicationContributionCatalog<'a> {
    declaration: &'a ErasedApplicationSchemaDeclaration,
}

impl<'a> WorthQueryInstalledApplicationContributionCatalog<'a> {
    pub(crate) const fn from_installed_declaration(
        declaration: &'a ErasedApplicationSchemaDeclaration,
    ) -> Self {
        Self { declaration }
    }

    pub fn len(&self) -> usize {
        self.declaration.contributions().len()
    }

    pub fn is_empty(&self) -> bool {
        self.declaration.contributions().is_empty()
    }

    pub fn get(&self, identity: &str) -> Option<WorthQueryInstalledApplicationContribution<'a>> {
        self.declaration
            .contributions()
            .binary_search_by(|candidate| candidate.identity().as_str().cmp(identity))
            .ok()
            .map(|index| self.row(&self.declaration.contributions()[index]))
    }

    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryInstalledApplicationContribution<'a>> + use<'a>
    {
        let members = self.declaration.members();
        self.declaration
            .contributions()
            .iter()
            .map(
                move |provenance| WorthQueryInstalledApplicationContribution {
                    provenance,
                    members,
                },
            )
    }

    fn row(
        &self,
        provenance: &'a ApplicationSchemaContributionProvenance,
    ) -> WorthQueryInstalledApplicationContribution<'a> {
        WorthQueryInstalledApplicationContribution {
            provenance,
            members: self.declaration.members(),
        }
    }
}
