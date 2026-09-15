mod connection_plan;
mod node;
mod settlement;
pub use settlement::*;

pub use connection_plan::{WorthQueryProgramConnectionPlan, WorthQueryProgramRootConnection};

use std::collections::BTreeSet;

use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationOutputPort, ApplicationProgramDefinition,
    ApplicationProgramInventoryIdentity,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_installation::facade::WorthQueryInstalledProgramInventoryPosture;

use connection_plan::ErasedProgramConnection;
use node::{runtime_matches, ErasedProgramNode, ErasedProgramSettlement, TypedProgramNode};

#[doc(hidden)]
pub struct WorthQueryProgramConnectionFactories<'application, Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    connections: Vec<
        Box<dyn ErasedProgramConnection<'application, Schema, Program, Inventory> + 'application>,
    >,
}

impl<'application, Schema, Program, Inventory>
    WorthQueryProgramConnectionFactories<'application, Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    fn new(
        connections: Vec<
            Box<
                dyn ErasedProgramConnection<'application, Schema, Program, Inventory>
                    + 'application,
            >,
        >,
    ) -> Self {
        Self { connections }
    }

    fn into_connections(
        self,
    ) -> Vec<
        Box<dyn ErasedProgramConnection<'application, Schema, Program, Inventory> + 'application>,
    > {
        self.connections
    }
}

pub struct WorthQueryApplicationProgramOutputHandle<'application, Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
    nodes: Vec<Option<Box<dyn ErasedProgramNode<'application, Schema> + 'application>>>,
    settlements: Vec<ErasedProgramSettlement>,
    connections: Vec<
        Box<dyn ErasedProgramConnection<'application, Schema, Program, Inventory> + 'application>,
    >,
    required_features: BTreeSet<&'static str>,
    inventory_features: BTreeSet<&'static str>,
    empty_features: BTreeSet<&'static str>,
    started_edges: BTreeSet<(&'static str, usize)>,
    controls: crate::application_entry::WorthQueryOutputDemandControls,
    closed: bool,
}

