use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{ApplicationFeature, ApplicationOutputPort};

pub trait ApplicationProgramInventoryIdentity: Sized + 'static {
    const IDENTITY: &'static str;
}

pub struct ApplicationProgramOutput<Feature, Port>(PhantomData<fn() -> (Feature, Port)>);

pub trait ApplicationProgramOutputNode<Schema>
where
    Schema: ApplicationSchema,
{
    fn declaration() -> ApplicationProgramOutputDeclaration;
}

impl<Schema, Feature, Port> ApplicationProgramOutputNode<Schema>
    for ApplicationProgramOutput<Feature, Port>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Port: ApplicationOutputPort<Schema, Feature>,
{
    fn declaration() -> ApplicationProgramOutputDeclaration {
        ApplicationProgramOutputDeclaration {
            feature: Feature::IDENTITY,
            feature_type: std::any::TypeId::of::<Feature>(),
            port: Port::IDENTITY,
            port_type: std::any::TypeId::of::<Port>(),
        }
    }
}

pub trait ApplicationProgramOutputInventorySet<Schema>
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationProgramOutputDeclaration>;
}

impl<Schema> ApplicationProgramOutputInventorySet<Schema> for ()
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationProgramOutputDeclaration> {
        Vec::new()
    }
}

macro_rules! impl_output_sets {
    ($($output:ident),+) => {
        impl<Schema, $($output),+> ApplicationProgramOutputInventorySet<Schema>
            for ($($output,)+)
        where
            Schema: ApplicationSchema,
            $($output: ApplicationProgramOutputNode<Schema>,)+
        {
            fn declarations() -> Vec<ApplicationProgramOutputDeclaration> {
                vec![$($output::declaration()),+]
            }
        }
    };
}

impl_output_sets!(A);
impl_output_sets!(A, B);
impl_output_sets!(A, B, C);
impl_output_sets!(A, B, C, D);
impl_output_sets!(A, B, C, D, E);
impl_output_sets!(A, B, C, D, E, F);
impl_output_sets!(A, B, C, D, E, F, G);
impl_output_sets!(A, B, C, D, E, F, G, H);
impl_output_sets!(A, B, C, D, E, F, G, H, I);
impl_output_sets!(A, B, C, D, E, F, G, H, I, J);
impl_output_sets!(A, B, C, D, E, F, G, H, I, J, K);
impl_output_sets!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_output_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_output_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_output_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_output_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

pub struct ApplicationProgramInventory<Inventory, Outputs>(
    PhantomData<fn() -> (Inventory, Outputs)>,
);

pub trait ApplicationProgramInventoryNode<Schema>
where
    Schema: ApplicationSchema,
{
    fn declaration() -> ApplicationProgramInventoryDeclaration;
}

impl<Schema, Inventory, Outputs> ApplicationProgramInventoryNode<Schema>
    for ApplicationProgramInventory<Inventory, Outputs>
where
    Schema: ApplicationSchema,
    Inventory: ApplicationProgramInventoryIdentity,
    Outputs: ApplicationProgramOutputInventorySet<Schema> + 'static,
{
    fn declaration() -> ApplicationProgramInventoryDeclaration {
        ApplicationProgramInventoryDeclaration {
            identity: Inventory::IDENTITY,
            marker_type: std::any::TypeId::of::<Inventory>(),
            node_type: std::any::TypeId::of::<Self>(),
            outputs: Outputs::declarations().into_boxed_slice(),
        }
    }
}

pub trait ApplicationProgramInventorySet<Schema>
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationProgramInventoryDeclaration>;
}

impl<Schema> ApplicationProgramInventorySet<Schema> for ()
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationProgramInventoryDeclaration> {
        Vec::new()
    }
}

macro_rules! impl_inventory_sets {
    ($($inventory:ident),+) => {
        impl<Schema, $($inventory),+> ApplicationProgramInventorySet<Schema>
            for ($($inventory,)+)
        where
            Schema: ApplicationSchema,
            $($inventory: ApplicationProgramInventoryNode<Schema>,)+
        {
            fn declarations() -> Vec<ApplicationProgramInventoryDeclaration> {
                vec![$($inventory::declaration()),+]
            }
        }
    };
}

impl_inventory_sets!(A);
impl_inventory_sets!(A, B);
impl_inventory_sets!(A, B, C);
impl_inventory_sets!(A, B, C, D);
impl_inventory_sets!(A, B, C, D, E);
impl_inventory_sets!(A, B, C, D, E, F);
impl_inventory_sets!(A, B, C, D, E, F, G);
impl_inventory_sets!(A, B, C, D, E, F, G, H);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramOutputDeclaration {
    feature: &'static str,
    feature_type: std::any::TypeId,
    port: &'static str,
    port_type: std::any::TypeId,
}

impl ApplicationProgramOutputDeclaration {
    pub const fn feature(&self) -> &'static str {
        self.feature
    }

    pub const fn port(&self) -> &'static str {
        self.port
    }
    pub const fn feature_type(&self) -> std::any::TypeId {
        self.feature_type
    }
    pub const fn port_type(&self) -> std::any::TypeId {
        self.port_type
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramInventoryDeclaration {
    identity: &'static str,
    marker_type: std::any::TypeId,
    node_type: std::any::TypeId,
    outputs: Box<[ApplicationProgramOutputDeclaration]>,
}

impl ApplicationProgramInventoryDeclaration {
    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub fn outputs(&self) -> &[ApplicationProgramOutputDeclaration] {
        &self.outputs
    }
    pub const fn marker_type(&self) -> std::any::TypeId {
        self.marker_type
    }
    pub const fn node_type(&self) -> std::any::TypeId {
        self.node_type
    }
}
