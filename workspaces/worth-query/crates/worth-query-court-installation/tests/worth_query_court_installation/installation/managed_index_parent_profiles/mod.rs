//! Each actual schema owns the recipes that its installed provenance must match.
use worth_query_declaration::facade::{application_query::*, application_schema::*};
use worth_query_declaration::{worth_query_structured_value_binding, worth_query_value_binding};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledApplicationSchemaDenialKind as SchemaKind, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
mod decision_read;
mod model;
mod queries;

#[derive(worth_foundational::facade::WorthValue)]
pub struct Probe {
    prefix: String,
}
worth_query_value_binding!(pub ProbeBinding for Probe {
    identity: "worth.query.tests.index-probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});

pub struct PrincipalQueryResult;
worth_query_structured_value_binding!(pub PrincipalQueryResultBinding for PrincipalQueryResult {
    identity: "worth.query.tests.principal-query-result.v1"
});

#[derive(worth_foundational::facade::WorthValue)]
pub struct OtherProbe {
    minimum: u64,
}
worth_query_declaration::worth_query_value_binding!(pub OtherProbeBinding for OtherProbe {
    identity: "worth.query.tests.index-probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});

fn bind<const MODE: u8>() -> Result<
    WorthQueryInstalledApplicationSchema<model::ParentSchema<MODE>>,
    WorthQueryInstalledApplicationSchemaDenial,
> {
    let declaration = model::ParentSchema::<MODE>::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        model::ParentSchema::<MODE>::OWNER,
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
}
#[test]
fn installed_parent_profiles_preserve_identity_and_refuse_codec_or_disclosure_drift() {
    type Lawful = model::ParentSchema<0>;
    let first = queries::ModelIndex::<Lawful>::descriptor();
    let other = queries::OtherIndex::<Lawful>::descriptor();
    let a = first.meaning().parts().probe.parts();
    let b = other.meaning().parts().probe.parts();
    assert_eq!(
        (&a.binding, a.scalar_family, &a.unit, &a.frame),
        (&b.binding, b.scalar_family, &b.unit, &b.frame)
    );
    assert_ne!(a.canonical, b.canonical);
    let lawful = bind::<0>().unwrap();
    let declared = lawful
        .installed_mutation_binding::<decision_read::PrincipalDecision<Lawful>>()
        .unwrap();
    assert_eq!(
        declared.operation().contracts().managed_index_reads().len(),
        1
    );
    let foreign_scope =
        bind::<4>().expect_err("an input-field operation cannot borrow a principal-scoped Parent");
    assert_eq!(
        foreign_scope.kind(),
        SchemaKind::OperationInstallationDenied
    );
    assert_eq!(
        foreign_scope.subject(),
        "worth.query.tests.managed-parent-field-decision.v1"
    );
    let installed = lawful
        .installed_managed_index::<queries::ModelIndex<Lawful>>()
        .unwrap();
    let reissued = lawful
        .installed_managed_index::<queries::ModelIndex<Lawful>>()
        .unwrap();
    assert!(installed.same_installation(&reissued));
    assert!(installed.is_installed_in(&lawful));
    assert!(installed.admits_parent(
        &lawful
            .installed_query_binding::<queries::ParentBinding<Lawful>>()
            .unwrap()
    ));
    let foreign = bind::<1>().expect_err("the actual foreign codec profile refuses");
    assert_eq!(
        foreign.kind(),
        SchemaKind::ManagedIndexBindingMeaningChanged
    );
    assert_eq!(
        foreign.subject(),
        queries::ModelIndex::<model::ParentSchema<1>>::NAME
    );
    type Protected = model::ParentSchema<2>;
    let protected = bind::<2>().expect("the genuinely declared READ ability can bind");
    assert!(protected
        .installed_managed_index::<queries::ModelIndex<Protected>>()
        .unwrap()
        .admits_parent(
            &protected
                .installed_query_binding::<queries::ParentBinding<Protected>>()
                .unwrap()
        ));
    let disclosure = bind::<3>()
        .expect_err("an opaque installed-policy recipe still lacks executable disclosure");
    assert_eq!(disclosure.kind(), SchemaKind::ManagedIndexParentUnsupported);
    assert_eq!(
        disclosure.subject(),
        queries::ModelIndex::<model::ParentSchema<3>>::NAME
    );
}

#[test]
fn installed_source_structure_changes_canonical_meaning_and_keeps_its_typed_profile() {
    type Opaque = model::ParentSchema<0>;
    type Tree = model::ParentSchema<5>;
    let opaque = Opaque::declaration().unwrap();
    let tree = Tree::declaration().unwrap();
    let normalize = |declaration: &ApplicationSchemaDeclaration<Tree>| {
        declaration
            .erased()
            .members()
            .iter()
            .cloned()
            .map(|member| {
                if let ApplicationSchemaMember::ManagedIndex { definition } = member {
                    let mut parts = definition.parts().clone();
                    parts.structure = ApplicationManagedIndexSourceStructure::Opaque;
                    ApplicationSchemaMember::ManagedIndex {
                        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                            parts,
                        ),
                    }
                } else {
                    member
                }
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        opaque.erased().members(),
        normalize(&tree),
        "only Source structure differs"
    );
    assert_ne!(
        opaque.identity(),
        tree.identity(),
        "canonical identity binds that structural meaning"
    );
    let installed = bind::<5>().unwrap();
    let index = installed
        .installed_managed_index::<queries::ModelIndex<Tree>>()
        .unwrap();
    assert_eq!(
        index.definition().parts().structure,
        ApplicationManagedIndexSourceStructure::BinarySourceTree
    );
    assert!(index.admits_parent(
        &installed
            .installed_query_binding::<queries::ParentBinding<Tree>>()
            .unwrap()
    ));
}

#[test]
fn installed_exhaustive_membership_binds_its_profile_without_changing_parent_meaning() {
    type Candidates = model::ParentSchema<5>;
    type Complete = model::ParentSchema<6>;
    let candidates = Candidates::declaration().unwrap();
    let complete = Complete::declaration().unwrap();
    let normalized = complete
        .erased()
        .members()
        .iter()
        .cloned()
        .map(|member| {
            if let ApplicationSchemaMember::ManagedIndex { definition } = member {
                let mut parts = definition.parts().clone();
                parts.membership = ApplicationManagedIndexMembership::Candidates;
                ApplicationSchemaMember::ManagedIndex {
                    definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                        parts,
                    ),
                }
            } else {
                member
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidates.erased().members(),
        normalized,
        "membership is the only changed meaning"
    );
    assert_ne!(candidates.identity(), complete.identity());
    let installed = bind::<6>().unwrap();
    let index = installed
        .installed_managed_index::<queries::ModelIndex<Complete>>()
        .unwrap();
    assert_eq!(
        index.definition().parts().membership,
        ApplicationManagedIndexMembership::ExhaustiveSourceTree
    );
    assert!(index.admits_parent(
        &installed
            .installed_query_binding::<queries::ParentBinding<Complete>>()
            .unwrap()
    ));
    type Checked = model::ParentSchema<7>;
    let checked = Checked::declaration().unwrap();
    let normalized = checked
        .erased()
        .members()
        .iter()
        .cloned()
        .map(|member| {
            if let ApplicationSchemaMember::ManagedIndex { definition } = member {
                let mut parts = definition.parts().clone();
                parts.membership = ApplicationManagedIndexMembership::ExhaustiveSourceTree;
                ApplicationSchemaMember::ManagedIndex {
                    definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                        parts,
                    ),
                }
            } else {
                member
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(complete.erased().members(), normalized);
    assert_ne!(complete.identity(), checked.identity());
    let installed = bind::<7>().unwrap();
    let index = installed
        .installed_managed_index::<queries::ModelIndex<Checked>>()
        .unwrap();
    assert_eq!(
        index.definition().parts().membership,
        ApplicationManagedIndexMembership::CheckedSummarySourceTree
    );
    assert!(index.admits_parent(
        &installed
            .installed_query_binding::<queries::ParentBinding<Checked>>()
            .unwrap()
    ));
}
