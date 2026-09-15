use std::marker::PhantomData;

use crate::application_schema::{ApplicationSchema, ApplicationStructuredValueBinding};

pub trait ApplicationFeature<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    const IDENTITY: &'static str;
}

pub trait ApplicationInputPort<Schema, Feature>: Sized + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    type Value: ApplicationStructuredValueBinding;
    const IDENTITY: &'static str;
    const REQUIRED: bool;
}

pub trait ApplicationOutputPort<Schema, Feature>: Sized + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    type Value: ApplicationStructuredValueBinding;
    const IDENTITY: &'static str;
    const REQUIRED: bool = false;
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationFeaturePosture {
    Available,
    Unavailable,
}

pub trait ApplicationFeaturePostureMarker: Sized + 'static {
    const POSTURE: ApplicationFeaturePosture;
}

pub struct ApplicationFeatureAvailable;
pub struct ApplicationFeatureUnavailable;

impl ApplicationFeaturePostureMarker for ApplicationFeatureAvailable {
    const POSTURE: ApplicationFeaturePosture = ApplicationFeaturePosture::Available;
}

impl ApplicationFeaturePostureMarker for ApplicationFeatureUnavailable {
    const POSTURE: ApplicationFeaturePosture = ApplicationFeaturePosture::Unavailable;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationPortDeclaration {
    identity: &'static str,
    type_id: std::any::TypeId,
    required: bool,
}

impl ApplicationPortDeclaration {
    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn required(&self) -> bool {
        self.required
    }

    pub const fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }
}

pub trait ApplicationProgramInputSet<Schema, Feature>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    fn declarations() -> Vec<ApplicationPortDeclaration>;
}

pub trait ApplicationProgramOutputSet<Schema, Feature>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    fn declarations() -> Vec<ApplicationPortDeclaration>;
}

impl<Schema, Feature> ApplicationProgramInputSet<Schema, Feature> for ()
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    fn declarations() -> Vec<ApplicationPortDeclaration> {
        Vec::new()
    }
}

impl<Schema, Feature> ApplicationProgramOutputSet<Schema, Feature> for ()
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    fn declarations() -> Vec<ApplicationPortDeclaration> {
        Vec::new()
    }
}

macro_rules! impl_port_sets {
    ($($port:ident),+) => {
        impl<Schema, Feature, $($port),+> ApplicationProgramInputSet<Schema, Feature>
            for ($($port,)+)
        where
            Schema: ApplicationSchema,
            Feature: ApplicationFeature<Schema>,
            $($port: ApplicationInputPort<Schema, Feature>,)+
        {
            fn declarations() -> Vec<ApplicationPortDeclaration> {
                vec![$(ApplicationPortDeclaration {
                    identity: $port::IDENTITY,
                    type_id: std::any::TypeId::of::<$port>(),
                    required: $port::REQUIRED,
                }),+]
            }
        }

        impl<Schema, Feature, $($port),+> ApplicationProgramOutputSet<Schema, Feature>
            for ($($port,)+)
        where
            Schema: ApplicationSchema,
            Feature: ApplicationFeature<Schema>,
            $($port: ApplicationOutputPort<Schema, Feature>,)+
        {
            fn declarations() -> Vec<ApplicationPortDeclaration> {
                vec![$(ApplicationPortDeclaration {
                    identity: $port::IDENTITY,
                    type_id: std::any::TypeId::of::<$port>(),
                    required: $port::REQUIRED,
                }),+]
            }
        }
    };
}

impl_port_sets!(A);
impl_port_sets!(A, B);
impl_port_sets!(A, B, C);
impl_port_sets!(A, B, C, D);
impl_port_sets!(A, B, C, D, E);
impl_port_sets!(A, B, C, D, E, F);
impl_port_sets!(A, B, C, D, E, F, G);
impl_port_sets!(A, B, C, D, E, F, G, H);

pub struct ApplicationProgramFeature<Feature, Inputs, Outputs, Posture> {
    marker: PhantomData<fn() -> (Feature, Inputs, Outputs, Posture)>,
}

pub trait ApplicationProgramFeatureNode<Schema>
where
    Schema: ApplicationSchema,
{
    fn declaration() -> ApplicationFeatureDeclaration;
}

impl<Schema, Feature, Inputs, Outputs, Posture> ApplicationProgramFeatureNode<Schema>
    for ApplicationProgramFeature<Feature, Inputs, Outputs, Posture>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Inputs: ApplicationProgramInputSet<Schema, Feature>,
    Outputs: ApplicationProgramOutputSet<Schema, Feature>,
    Posture: ApplicationFeaturePostureMarker,
{
    fn declaration() -> ApplicationFeatureDeclaration {
        ApplicationFeatureDeclaration {
            identity: Feature::IDENTITY,
            type_id: std::any::TypeId::of::<Feature>(),
            inputs: Inputs::declarations().into_boxed_slice(),
            outputs: Outputs::declarations().into_boxed_slice(),
            posture: Posture::POSTURE,
        }
    }
}

pub trait ApplicationProgramFeatureSet<Schema>
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationFeatureDeclaration>;
}

impl<Schema> ApplicationProgramFeatureSet<Schema> for ()
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationFeatureDeclaration> {
        Vec::new()
    }
}

macro_rules! impl_feature_sets {
    ($($feature:ident),+) => {
        impl<Schema, $($feature),+> ApplicationProgramFeatureSet<Schema> for ($($feature,)+)
        where
            Schema: ApplicationSchema,
            $($feature: ApplicationProgramFeatureNode<Schema>,)+
        {
            fn declarations() -> Vec<ApplicationFeatureDeclaration> {
                vec![$($feature::declaration()),+]
            }
        }
    };
}

impl_feature_sets!(A);
impl_feature_sets!(A, B);
impl_feature_sets!(A, B, C);
impl_feature_sets!(A, B, C, D);
impl_feature_sets!(A, B, C, D, E);
impl_feature_sets!(A, B, C, D, E, F);
impl_feature_sets!(A, B, C, D, E, F, G);
impl_feature_sets!(A, B, C, D, E, F, G, H);
impl_feature_sets!(A, B, C, D, E, F, G, H, I);
impl_feature_sets!(A, B, C, D, E, F, G, H, I, J);
impl_feature_sets!(A, B, C, D, E, F, G, H, I, J, K);
impl_feature_sets!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_feature_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_feature_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_feature_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_feature_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationFeatureDeclaration {
    identity: &'static str,
    type_id: std::any::TypeId,
    inputs: Box<[ApplicationPortDeclaration]>,
    outputs: Box<[ApplicationPortDeclaration]>,
    posture: ApplicationFeaturePosture,
}

impl ApplicationFeatureDeclaration {
    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    pub fn inputs(&self) -> &[ApplicationPortDeclaration] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[ApplicationPortDeclaration] {
        &self.outputs
    }

    pub const fn posture(&self) -> ApplicationFeaturePosture {
        self.posture
    }
}
