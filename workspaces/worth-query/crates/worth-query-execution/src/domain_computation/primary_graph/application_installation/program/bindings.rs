use std::any::TypeId;

use worth_query_declaration::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationConnectionRef, ApplicationFeature,
    ApplicationInputPort, ApplicationOccurrenceConnectionBinding, ApplicationOutputPort,
    ApplicationProgramDependentConnection, ApplicationProgramRequiredConnection,
    ApplicationProgramUnavailableConnection,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    WorthQueryApplicationProgramInstallationDenial,
    WorthQueryApplicationProgramInstallationDenialKind,
};

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationRequiredOutputConnection,
};

pub struct WorthQueryProgramExecutionBindings {
    connections: Vec<WorthQueryProgramExecutionConnectionBinding>,
    required_sources: Vec<TypeId>,
}

pub(super) struct WorthQueryProgramExecutionConnectionBinding {
    pub(super) identity: &'static str,
    pub(super) node_type: TypeId,
    pub(super) binding_type: TypeId,
    pub(super) source_feature_type: TypeId,
    pub(super) source_port_type: TypeId,
    pub(super) target_feature_type: TypeId,
    pub(super) target_port_type: TypeId,
    pub(super) root_demand_type: Option<TypeId>,
    pub(super) target_demand_type: Option<TypeId>,
}

impl WorthQueryProgramExecutionBindings {
    fn new() -> Self {
        Self {
            connections: Vec::new(),
            required_sources: Vec::new(),
        }
    }

    pub(super) fn connections(&self) -> &[WorthQueryProgramExecutionConnectionBinding] {
        &self.connections
    }

    pub(super) fn required_sources(self) -> impl Iterator<Item = TypeId> {
        self.required_sources.into_iter()
    }
}

pub(super) fn require_matching_root_demands(
    bindings: &WorthQueryProgramExecutionBindings,
) -> Result<(), WorthQueryApplicationProgramInstallationDenial> {
    for dependent in bindings
        .connections()
        .iter()
        .filter(|binding| binding.root_demand_type.is_some())
    {
        let parent_demand = bindings.connections().iter().find_map(|candidate| {
            (candidate.target_feature_type == dependent.source_feature_type)
                .then_some(candidate.target_demand_type)
                .flatten()
        });
        if parent_demand != dependent.root_demand_type {
            return Err(WorthQueryApplicationProgramInstallationDenial::new(
                WorthQueryApplicationProgramInstallationDenialKind::RootDemandMismatch,
                dependent.identity,
            ));
        }
    }
    Ok(())
}

fn require_connection_identity(
    declared: &'static str,
    executable: &'static str,
) -> Result<&'static str, WorthQueryApplicationProgramInstallationDenial> {
    if declared != executable {
        return Err(WorthQueryApplicationProgramInstallationDenial::new(
            WorthQueryApplicationProgramInstallationDenialKind::ConnectionIdentityMismatch,
            declared,
        ));
    }
    Ok(declared)
}

pub trait WorthQueryProgramConnectionBinding<Schema>
where
    Schema: ApplicationSchema,
{
    fn append(
        bindings: &mut WorthQueryProgramExecutionBindings,
    ) -> Result<(), WorthQueryApplicationProgramInstallationDenial>;
}

type Connection<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding> =
    ApplicationConnectionRef<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>;

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
    WorthQueryProgramConnectionBinding<Schema>
    for ApplicationProgramRequiredConnection<
        Connection<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>,
    >
where
    Schema: ApplicationSchema,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
    TargetPort: ApplicationInputPort<Schema, TargetFeature, Value = SourcePort::Value>,
    Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>
        + WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    fn append(
        bindings: &mut WorthQueryProgramExecutionBindings,
    ) -> Result<(), WorthQueryApplicationProgramInstallationDenial> {
        let declared = <Binding as ApplicationConnectionIdentity>::IDENTITY;
        let executable =
            <Binding as WorthQueryApplicationRequiredOutputConnection<Schema>>::IDENTITY;
        let declared = require_connection_identity(declared, executable)?;
        bindings
            .connections
            .push(WorthQueryProgramExecutionConnectionBinding {
                identity: declared,
                node_type: TypeId::of::<Self>(),
                binding_type: TypeId::of::<Binding>(),
                source_feature_type: TypeId::of::<SourceFeature>(),
                source_port_type: TypeId::of::<SourcePort>(),
                target_feature_type: TypeId::of::<TargetFeature>(),
                target_port_type: TypeId::of::<TargetPort>(),
                root_demand_type: None,
                target_demand_type: Some(TypeId::of::<Binding::Demand>()),
            });
        bindings
            .required_sources
            .push(TypeId::of::<Binding::Source>());
        Ok(())
    }
}

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
    WorthQueryProgramConnectionBinding<Schema>
    for ApplicationProgramDependentConnection<
        Connection<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>,
    >
