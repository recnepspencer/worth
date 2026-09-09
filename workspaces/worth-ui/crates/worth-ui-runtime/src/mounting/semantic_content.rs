use std::collections::BTreeMap;
use std::sync::Arc;

#[cfg(test)]
#[path = "semantic_content_tests.rs"]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedSemanticContentInput {
    by_graph_node: BTreeMap<crate::graph::UiGraphNodeIdentity, UiMountedSemanticTextContent>,
    projection_inputs: UiMountedProjectionInputTransition,
    schema_transitions: Vec<crate::runtime::rebind::UiProjectionSchemaTransition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedProjectionInputTransition {
    Retain,
    Merge {
        capacity: usize,
        inputs: BTreeMap<
            worth_ui_query_binding::UiProjectionInputSlot,
            worth_ui_query_binding::UiProjectionInputFactTransition,
        >,
    },
    Replace {
        capacity: usize,
        inputs: BTreeMap<
            worth_ui_query_binding::UiProjectionInputSlot,
            worth_ui_query_binding::UiProjectionInputFactTransition,
        >,
    },
}

impl UiMountedProjectionInputTransition {
    pub(in crate::mounting) fn replaces_table(&self, predecessor_capacity: usize) -> bool {
        match self {
            Self::Retain => false,
            Self::Merge { capacity, .. } => *capacity != predecessor_capacity,
            Self::Replace { .. } => true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedSemanticTextContent {
    Scalar(UiMountedScalarSemanticTextContent),
    Collection(UiMountedCollectionSemanticTextContent),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedScalarSemanticTextContent {
    value: UiMountedSemanticTextValueDirective,
    posture: Arc<str>,
    formatting: Option<UiMountedSemanticTextFormattingDirective>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedSemanticTextFormattingDirective {
    contract: crate::capability::ComponentSemanticTextContract,
    token_values: BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedCollectionSemanticTextContent {
    value: UiMountedCollectionTextDirective,
    posture: Arc<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedSemanticTextValueDirective {
    Replace(Arc<str>),
    Preserve,
    Clear,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedCollectionTextDirective {
    Replace(Box<[UiMountedCollectionTextRow]>),
    Patch(Box<[UiMountedCollectionTextChange]>),
    Preserve,
    Clear,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedCollectionTextRow {
    identity: UiMountedCollectionRowIdentity,
    selected_values: Box<[Arc<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedCollectionTextChange {
    Insert {
        row: UiMountedCollectionTextRow,
        at: usize,
    },
    Remove {
        identity: UiMountedCollectionRowIdentity,
        from: usize,
    },
    Move {
        identity: UiMountedCollectionRowIdentity,
        from: usize,
        to: usize,
    },
    Regroup {
        identity: UiMountedCollectionRowIdentity,
    },
    Update(UiMountedCollectionTextRow),
    WindowShift,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedCollectionRowIdentity(
    worth_ui_query_binding::UiCollectionProjectionRowReference,
);

impl UiMountedSemanticContentInput {
    pub(crate) fn empty() -> Self {
        Self {
            by_graph_node: BTreeMap::new(),
            projection_inputs: UiMountedProjectionInputTransition::Retain,
            schema_transitions: Vec::new(),
        }
    }

    pub(crate) fn merge_application_presentation(
        &mut self,
        presentation: Self,
    ) -> Result<(), crate::mounting::UiMountedProjectionDenial> {
        if !matches!(
            presentation.projection_inputs,
            UiMountedProjectionInputTransition::Retain
        ) || !presentation.schema_transitions.is_empty()
        {
            return Err(crate::mounting::UiMountedProjectionDenial::DuplicateLaneContribution);
        }
        if presentation
            .by_graph_node
            .keys()
            .any(|graph_node| self.by_graph_node.contains_key(graph_node))
        {
            return Err(crate::mounting::UiMountedProjectionDenial::DuplicateLaneContribution);
        }
        self.by_graph_node.extend(presentation.by_graph_node);
        Ok(())
    }

    pub(crate) fn insert_scalar(
        &mut self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        value: UiMountedSemanticTextValueDirective,
        posture: Arc<str>,
    ) -> Result<(), ()> {
        self.insert_scalar_with_formatting(graph_node, value, posture, None)
    }

    pub(crate) fn insert_scalar_with_formatting(
        &mut self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        value: UiMountedSemanticTextValueDirective,
        posture: Arc<str>,
        formatting: Option<UiMountedSemanticTextFormattingDirective>,
    ) -> Result<(), ()> {
        self.insert(
            graph_node,
            UiMountedSemanticTextContent::Scalar(UiMountedScalarSemanticTextContent {
                value,
                posture,
                formatting,
            }),
        )
    }

    pub(crate) fn insert_collection(
        &mut self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        value: UiMountedCollectionTextDirective,
        posture: Arc<str>,
    ) -> Result<(), ()> {
        self.insert(
            graph_node,
            UiMountedSemanticTextContent::Collection(UiMountedCollectionSemanticTextContent {
                value,
                posture,
            }),
        )
    }

    fn insert(
        &mut self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        content: UiMountedSemanticTextContent,
    ) -> Result<(), ()> {
        if self.by_graph_node.insert(graph_node, content).is_some() {
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn get(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Option<&UiMountedSemanticTextContent> {
        self.by_graph_node.get(&graph_node)
    }

    pub(crate) fn graph_nodes(
        &self,
    ) -> impl ExactSizeIterator<Item = crate::graph::UiGraphNodeIdentity> + '_ {
        self.by_graph_node.keys().copied()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.by_graph_node.is_empty()
            && match &self.projection_inputs {
                UiMountedProjectionInputTransition::Retain => true,
                UiMountedProjectionInputTransition::Merge { inputs, .. } => inputs.is_empty(),
                UiMountedProjectionInputTransition::Replace { .. } => false,
            }
    }

    pub(crate) fn merge_projection_inputs(&mut self, capacity: usize) {
        self.projection_inputs = UiMountedProjectionInputTransition::Merge {
            capacity,
            inputs: BTreeMap::new(),
        };
    }

    pub(crate) fn replace_projection_inputs(&mut self, capacity: usize) {
        self.projection_inputs = UiMountedProjectionInputTransition::Replace {
            capacity,
            inputs: BTreeMap::new(),
        };
    }

    pub(crate) fn require_projection_input_replacement(&mut self, capacity: usize) {
        let transition = std::mem::replace(
            &mut self.projection_inputs,
            UiMountedProjectionInputTransition::Retain,
        );
        self.projection_inputs = match transition {
            UiMountedProjectionInputTransition::Retain => {
                UiMountedProjectionInputTransition::Replace {
                    capacity,
                    inputs: BTreeMap::new(),
                }
            }
            UiMountedProjectionInputTransition::Merge {
                capacity: declared,
                inputs,
            }
            | UiMountedProjectionInputTransition::Replace {
                capacity: declared,
                inputs,
            } => {
                assert_eq!(
                    declared, capacity,
                    "candidate projection facts use the candidate plan width"
                );
                UiMountedProjectionInputTransition::Replace { capacity, inputs }
            }
        };
    }

    pub(crate) fn insert_projection_input_transition(
        &mut self,
        input: worth_ui_query_binding::UiProjectionInputFactTransition,
    ) -> Result<(), ()> {
        let slot = input.revision().slot();
        let (capacity, inputs) = match &mut self.projection_inputs {
            UiMountedProjectionInputTransition::Retain => return Err(()),
            UiMountedProjectionInputTransition::Merge { capacity, inputs }
            | UiMountedProjectionInputTransition::Replace { capacity, inputs } => {
                (*capacity, inputs)
            }
        };
        if slot.index() >= capacity {
            return Err(());
        }
        match inputs.get(&slot) {
            Some(existing) if existing != &input => Err(()),
            Some(_) => Ok(()),
            None => {
                inputs.insert(slot, input);
                Ok(())
            }
        }
    }

    pub(crate) const fn projection_input_transition(&self) -> &UiMountedProjectionInputTransition {
        &self.projection_inputs
    }

    pub(crate) fn record_schema_transition(
        &mut self,
        transition: crate::runtime::rebind::UiProjectionSchemaTransition,
    ) {
        self.schema_transitions.push(transition);
    }

    pub(crate) fn schema_transitions(
        &self,
    ) -> &[crate::runtime::rebind::UiProjectionSchemaTransition] {
        &self.schema_transitions
    }
}

impl UiMountedScalarSemanticTextContent {
    pub(crate) const fn value(&self) -> &UiMountedSemanticTextValueDirective {
        &self.value
    }

    pub(crate) fn posture(&self) -> &Arc<str> {
        &self.posture
    }

    pub(crate) const fn formatting(&self) -> Option<&UiMountedSemanticTextFormattingDirective> {
        self.formatting.as_ref()
    }
}

impl UiMountedSemanticTextFormattingDirective {
    pub(crate) fn new(
        contract: crate::capability::ComponentSemanticTextContract,
        token_values: BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>,
    ) -> Self {
        Self {
            contract,
            token_values,
        }
    }

    pub(crate) const fn contract(&self) -> &crate::capability::ComponentSemanticTextContract {
        &self.contract
    }

    pub(crate) fn token_value(
        &self,
        token: &crate::capability::ThemeTokenId,
    ) -> Option<&crate::capability::ThemeTokenValue> {
        self.token_values.get(token)
    }
}

impl UiMountedCollectionSemanticTextContent {
    pub(crate) const fn value(&self) -> &UiMountedCollectionTextDirective {
        &self.value
    }

    pub(crate) fn posture(&self) -> &Arc<str> {
        &self.posture
    }
}

impl UiMountedCollectionTextRow {
    pub(crate) fn new(
        identity: UiMountedCollectionRowIdentity,
        selected_values: impl Into<Box<[Arc<str>]>>,
    ) -> Self {
        Self {
            identity,
            selected_values: selected_values.into(),
        }
    }

    pub(crate) fn identity(&self) -> &UiMountedCollectionRowIdentity {
        &self.identity
    }

    pub(crate) fn selected_values(&self) -> &[Arc<str>] {
        &self.selected_values
    }
}

impl UiMountedCollectionRowIdentity {
    pub(crate) fn from_query(
        identity: &worth_ui_query_binding::UiCollectionProjectionRowReference,
    ) -> Self {
        Self(identity.clone())
    }

    pub(crate) fn query_reference(
        &self,
    ) -> &worth_ui_query_binding::UiCollectionProjectionRowReference {
        &self.0
    }
}
