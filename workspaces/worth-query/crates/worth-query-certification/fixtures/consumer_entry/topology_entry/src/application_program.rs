use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureInputList, ApplicationInputPort, ApplicationOccurrenceConnectionBinding,
    ApplicationOutputPort,
};
use worth_query_host::facade::{
    application_contribution::WorthQueryApplicationOutputDemand,
    primary_graph::{
        WorthQueryApplicationDependentOutputConnection,
        WorthQueryApplicationDiscoveredOutputConnection,
        WorthQueryApplicationRequiredOutputConnection, WorthQueryRequiredOutputConnectionDenial,
    },
};

use super::{
    PlanarOutputFamily, PlanarRead, PlanarReadResultBinding, PlanarSourceAdjustment,
    PlanarSourceAdjustmentBinding, TopologySchemaBinding,
};

mod connections;

pub struct PlanarSourceFeature;
pub struct PlanarOutputFeature;
pub struct PlanarBodyOutput;
pub struct PlanarBodyInput;
pub struct PlanarSourceToOutputConnection;
pub struct PlanarSourceToRemoteOutputConnection;
pub struct PlanarFinalOutputFeature;
pub struct PlanarAlternateFinalOutputFeature;
pub struct PlanarDerivedBodyOutput;
pub struct PlanarDerivedBodyInput;
pub struct PlanarAlternateDerivedBodyInput;
pub struct PlanarOutputToFinalConnection;
pub struct PlanarOutputToLateFinalConnection;
pub struct PlanarOutputToAlternateFinalConnection;
pub struct PlanarSummaryFeature;
pub struct PlanarAlternateSummaryFeature;
pub struct PlanarFinalBodyOutput;
pub struct PlanarAlternateFinalBodyOutput;
pub struct PlanarSummaryInput;
pub struct PlanarAlternateSummaryInput;
pub struct PlanarFinalToSummaryConnection;
pub struct PlanarAlternateFinalToSummaryConnection;

impl<Schema: TopologySchemaBinding> ApplicationFeature<Schema> for PlanarSourceFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.certification.planar-source-feature.v1";
}

impl<Schema: TopologySchemaBinding> ApplicationFeature<Schema> for PlanarOutputFeature {
    type Inputs = ApplicationFeatureInputList<PlanarBodyInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.planar-output-feature.v1";
}

impl<Schema: TopologySchemaBinding> ApplicationFeature<Schema> for PlanarFinalOutputFeature {
    type Inputs = ApplicationFeatureInputList<PlanarDerivedBodyInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.planar-final-output-feature.v1";
}

impl<Schema: TopologySchemaBinding> ApplicationFeature<Schema>
    for PlanarAlternateFinalOutputFeature
{
    type Inputs =
        ApplicationFeatureInputList<PlanarAlternateDerivedBodyInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str =
        "worth.query.certification.planar-alternate-final-output-feature.v1";
}

impl<Schema: TopologySchemaBinding> ApplicationFeature<Schema> for PlanarSummaryFeature {
    type Inputs = ApplicationFeatureInputList<PlanarSummaryInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.planar-summary-feature.v1";
}

impl<Schema: TopologySchemaBinding> ApplicationFeature<Schema> for PlanarAlternateSummaryFeature {
    type Inputs =
        ApplicationFeatureInputList<PlanarAlternateSummaryInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.planar-alternate-summary-feature.v1";
}

impl<Schema: TopologySchemaBinding> ApplicationOutputPort<Schema, PlanarSourceFeature>
    for PlanarBodyOutput
{
    type Value = PlanarReadResultBinding;

    const IDENTITY: &'static str = "body";
}

impl<Schema: TopologySchemaBinding> ApplicationInputPort<Schema, PlanarOutputFeature>
    for PlanarBodyInput
{
    type Value = PlanarReadResultBinding;

    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl<Schema: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<Schema, PlanarSourceFeature, PlanarOutputFeature>
    for PlanarSourceToOutputConnection
{
}

impl ApplicationConnectionIdentity for PlanarSourceToOutputConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-source-to-output.v1";
}

