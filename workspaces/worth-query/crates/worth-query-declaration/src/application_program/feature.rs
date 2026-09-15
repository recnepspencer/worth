use std::marker::PhantomData;

use crate::application_schema::{ApplicationSchema, ApplicationStructuredValueBinding};

/// One semantic feature participating in an application program.
pub trait ApplicationFeature<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    const IDENTITY: &'static str;
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
    identity: &'static str,
    required_inputs: &'static [&'static str],
}

impl ApplicationFeatureDeclaration {
    pub const fn new(identity: &'static str, required_inputs: &'static [&'static str]) -> Self {
        Self {
            identity,
            required_inputs,
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn required_inputs(&self) -> &'static [&'static str] {
        self.required_inputs
    }
}
