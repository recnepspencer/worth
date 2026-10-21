//! Readiness follows the primary member's actual admitted Value contract.
use super::*;
use worth_query_decl::facade::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::{canonicalize_request, CanonicalQueryBundle},
};

pub(super) fn meaning<Schema: BirthSchema>() -> (
    domain::WorthQueryOperationNativeProjectionContract,
    domain::WorthQuerySemanticTruthDependency,
    CanonicalQueryBundle,
) {
    // This admission supplies descriptive meaning only. The real runtime's
    // bind_output_readiness still owns the exact installed producer authority.
    let package = domain::WorthQueryPortableDomainPackage::new(
        domain::WorthQueryPortableDomainIdentity::new(Schema::OWNER, Schema::MAJOR, Schema::MINOR),
    )
    .application_schema(Schema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted =
        domain::WorthQueryInstallationAdmissionProfile::new("optional-readiness", "configuration")
            .admit(package)
            .unwrap();
    let installed = domain::WorthQueryInstalledPackageIndex::build(
        domain::WorthQueryInstallationRuntimeIdentity::fresh(),
        domain::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(Schema::declaration().unwrap())
    .unwrap();
    let field = Value::reference::<Schema>();
    let native = installed
        .native_contracts()
        .aspect(field.entity(), field.aspect())
        .unwrap();
    let mask =
        domain::AspectMask::<domain::ProjectionMask>::new([domain::CanonicalFieldPath::single(
            domain::FieldKey::new(field.field()).unwrap(),
        )]);
    let projection = domain::WorthQueryOperationNativeProjectionContract::new(
        native.contract().clone(),
        mask.clone(),
    )
    .unwrap();
    let dependency = domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        native.contract().clone(),
        mask,
        native.binding().clone(),
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap();
    let query = DetailQueryBuilder::new(RootEntityKey::new(field.entity()).unwrap())
        .project(AspectFieldSelector::new(field.aspect(), field.field()).unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new(field.aspect(), field.field(), "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let canonical = canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap();
    (projection, dependency, canonical)
}
