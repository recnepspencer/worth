//! Lowering of installed persistent name schemes (entity, relation, and group)
//! from `worth-query-installation` into execution-time contracts and Relational
//! minting registries.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind;
use worth_query_installation::facade::{
    InstalledGroupNameSchemeLayout, InstalledNameSchemeCatalog, InstalledNameSchemeLayout,
};
use worth_relational::facade::aggregate_identity::{
    AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
};
use worth_relational::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
};

use crate::denial::ExecutionNamingDenial;

// A structural-identity discriminator the declaration admits at compile
// time is one Relational admits at commit.
const _: () = assert!(
    worth_query_declaration::facade::application_naming::MAX_STRUCTURAL_DISCRIMINATOR_BYTES
        == worth_relational::facade::lineage::MAX_STRUCTURAL_DISCRIMINATOR_BYTES
);

/// Lowered execution contract for one installed group name scheme.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweredExecutionGroupScheme {
    layout: InstalledGroupNameSchemeLayout,
    relational_scheme: AggregateGroupScheme,
}

impl LoweredExecutionGroupScheme {
    pub fn scheme_id(&self) -> &str {
        self.layout.scheme_id()
    }

    pub fn group_name(&self) -> &str {
        self.layout.group_name()
    }

    pub fn layout(&self) -> &InstalledGroupNameSchemeLayout {
        &self.layout
    }

    pub fn relational_scheme(&self) -> &AggregateGroupScheme {
        &self.relational_scheme
    }

    pub fn relational_member_role(&self, declared_role: &str) -> Option<AggregateMemberRole> {
        self.layout.name_scheme().role_rank(declared_role)?;
        Some(self.layout.member_role(declared_role))
    }
}

/// Immutable lowered execution naming catalog derived from one installed
/// [`InstalledNameSchemeCatalog`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweredExecutionNameSchemeCatalog {
    installed_schemes: BTreeMap<String, InstalledNameSchemeLayout>,
    by_subject: BTreeMap<(ApplicationNameSubjectKind, String), String>,
    relational_registry: InstalledPersistentNameSchemeRegistry,
    group_schemes: BTreeMap<String, LoweredExecutionGroupScheme>,
    group_by_subject: BTreeMap<String, String>,
}

impl LoweredExecutionNameSchemeCatalog {
    /// Lowers every installed entity, relation, and group scheme from
    /// `catalog` into Relational's immutable minting contracts.
    ///
    /// Complexity: O(S * R log R) where S is installed scheme count and R is
    /// roles per scheme.
    pub fn lower_from_installed(
        catalog: &InstalledNameSchemeCatalog,
    ) -> Result<Self, ExecutionNamingDenial> {
        let mut installed_schemes = BTreeMap::new();
        let mut by_subject = BTreeMap::new();
        let mut relational_registry = InstalledPersistentNameSchemeRegistry::new();
        let mut group_schemes = BTreeMap::new();
        let mut group_by_subject = BTreeMap::new();

        for layout in catalog.schemes() {
            let relational_scheme = lower_persistent_name_scheme(layout)?;
            relational_registry
                .register(relational_scheme)
                .map_err(|detail| ExecutionNamingDenial::SchemeLowering {
                    scheme_id: layout.scheme_id().to_owned(),
                    detail,
                })?;
            by_subject.insert(
                (layout.subject_kind(), layout.subject_name().to_owned()),
                layout.scheme_id().to_owned(),
            );
            installed_schemes.insert(layout.scheme_id().to_owned(), layout.clone());
        }

        for group_layout in catalog.group_schemes() {
            let lowered = lower_group_scheme(group_layout)?;
            let scheme_id = lowered.scheme_id().to_owned();
            // Relational records each group's scheme, so a group read is
            // kind-verified exactly like an entity or relation read.
            let group_subject = lower_persistent_name_scheme(group_layout.name_scheme())?;
            let group_name = lowered.group_name().to_owned();
            if installed_schemes.contains_key(&scheme_id) || group_schemes.contains_key(&scheme_id)
            {
                return Err(ExecutionNamingDenial::SchemeLowering {
                    scheme_id,
                    detail: "duplicate group scheme identifier".to_owned(),
                });
            }
            relational_registry
                .register(group_subject)
                .map_err(|detail| ExecutionNamingDenial::SchemeLowering {
                    scheme_id: scheme_id.clone(),
                    detail,
                })?;
            group_by_subject.insert(group_name, scheme_id.clone());
            group_schemes.insert(scheme_id, lowered);
        }

        Ok(Self {
            installed_schemes,
            by_subject,
            relational_registry,
            group_schemes,
            group_by_subject,
        })
    }

    pub fn relational_registry(&self) -> &InstalledPersistentNameSchemeRegistry {
        &self.relational_registry
    }

    pub fn scheme_by_id(&self, scheme_id: &str) -> Option<&InstalledNameSchemeLayout> {
        self.installed_schemes.get(scheme_id)
    }

    pub fn scheme_for_subject(
        &self,
        kind: ApplicationNameSubjectKind,
        subject_name: &str,
    ) -> Option<&InstalledNameSchemeLayout> {
        let id = self.by_subject.get(&(kind, subject_name.to_owned()))?;
        self.installed_schemes.get(id)
    }

    pub fn group_scheme_by_id(&self, scheme_id: &str) -> Option<&LoweredExecutionGroupScheme> {
        self.group_schemes.get(scheme_id)
    }

    pub fn group_scheme_for_subject(
        &self,
        group_name: &str,
    ) -> Option<&LoweredExecutionGroupScheme> {
        let id = self.group_by_subject.get(group_name)?;
        self.group_schemes.get(id)
    }
}

fn lower_persistent_name_scheme(
    layout: &InstalledNameSchemeLayout,
) -> Result<InstalledPersistentNameScheme, ExecutionNamingDenial> {
    InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
        layout.scheme_id(),
        layout.subject_kind(),
        layout.subject_name(),
        layout.role_order(),
        layout.split_capable(),
        layout.require_authored_occurrence(),
        layout.structural_identity(),
    )
    .map_err(|detail| ExecutionNamingDenial::SchemeLowering {
        scheme_id: layout.scheme_id().to_owned(),
        detail,
    })
}

fn lower_group_scheme(
    layout: &InstalledGroupNameSchemeLayout,
) -> Result<LoweredExecutionGroupScheme, ExecutionNamingDenial> {
    let scheme_id = AggregateGroupSchemeId::new(layout.scheme_id()).map_err(|denial| {
        ExecutionNamingDenial::SchemeLowering {
            scheme_id: layout.scheme_id().to_owned(),
            detail: format!("invalid aggregate group scheme id: {denial:?}"),
        }
    })?;
    Ok(LoweredExecutionGroupScheme {
        layout: layout.clone(),
        relational_scheme: AggregateGroupScheme::new(scheme_id, layout.continuity_rule()),
    })
}
