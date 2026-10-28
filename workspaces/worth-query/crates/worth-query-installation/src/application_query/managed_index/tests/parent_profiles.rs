//! Each actual schema owns the recipes that its installed provenance must match.
use super::ProbeBinding;
use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledApplicationSchemaDenialKind as SchemaKind, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_declaration::facade::{application_query::*, application_schema::*};
mod model;
mod queries;

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
pub(super) fn check_actual_parent_profiles() {
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
    let foreign = bind::<1>()
        .err()
        .expect("the actual foreign codec profile refuses");
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
        .err()
        .expect("an opaque installed-policy recipe still lacks executable disclosure");
    assert_eq!(disclosure.kind(), SchemaKind::ManagedIndexParentUnsupported);
    assert_eq!(
        disclosure.subject(),
        queries::ModelIndex::<model::ParentSchema<3>>::NAME
    );
}
