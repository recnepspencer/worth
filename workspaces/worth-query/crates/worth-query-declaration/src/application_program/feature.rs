use std::marker::PhantomData;

use crate::application_schema::{ApplicationSchema, ApplicationStructuredValueBinding};

mod authoring;

pub use authoring::{ApplicationFeatureSpec, ApplicationFeatureSpecBuilder};

/// One semantic feature participating in an application program.
pub trait ApplicationFeature<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Inputs: ApplicationFeatureInputsShape<Schema, Self>;

    const IDENTITY: &'static str;
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

/// An input port owned by one feature.
pub trait ApplicationInputPort<Schema, Feature>: Sized + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    type Value: ApplicationStructuredValueBinding;

    const IDENTITY: &'static str;
    const REQUIRED: bool;
}

/// An output port owned by one feature.
pub trait ApplicationOutputPort<Schema, Feature>: Sized + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    type Value: ApplicationStructuredValueBinding;

    const IDENTITY: &'static str;
}

/// Typed reference to a feature port. It carries no runtime authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationPortRef<Schema, Feature, Port> {
    marker: PhantomData<fn() -> (Schema, Feature, Port)>,
}

impl<Schema, Feature, Port> ApplicationPortRef<Schema, Feature, Port> {
    pub const fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<Schema, Feature, Port> Default for ApplicationPortRef<Schema, Feature, Port> {
    fn default() -> Self {
        Self::new()
    }
}

/// Erased descriptive feature inventory used by installation validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationFeatureDeclaration {
    composition_instance: &'static str,
    identity: &'static str,
    major: u16,
    minor: u16,
    inputs: Box<[ApplicationFeatureInputDeclaration]>,
    outputs: Box<[ApplicationFeatureOutputDeclaration]>,
    derived_artifacts: Box<[super::ApplicationDerivedArtifactDeclaration]>,
}

impl ApplicationFeatureDeclaration {
    pub(super) fn new(
        composition_instance: &'static str,
        identity: &'static str,
        major: u16,
        minor: u16,
        inputs: Vec<ApplicationFeatureInputDeclaration>,
    ) -> Self {
        Self {
            composition_instance,
            identity,
            major,
            minor,
            inputs: inputs.into_boxed_slice(),
            outputs: Box::new([]),
            derived_artifacts: Box::new([]),
        }
    }

    pub const fn composition_instance(&self) -> &'static str {
        self.composition_instance
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn major(&self) -> u16 {
        self.major
    }

    pub const fn minor(&self) -> u16 {
        self.minor
    }

    pub fn inputs(&self) -> &[ApplicationFeatureInputDeclaration] {
        &self.inputs
    }

    pub fn required_inputs(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.inputs
            .iter()
            .filter(|input| input.required())
            .map(ApplicationFeatureInputDeclaration::identity)
    }

    pub fn outputs(&self) -> &[ApplicationFeatureOutputDeclaration] {
        &self.outputs
    }

    pub(super) fn set_outputs(&mut self, outputs: Vec<ApplicationFeatureOutputDeclaration>) {
        self.outputs = outputs.into_boxed_slice();
    }

    pub fn derived_artifacts(&self) -> &[super::ApplicationDerivedArtifactDeclaration] {
        &self.derived_artifacts
    }

    pub(super) fn set_derived_artifacts(
        &mut self,
        artifacts: Vec<super::ApplicationDerivedArtifactDeclaration>,
    ) {
        self.derived_artifacts = artifacts.into_boxed_slice();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationFeatureInputDeclaration {
    identity: &'static str,
    required: bool,
}

impl ApplicationFeatureInputDeclaration {
    const fn new(identity: &'static str, required: bool) -> Self {
        Self { identity, required }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn required(&self) -> bool {
        self.required
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationFeatureOutputDeclaration {
    identity: &'static str,
}

impl ApplicationFeatureOutputDeclaration {
    pub(super) const fn new(identity: &'static str) -> Self {
        Self { identity }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }
}

/// One semantic feature installed at an explicit composition-instance path.
pub(crate) struct ApplicationFeatureInstanceRef<Schema, Instance, Feature> {
    marker: PhantomData<fn() -> (Schema, Instance, Feature)>,
}

/// One input port followed by the remaining ports on the same feature.
pub struct ApplicationFeatureInputList<Port, Tail> {
    marker: PhantomData<fn() -> (Port, Tail)>,
}

pub struct ApplicationFeatureInputLeaf;

mod sealed {
    pub trait FeatureShape {}
    pub trait FeatureInputsShape {}
}

pub(crate) trait ApplicationFeatureShape<Schema>:
    sealed::FeatureShape + Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn declaration() -> ApplicationFeatureDeclaration;
}

pub trait ApplicationFeatureInputsShape<Schema, Feature>:
    sealed::FeatureInputsShape + Sized + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    fn append_inputs(inputs: &mut Vec<ApplicationFeatureInputDeclaration>);
}

impl<Schema, Instance, Feature> sealed::FeatureShape
    for ApplicationFeatureInstanceRef<Schema, Instance, Feature>
{
}

impl<Schema, Instance, Feature> ApplicationFeatureShape<Schema>
    for ApplicationFeatureInstanceRef<Schema, Instance, Feature>
where
    Schema: ApplicationSchema + 'static,
    Instance: super::ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
{
    fn declaration() -> ApplicationFeatureDeclaration {
        let mut inputs = Vec::new();
        Feature::Inputs::append_inputs(&mut inputs);
        ApplicationFeatureDeclaration::new(
            Instance::PATH,
            Feature::IDENTITY,
            Feature::MAJOR,
            Feature::MINOR,
            inputs,
        )
    }
}

impl sealed::FeatureInputsShape for ApplicationFeatureInputLeaf {}

impl<Schema, Feature> ApplicationFeatureInputsShape<Schema, Feature> for ApplicationFeatureInputLeaf
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    fn append_inputs(_: &mut Vec<ApplicationFeatureInputDeclaration>) {}
}

impl<Port, Tail> sealed::FeatureInputsShape for ApplicationFeatureInputList<Port, Tail> {}

impl<Schema, Feature, Port, Tail> ApplicationFeatureInputsShape<Schema, Feature>
    for ApplicationFeatureInputList<Port, Tail>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Port: ApplicationInputPort<Schema, Feature>,
    Tail: ApplicationFeatureInputsShape<Schema, Feature>,
{
    fn append_inputs(inputs: &mut Vec<ApplicationFeatureInputDeclaration>) {
        inputs.push(ApplicationFeatureInputDeclaration::new(
            Port::IDENTITY,
            Port::REQUIRED,
        ));
        Tail::append_inputs(inputs);
    }
}