impl<'application, Schema, Program, Inventory>
    WorthQueryApplicationProgramOutputHandle<'application, Schema, Program, Inventory>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryProgramConnectionPlan<Schema, Program, Inventory>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    pub(in crate::application_entry) fn new<Demand>(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
        root_feature: &'static str,
        root_feature_type: std::any::TypeId,
        root: crate::application_entry::demand::WorthQueryProgramNodeHandle<
            'application,
            Schema,
            Program,
            Inventory,
            Demand,
        >,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<Self, crate::application_entry::WorthQueryRequiredOutputPreparationDenial>
    where
        Demand: worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand<Schema>
            + Clone
            + 'static,
        TypedProgramNode<'application, Schema, Program, Inventory, Demand>:
            ErasedProgramNode<'application, Schema>,
    {
        match application.installed_program().inventory_posture::<Inventory>() {
            WorthQueryInstalledProgramInventoryPosture::Missing => {
                return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingInventory)
            }
            WorthQueryInstalledProgramInventoryPosture::Unavailable { feature } => {
                return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Unavailable { feature })
            }
            WorthQueryInstalledProgramInventoryPosture::Available => {}
        }
        let (required_features, inventory_features) =
            inventory_closure::<Schema, Program, Inventory>(application)?;
        if !required_features.contains(root_feature) {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::WrongRoot,
            );
        }
        Ok(Self {
            application,
            source_commit,
            nodes: vec![Some(Box::new(TypedProgramNode::new(
                root_feature,
                root_feature_type,
                root,
            )))],
            settlements: Vec::new(),
            connections: Program::Connections::connections().into_connections(),
            required_features,
            inventory_features,
            empty_features: BTreeSet::new(),
            started_edges: BTreeSet::new(),
            controls,
            closed: false,
        })
    }

    pub fn notifications(
        &self,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryOutputDemandNotifications,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        self.nodes
            .iter()
            .flatten()
            .next()
            .ok_or(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed)?
            .notifications()
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)
    }

    pub fn advance(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<
        WorthQueryApplicationProgramOutputProgress<Schema, Program, Inventory>,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        if self.closed {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed,
            );
        }
        if !runtime_matches(self.application, request) {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::ForeignProgram,
            );
        }
        self.advance_active(request)?;
        self.start_ready_connections(request)?;
        if self.nodes.iter().any(Option::is_some) {
            return Ok(WorthQueryApplicationProgramOutputProgress::Pending);
        }
        let complete = self
            .inventory_features
            .iter()
            .all(|feature| self.feature_is_complete(feature));
        if !complete {
            return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::IncompleteInventory);
        }
        self.closed = true;
        worth_query_execution::facade::publication_integration::program_execution_port(
            self.application,
        )
        .release_program_recovery(&self.source_commit);
        let basis = self
            .settlements
            .iter()
            .map(|settlement| &settlement.observation)
            .max_by_key(|observation| observation.selected_commit().ordinal())
            .expect("an admitted program settles its root before completion")
            .clone();
        let root_observation = self
            .settlements
            .first()
            .expect("the first program settlement belongs to its root")
            .observation
            .clone();
        Ok(WorthQueryApplicationProgramOutputProgress::Settled(
            WorthQueryApplicationProgramOutputSettlement {
                outputs: self.settled_inventory_outputs(),
                basis,
                root_observation,
                marker: std::marker::PhantomData,
            },
        ))
    }

    fn settled_inventory_outputs(&self) -> Vec<SettledProgramOutput> {
        let inventory = self
            .application
            .installed_program()
            .inventory::<Inventory>()
            .expect("the admitted inventory remains installed");
        inventory
            .outputs()
            .iter()
            .flat_map(|output| {
                self.settlements
                    .iter()
                    .filter(move |settlement| settlement.feature_type == output.feature_type())
                    .map(move |settlement| SettledProgramOutput {
                        feature_type: output.feature_type(),
                        port_type: output.port_type(),
                        demand: std::sync::Arc::clone(&settlement.demand),
                        observation: settlement.observation.clone(),
                    })
            })
            .collect()
    }

    fn advance_active(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<(), crate::application_entry::WorthQueryRequiredOutputPreparationDenial> {
        for index in 0..self.nodes.len() {
            let Some(node) = &mut self.nodes[index] else {
                continue;
            };
            if let Some(settlement) = node.advance(request).map_err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand,
            )? {
                self.nodes[index] = None;
                self.settlements.push(settlement);
            }
        }
        Ok(())
    }

    fn start_ready_connections(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<(), crate::application_entry::WorthQueryRequiredOutputPreparationDenial> {
        for connection_index in 0..self.connections.len() {
            let connection = &self.connections[connection_index];
            if !self.required_features.contains(connection.target_feature())
                || !self.parents_complete(connection.target_feature())
            {
                continue;
            }
            for settlement_index in 0..self.settlements.len() {
                let parent = &self.settlements[settlement_index];
                if parent.feature != connection.source_feature()
                    || !self
                        .started_edges
                        .insert((connection.identity(), settlement_index))
                {
                    continue;
                }
                let started = connection.start(parent, request, self.application, self.controls)?;
                if started.is_empty() {
                    self.empty_features.insert(connection.target_feature());
                }
                self.nodes.extend(started.into_iter().map(Some));
            }
        }
        Ok(())
    }

    fn parents_complete(&self, target: &str) -> bool {
        self.application
            .installed_program()
            .connections()
            .iter()
            .filter(|connection| connection.target_feature() == target)
            .all(|connection| self.feature_is_complete(connection.source_feature()))
    }

    fn feature_is_complete(&self, feature: &str) -> bool {
        (self.empty_features.contains(feature)
            || self
                .settlements
                .iter()
                .any(|settlement| settlement.feature == feature))
            && !self
                .nodes
                .iter()
                .flatten()
                .any(|node| node.feature() == feature)
    }

    pub fn close(&mut self) {
        if self.closed {
            return;
        }
        for node in self.nodes.iter_mut().flatten() {
            node.close();
        }
        self.nodes.clear();
        self.closed = true;
    }
}

impl<Schema, Program, Inventory> Drop
    for WorthQueryApplicationProgramOutputHandle<'_, Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    fn drop(&mut self) {
        if !self.closed {
            for node in self.nodes.iter_mut().flatten() {
                node.close();
            }
        }
    }
}

fn inventory_closure<Schema, Program, Inventory>(
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
) -> Result<
    (BTreeSet<&'static str>, BTreeSet<&'static str>),
    crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    let inventory = application
        .installed_program()
        .inventory::<Inventory>()
        .ok_or(
            crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingInventory,
        )?;
    let inventory_feature_types = inventory
        .outputs()
        .iter()
        .map(|output| output.feature_type())
        .collect::<BTreeSet<_>>();
    let required_types = application
        .installed_program()
        .inventory_feature_closure::<Inventory>()
        .expect("the inventory was resolved above");
    let identities = application
        .installed_program()
        .features()
        .iter()
        .filter(|feature| required_types.contains(&feature.type_id()))
        .map(|feature| feature.identity())
        .collect();
    let inventory_features = application
        .installed_program()
        .features()
        .iter()
        .filter(|feature| inventory_feature_types.contains(&feature.type_id()))
        .map(|feature| feature.identity())
        .collect();
    Ok((identities, inventory_features))
}
