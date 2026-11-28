//! Public declaration and installed co-anchor retirement use actual typed kinds.
use super::super::{
    correction_courts::install_program,
    program::*,
};
use super::*;
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchemaDenialKind;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type S = Model<44, Present<true>>;
#[test]
fn heterogeneous_non_anchor_kind_refuses_with_its_typed_subject() {
    let error = super::super::correction_courts::try_install_definition::<
        Model<45, Present<false>>,
        Program,
    >()
    .err()
    .expect("Body is not declared as an anchor by this correction");
    let WorthQueryApplicationPreparationDenial::Schema(error) = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::PreImageDemandNotCoveredByAuthoredAnchors
    );
    assert_eq!(error.subject(), "Body");
}

#[test]
fn a_co_anchor_declared_by_a_different_operation_cannot_cover_the_retirement_demand() {
    let error = super::super::correction_courts::try_install_definition::<
        Model<46, Present<true>>,
        Program,
    >()
    .err()
    .expect("Create's co-anchor cannot cover Retire's pre-image");
    let WorthQueryApplicationPreparationDenial::Schema(error) = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::PreImageDemandNotCoveredByAuthoredAnchors
    );
    assert_eq!(error.subject(), "Body");
}