where
    Schema: ApplicationSchema,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
    TargetPort: ApplicationInputPort<Schema, TargetFeature, Value = SourcePort::Value>,
    Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>
        + WorthQueryApplicationDependentOutputConnection<Schema>,
{
    fn append(
        bindings: &mut WorthQueryProgramExecutionBindings,
    ) -> Result<(), WorthQueryApplicationProgramInstallationDenial> {
        let declared = <Binding as ApplicationConnectionIdentity>::IDENTITY;
        let executable =
            <Binding as WorthQueryApplicationDependentOutputConnection<Schema>>::IDENTITY;
        let declared = require_connection_identity(declared, executable)?;
        bindings
            .connections
            .push(WorthQueryProgramExecutionConnectionBinding {
                identity: declared,
                node_type: TypeId::of::<Self>(),
                binding_type: TypeId::of::<Binding>(),
                source_feature_type: TypeId::of::<SourceFeature>(),
                source_port_type: TypeId::of::<SourcePort>(),
                target_feature_type: TypeId::of::<TargetFeature>(),
                target_port_type: TypeId::of::<TargetPort>(),
                root_demand_type: Some(TypeId::of::<Binding::RootDemand>()),
                target_demand_type: Some(TypeId::of::<Binding::Demand>()),
            });
        Ok(())
    }
}

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
    WorthQueryProgramConnectionBinding<Schema>
    for ApplicationProgramUnavailableConnection<
        Connection<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>,
    >
where
    Schema: ApplicationSchema,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
    TargetPort: ApplicationInputPort<Schema, TargetFeature, Value = SourcePort::Value>,
    Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>,
{
    fn append(
        _: &mut WorthQueryProgramExecutionBindings,
    ) -> Result<(), WorthQueryApplicationProgramInstallationDenial> {
        Ok(())
    }
}

pub trait WorthQueryProgramConnectionBindings<Schema>
where
    Schema: ApplicationSchema,
{
    fn bind(
    ) -> Result<WorthQueryProgramExecutionBindings, WorthQueryApplicationProgramInstallationDenial>;
}

impl<Schema> WorthQueryProgramConnectionBindings<Schema> for ()
where
    Schema: ApplicationSchema,
{
    fn bind(
    ) -> Result<WorthQueryProgramExecutionBindings, WorthQueryApplicationProgramInstallationDenial>
    {
        Ok(WorthQueryProgramExecutionBindings::new())
    }
}

macro_rules! impl_connection_bindings {
    ($($binding:ident),+) => {
        impl<Schema, $($binding),+> WorthQueryProgramConnectionBindings<Schema> for ($($binding,)+)
        where
            Schema: ApplicationSchema,
            $($binding: WorthQueryProgramConnectionBinding<Schema>,)+
        {
            fn bind() -> Result<
                WorthQueryProgramExecutionBindings,
                WorthQueryApplicationProgramInstallationDenial,
            > {
                let mut bindings = WorthQueryProgramExecutionBindings::new();
                $($binding::append(&mut bindings)?;)+
                Ok(bindings)
            }
        }
    };
}

impl_connection_bindings!(A);
impl_connection_bindings!(A, B);
impl_connection_bindings!(A, B, C);
impl_connection_bindings!(A, B, C, D);
impl_connection_bindings!(A, B, C, D, E);
impl_connection_bindings!(A, B, C, D, E, F);
impl_connection_bindings!(A, B, C, D, E, F, G);
impl_connection_bindings!(A, B, C, D, E, F, G, H);
impl_connection_bindings!(A, B, C, D, E, F, G, H, I);
impl_connection_bindings!(A, B, C, D, E, F, G, H, I, J);
impl_connection_bindings!(A, B, C, D, E, F, G, H, I, J, K);
impl_connection_bindings!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_connection_bindings!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_connection_bindings!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_connection_bindings!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_connection_bindings!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executable_connection_identity_must_equal_its_declaration() {
        let denial = require_connection_identity("declared", "executable")
            .expect_err("a producer cannot bind under another connection identity");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationProgramInstallationDenialKind::ConnectionIdentityMismatch
        );
        assert_eq!(denial.subject(), "declared");
    }

    #[test]
    fn dependent_root_demand_must_equal_its_parent_output_demand() {
        struct ParentFeature;
        struct ChildFeature;
        struct ParentDemand;
        struct WrongDemand;
        let mut bindings = WorthQueryProgramExecutionBindings::new();
        let connection =
            |identity, source, target, root, demand| WorthQueryProgramExecutionConnectionBinding {
                identity,
                node_type: TypeId::of::<()>(),
                binding_type: TypeId::of::<()>(),
                source_feature_type: source,
                source_port_type: TypeId::of::<()>(),
                target_feature_type: target,
                target_port_type: TypeId::of::<()>(),
                root_demand_type: root,
                target_demand_type: demand,
            };
        bindings.connections.push(connection(
            "root",
            TypeId::of::<()>(),
            TypeId::of::<ParentFeature>(),
            None,
            Some(TypeId::of::<ParentDemand>()),
        ));
        bindings.connections.push(connection(
            "dependent",
            TypeId::of::<ParentFeature>(),
            TypeId::of::<ChildFeature>(),
            Some(TypeId::of::<WrongDemand>()),
            Some(TypeId::of::<()>()),
        ));
        let denial = require_matching_root_demands(&bindings)
            .expect_err("a dependent cannot claim another parent demand type");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationProgramInstallationDenialKind::RootDemandMismatch
        );
        assert_eq!(denial.subject(), "dependent");
    }
}
