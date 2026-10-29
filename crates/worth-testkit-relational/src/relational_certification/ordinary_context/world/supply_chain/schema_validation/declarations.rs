pub use {std::collections::BTreeMap, std::collections::BTreeSet};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema, crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationContract, crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind};

impl SupplyChainSchema {
    pub fn validate_edge(
        &self,
        edge: RelationEdge,
        source_kind: EntityKind,
        target_kind: EntityKind,
    ) -> Result<(), SchemaError> {
        let contract = self
            .relations
            .get(&edge.key.kind)
            .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::UnknownRelation(edge.key.kind))?;
        if contract.source != source_kind || contract.target != target_kind {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                relation: edge.key.kind,
                source: source_kind,
                target: target_kind,
            });
        }
        Ok(())
    }

    /// Validates a relation vector without requiring every minimum cardinality.
    /// This is the correct boundary for adversarial partial observations.
    pub fn validate_relation_sequence(
        &self,
        edges: &[RelationEdge],
        entities: &BTreeMap<EntityKey, EntityRecord>,
    ) -> Result<(), SchemaError> {
        let mut seen = std::collections::BTreeSet::new();
        let mut counts = BTreeMap::<(RelationKind, EntityKey), u16>::new();
        for edge in edges {
            if !seen.insert(edge.key) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::DuplicateRelation(edge.key));
            }
            let source_kind = entities.get(&edge.source).map(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::kind).ok_or(
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                    relation: edge.key.kind,
                    source: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                    target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                },
            )?;
            let target_kind = entities.get(&edge.target).map(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::kind).ok_or(
                crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                    relation: edge.key.kind,
                    source: source_kind,
                    target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                },
            )?;
            self.validate_edge(*edge, source_kind, target_kind)?;
            let count = counts.entry((edge.key.kind, edge.source)).or_default();
            *count += 1;
            if self
                .contract(edge.key.kind)
                .and_then(|c| c.max_per_source)
                .is_some_and(|maximum| *count > maximum)
            {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::CardinalityExceeded(edge.key.kind, edge.source));
            }
        }
        self.validate_symmetry(edges)?;
        self.validate_route_acyclicity(edges)?;
        self.validate_order(edges, entities)?;
        Ok(())
    }

    /// Validates a complete world, including minimum-cardinality obligations.
    pub fn validate_complete_world(
        &self,
        edges: &[RelationEdge],
        entities: &BTreeMap<EntityKey, EntityRecord>,
    ) -> Result<(), SchemaError> {
        self.validate_relation_sequence(edges, entities)?;
        let mut counts = BTreeMap::<(RelationKind, EntityKey), u16>::new();
        for edge in edges {
            *counts.entry((edge.key.kind, edge.source)).or_default() += 1;
        }
        for contract in self.relations.values() {
            if contract.min_per_source == 0 {
                continue;
            }
            for (key, record) in entities {
                if record.kind() == contract.source
                    && counts
                        .get(&(contract.kind, *key))
                        .copied()
                        .unwrap_or_default()
                        < contract.min_per_source
                {
                    return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::MinimumCardinality(contract.kind, *key));
                }
            }
        }
        self.validate_complete_routes(edges, entities)?;
        self.validate_hazard_meaning(entities)?;
        Ok(())
    }

    pub fn contract(&self, kind: RelationKind) -> Option<&RelationContract> {
        self.relations.get(&kind)
    }

    pub fn validate_symmetry(&self, edges: &[RelationEdge]) -> Result<(), SchemaError> {
        for edge in edges {
            if self
                .contract(edge.key.kind)
                .is_some_and(|contract| contract.symmetric)
                && !edges.iter().any(|candidate| {
                    candidate.key.kind == edge.key.kind
                        && candidate.source == edge.target
                        && candidate.target == edge.source
                })
            {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::MissingSymmetricReverse(edge.key));
            }
        }
        Ok(())
    }

    pub fn validate_order(
        &self,
        edges: &[RelationEdge],
        entities: &BTreeMap<EntityKey, EntityRecord>,
    ) -> Result<(), SchemaError> {
        let mut calls_by_voyage = BTreeMap::<EntityKey, Vec<&RelationEdge>>::new();
        let mut owner_by_call = BTreeMap::<EntityKey, EntityKey>::new();
        for edge in edges
            .iter()
            .filter(|edge| edge.key.kind == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall)
        {
            calls_by_voyage.entry(edge.source).or_default().push(edge);
            if owner_by_call.insert(edge.target, edge.source).is_some() {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::DuplicateVoyageCall(edge.target));
            }
        }
        for calls in calls_by_voyage.values_mut() {
            calls.sort_by_key(|edge| edge.key.ordinal);
            for pair in calls.windows(2) {
                let first = port_call_sequence(entities, pair[0].target)?;
                let second = port_call_sequence(entities, pair[1].target)?;
                if first >= second {
                    return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrderedRouteViolation(pair[1].key));
                }
            }
        }
        for edge in edges
            .iter()
            .filter(|edge| edge.key.kind == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes)
        {
            let Some(owner) = owner_by_call.get(&edge.source) else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrphanRouteLink(edge.key));
            };
            if owner_by_call.get(&edge.target) != Some(owner) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrphanRouteLink(edge.key));
            }
            let source = port_call_sequence(entities, edge.source)?;
            let target = port_call_sequence(entities, edge.target)?;
            if target != source + 1 {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrderedRouteViolation(edge.key));
            }
        }
        Ok(())
    }

    pub fn validate_route_acyclicity(
        &self,
        edges: &[RelationEdge],
    ) -> Result<(), SchemaError> {
        let mut graph = BTreeMap::<EntityKey, Vec<EntityKey>>::new();
        for edge in edges
            .iter()
            .filter(|edge| edge.key.kind == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes)
        {
            graph.entry(edge.source).or_default().push(edge.target);
        }
        let mut visiting = std::collections::BTreeSet::new();
        let mut finished = std::collections::BTreeSet::new();
        for node in graph.keys().copied() {
            if has_cycle(node, &graph, &mut visiting, &mut finished) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::RouteCycle);
            }
        }
        Ok(())
    }

    pub fn validate_complete_routes(
        &self,
        edges: &[RelationEdge],
        entities: &BTreeMap<EntityKey, EntityRecord>,
    ) -> Result<(), SchemaError> {
        let mut calls_by_voyage = BTreeMap::<EntityKey, Vec<&RelationEdge>>::new();
        let mut links = BTreeSet::<(EntityKey, EntityKey)>::new();
        let mut link_keys = BTreeMap::<(EntityKey, EntityKey), RelationKey>::new();
        let mut owners = BTreeMap::<EntityKey, EntityKey>::new();
        for edge in edges {
            match edge.key.kind {
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall => {
                    if owners.insert(edge.target, edge.source).is_some() {
                        return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::DuplicateVoyageCall(edge.target));
                    }
                    calls_by_voyage.entry(edge.source).or_default().push(edge);
                }
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes => {
                    if link_keys
                        .insert((edge.source, edge.target), edge.key)
                        .is_some()
                    {
                        return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrphanRouteLink(edge.key));
                    }
                    links.insert((edge.source, edge.target));
                }
                _ => {}
            }
        }
        for (voyage, calls) in calls_by_voyage {
            let mut ordered = calls;
            ordered
                .sort_by_key(|edge| port_call_sequence(entities, edge.target).unwrap_or(u16::MAX));
            for (index, edge) in ordered.iter().enumerate() {
                let sequence = port_call_sequence(entities, edge.target)?;
                if sequence != index as u16 {
                    return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrderedRouteViolation(edge.key));
                }
                if let Some(next) = ordered.get(index + 1)
                    && !links.contains(&(edge.target, next.target))
                {
                    return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::MissingRouteLink {
                        voyage,
                        source: edge.target,
                        target: next.target,
                    });
                }
            }
        }
        for edge in edges
            .iter()
            .filter(|edge| edge.key.kind == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes)
        {
            if !owners.contains_key(&edge.source) || !owners.contains_key(&edge.target) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrphanRouteLink(edge.key));
            }
            if owners.get(&edge.source) != owners.get(&edge.target) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::OrphanRouteLink(edge.key));
            }
        }
        Ok(())
    }

    pub fn validate_hazard_meaning(
        &self,
        entities: &BTreeMap<EntityKey, EntityRecord>,
    ) -> Result<(), SchemaError> {
        for (entity, record) in entities {
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(cargo) = record else {
                continue;
            };
            if cargo.hazard == crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::HazardousV2
                && (self.version.hazard == crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema::V1 || self.version != crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2)
            {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::HazardMeaningViolation {
                    entity: *entity,
                    schema: self.version,
                    hazard: cargo.hazard,
                });
            }
        }
        Ok(())
    }
}

pub fn port_call_sequence(
    entities: &BTreeMap<EntityKey, EntityRecord>,
    key: EntityKey,
) -> Result<u16, SchemaError> {
    match entities.get(&key) {
        Some(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(call)) => Ok(call.sequence),
        Some(record) => Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
            relation: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes,
            source: record.kind(),
            target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
        }),
        None => Err(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
            relation: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes,
            source: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
            target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
        }),
    }
}

pub fn has_cycle(
    node: EntityKey,
    graph: &BTreeMap<EntityKey, Vec<EntityKey>>,
    visiting: &mut BTreeSet<EntityKey>,
    finished: &mut BTreeSet<EntityKey>,
) -> bool {
    if visiting.contains(&node) {
        return true;
    }
    if finished.contains(&node) {
        return false;
    }
    visiting.insert(node);
    for child in graph.get(&node).into_iter().flatten() {
        if has_cycle(*child, graph, visiting, finished) {
            return true;
        }
    }
    visiting.remove(&node);
    finished.insert(node);
    false
}