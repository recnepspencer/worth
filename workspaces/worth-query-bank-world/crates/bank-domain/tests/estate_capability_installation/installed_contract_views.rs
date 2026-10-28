//! Typed test views of installed application operation contracts.

use std::collections::BTreeSet;

use worth_query_host::facade::domain::{
    WorthQueryCompiledApplicationOperationContracts, WorthQueryOperationGraphReadScope,
    WorthQueryOperationTouchScope,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum InstalledReadTarget {
    Entity(String),
    Field {
        entity: String,
        aspect: String,
        path: worth_foundational::facade::CanonicalFieldPath,
    },
    WholeAspect {
        entity: String,
        aspect: String,
    },
    DerivedSource(String),
    Components(String),
    Relation {
        relation: String,
        from: String,
        to: String,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum InstalledProgramTarget {
    Create(String),
    Delete(String),
    Write {
        entity: String,
        aspect: String,
        path: worth_foundational::facade::CanonicalFieldPath,
    },
    InitializeRelationField {
        relation: String,
        from: String,
        to: String,
        aspect: String,
        path: worth_foundational::facade::CanonicalFieldPath,
    },
    Link {
        relation: String,
        from: String,
        to: String,
    },
    Unlink {
        relation: String,
        from: String,
        to: String,
    },
    Emit(String),
    DeclaredDomain(String),
}

pub(super) fn installed_read_targets(
    contracts: &WorthQueryCompiledApplicationOperationContracts,
) -> BTreeSet<InstalledReadTarget> {
    contracts
        .graph_reads()
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .flat_map(|scope| match scope {
            WorthQueryOperationGraphReadScope::DerivedSource(scope) => {
                vec![InstalledReadTarget::DerivedSource(
                    scope.relation().to_owned(),
                )]
            }
            WorthQueryOperationGraphReadScope::Components(scope) => {
                vec![InstalledReadTarget::Components(
                    scope.meaning().canonical_record(),
                )]
            }
            WorthQueryOperationGraphReadScope::Entity(scope) => {
                vec![InstalledReadTarget::Entity(scope.semantic_key().to_owned())]
            }
            WorthQueryOperationGraphReadScope::NativeProjection(scope) => {
                if scope.projection().mask().is_whole_aspect() {
                    vec![InstalledReadTarget::WholeAspect {
                        entity: scope.entity().semantic_key().to_owned(),
                        aspect: scope.aspect().as_str().to_owned(),
                    }]
                } else {
                    scope
                        .projection()
                        .mask()
                        .paths()
                        .iter()
                        .cloned()
                        .map(|path| InstalledReadTarget::Field {
                            entity: scope.entity().semantic_key().to_owned(),
                            aspect: scope.aspect().as_str().to_owned(),
                            path,
                        })
                        .collect()
                }
            }
            WorthQueryOperationGraphReadScope::Relation(scope) => {
                vec![InstalledReadTarget::Relation {
                    relation: scope.relation().to_owned(),
                    from: scope.from().to_owned(),
                    to: scope.to().to_owned(),
                }]
            }
        })
        .collect()
}

pub(super) fn installed_program_targets(
    contracts: &WorthQueryCompiledApplicationOperationContracts,
) -> BTreeSet<InstalledProgramTarget> {
    let mut targets = contracts
        .touches()
        .scopes()
        .iter()
        .map(|scope| match scope {
            WorthQueryOperationTouchScope::CreateEntity(scope) => {
                InstalledProgramTarget::Create(scope.entity().to_owned())
            }
            WorthQueryOperationTouchScope::DeleteEntity(scope) => {
                InstalledProgramTarget::Delete(scope.entity().to_owned())
            }
            WorthQueryOperationTouchScope::WriteField(scope) => InstalledProgramTarget::Write {
                entity: scope.entity().to_owned(),
                aspect: scope.contract().key().as_str().to_owned(),
                path: scope.field_path().clone(),
            },
            WorthQueryOperationTouchScope::LinkRelation(scope) => InstalledProgramTarget::Link {
                relation: scope.relation().to_owned(),
                from: scope.from().to_owned(),
                to: scope.to().to_owned(),
            },
            WorthQueryOperationTouchScope::InitializeRelationField(scope) => {
                InstalledProgramTarget::InitializeRelationField {
                    relation: scope.relation().relation().to_owned(),
                    from: scope.relation().from().to_owned(),
                    to: scope.relation().to().to_owned(),
                    aspect: scope.contract().key().as_str().to_owned(),
                    path: scope.field_path().clone(),
                }
            }
            WorthQueryOperationTouchScope::UnlinkRelation(scope) => {
                InstalledProgramTarget::Unlink {
                    relation: scope.relation().to_owned(),
                    from: scope.from().to_owned(),
                    to: scope.to().to_owned(),
                }
            }
            WorthQueryOperationTouchScope::DeclaredDomain(scope) => {
                InstalledProgramTarget::DeclaredDomain(scope.as_str().to_owned())
            }
        })
        .collect::<BTreeSet<_>>();
    targets.extend(
        contracts
            .emissions()
            .emissions()
            .iter()
            .map(|emission| InstalledProgramTarget::Emit(emission.effect().to_owned())),
    );
    targets
}
