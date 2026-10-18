//! Typed declaration world reusing the mutation binding owner's schema.
use super::super::*;
use crate::application_program::*;

pub(super) struct Source;
pub(super) struct Target;
pub(super) struct Export;
pub(super) struct Import;
pub(super) struct Connection;
pub(super) struct Delivery;
pub(super) struct Scope;
pub(super) struct Change;
pub(super) struct Correspondence;
pub(super) struct Rule;
pub(super) struct Provider;

impl ApplicationFeature<MutationSchema> for Source {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "test.action.source.v1";
}
impl ApplicationFeature<MutationSchema> for Target {
    type Inputs = ApplicationFeatureInputList<Import, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "test.action.target.v1";
}
impl ApplicationOutputPort<MutationSchema, Source> for Export {
    type Value = RenameInputBinding;
    const IDENTITY: &'static str = "test.action.export.v1";
}
impl ApplicationInputPort<MutationSchema, Target> for Import {
    type Value = RenameInputBinding;
    const IDENTITY: &'static str = "test.action.import.v1";
    const REQUIRED: bool = true;
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "test.action.connection.v1";
}
impl ApplicationOccurrenceConnectionBinding<MutationSchema, Source, Target> for Connection {}

pub(super) type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<MutationSchema, Source, Export, Target, Import, Connection>,
    ApplicationOutputLeaf,
>;

impl ApplicationActionDelivery<MutationSchema, RenameBinding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "test.action.fixed-delivery.v1";
}
impl ApplicationFixedActionDelivery<MutationSchema, RenameBinding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
impl ApplicationLocalityScope for Scope {
    const IDENTITY: &'static str = "test.action.locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}
impl ApplicationChangeShape for Change {
    const IDENTITY: &'static str = "test.action.change.v1";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Replace;
}
impl ApplicationRepeatedOptionalMemberCorrespondence<MutationSchema, RenameBinding>
    for Correspondence
{
    type Row = (u64, Option<u64>);
    type Target = u64;
    type Member = u64;
    const IDENTITY: &'static str = "test.action.correspondence.v1";

    fn target(row: &Self::Row) -> u64 {
        row.0
    }
    fn initial_member(row: &Self::Row) -> Option<u64> {
        row.1
    }
    fn input(target: &u64, _: Option<u64>) -> RenameInput {
        RenameInput {
            account_id: *target,
        }
    }
}
impl ApplicationEvaluatedRequirementRule<MutationSchema, RenameOperation> for Rule {
    const IDENTITY: &'static str = "test.action.rule.v1";
    type Context = bool;
    type Requirement = &'static str;
    type Finding = &'static str;

    fn evaluate(context: &bool) -> ApplicationEvaluatedRequirement<&'static str, &'static str> {
        if *context {
            ApplicationEvaluatedRequirement::required("member", "member-required")
        } else {
            ApplicationEvaluatedRequirement::not_required("member-optional")
        }
    }
}
impl ApplicationExternalInputProvider<MutationSchema, RenameOperation> for Provider {
    const IDENTITY: &'static str = "test.action.provider.v1";
    type Selection = u64;
    type Values = u64;
    type Revision = u64;
    type Provenance = &'static str;
    type Denial = ();

    fn resolve(
        &self,
        selection: &u64,
    ) -> Result<ApplicationExternalInputResolution<u64, u64, &'static str>, ()> {
        Ok(ApplicationExternalInputResolution::new(
            *selection,
            1,
            "owner-test",
        ))
    }
    fn validate_revision(&self, _: &u64, revision: &u64) -> Result<(), ()> {
        if *revision == 1 {
            Ok(())
        } else {
            Err(())
        }
    }
}