impl ApplicationConnectionIdentity for PlanarSourceToRemoteOutputConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-source-to-remote-output.v1";
}

impl<Schema: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<Schema, PlanarSourceFeature, PlanarOutputFeature>
    for PlanarSourceToRemoteOutputConnection
{
}

impl<Schema: TopologySchemaBinding> ApplicationOutputPort<Schema, PlanarOutputFeature>
    for PlanarDerivedBodyOutput
{
    type Value = super::PlanarOutputReadResultBinding;

    const IDENTITY: &'static str = "derived-body";
}

impl<Schema: TopologySchemaBinding> ApplicationInputPort<Schema, PlanarFinalOutputFeature>
    for PlanarDerivedBodyInput
{
    type Value = super::PlanarOutputReadResultBinding;

    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl<Schema: TopologySchemaBinding> ApplicationInputPort<Schema, PlanarAlternateFinalOutputFeature>
    for PlanarAlternateDerivedBodyInput
{
    type Value = super::PlanarOutputReadResultBinding;

    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl<Schema: TopologySchemaBinding> ApplicationOutputPort<Schema, PlanarFinalOutputFeature>
    for PlanarFinalBodyOutput
{
    type Value = PlanarReadResultBinding;

    const IDENTITY: &'static str = "final-body";
}

impl<Schema: TopologySchemaBinding> ApplicationOutputPort<Schema, PlanarAlternateFinalOutputFeature>
    for PlanarAlternateFinalBodyOutput
{
    type Value = PlanarReadResultBinding;

    const IDENTITY: &'static str = "final-body";
}

impl<Schema: TopologySchemaBinding> ApplicationInputPort<Schema, PlanarSummaryFeature>
    for PlanarSummaryInput
{
    type Value = PlanarReadResultBinding;

    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl<Schema: TopologySchemaBinding> ApplicationInputPort<Schema, PlanarAlternateSummaryFeature>
    for PlanarAlternateSummaryInput
{
    type Value = PlanarReadResultBinding;

    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl ApplicationConnectionIdentity for PlanarOutputToFinalConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-final.v1";
}

impl ApplicationConnectionIdentity for PlanarOutputToLateFinalConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-late-final.v1";
}

impl<Schema: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<Schema, PlanarOutputFeature, PlanarFinalOutputFeature>
    for PlanarOutputToLateFinalConnection
{
}

impl ApplicationConnectionIdentity for PlanarOutputToAlternateFinalConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-alternate-final.v1";
}

impl<Schema: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<Schema, PlanarOutputFeature, PlanarFinalOutputFeature>
    for PlanarOutputToFinalConnection
{
}

impl<Schema: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<
        Schema,
        PlanarOutputFeature,
        PlanarAlternateFinalOutputFeature,
    > for PlanarOutputToAlternateFinalConnection
{
}

impl ApplicationConnectionIdentity for PlanarFinalToSummaryConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-final-to-summary.v1";
}

impl ApplicationConnectionIdentity for PlanarAlternateFinalToSummaryConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-alternate-final-to-summary.v1";
}

impl<Schema: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<Schema, PlanarFinalOutputFeature, PlanarSummaryFeature>
    for PlanarFinalToSummaryConnection
{
}

impl<Schema: TopologySchemaBinding>
    ApplicationOccurrenceConnectionBinding<
        Schema,
        PlanarAlternateFinalOutputFeature,
        PlanarAlternateSummaryFeature,
    > for PlanarAlternateFinalToSummaryConnection
{
}

#[derive(Clone)]
pub struct PlanarOutputDemand {
    body_key: String,
}

impl PlanarOutputDemand {
    pub fn new(body_key: impl Into<String>) -> Self {
        Self {
            body_key: body_key.into(),
        }
    }

    pub fn body_key(&self) -> &str {
        &self.body_key
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputDemand<Schema>
    for PlanarOutputDemand
{
    type OutputFamily = PlanarOutputFamily;

    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.body_key.clone(),
        }
    }
}
