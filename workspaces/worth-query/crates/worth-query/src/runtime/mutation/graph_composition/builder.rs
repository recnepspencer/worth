use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeSet;

mod existing_target_methods;

use super::denial::{
    graph_composition_error, WorthQueryGraphCompositionDenial, WorthQueryGraphCompositionDenialKind,
};
use super::relation_builder::WorthQueryGraphRelationMutationBuilder;
use super::symbols::{WorthQueryGraphEntitySymbol, WorthQueryGraphRelationSymbol};
use crate::runtime::mutation::{WorthQueryAspectMutationBuilder, WorthQueryDeleteMutationBuilder};
use crate::runtime::{
    WorthQueryExistingTruthTargetBinding, WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionProgram, WorthQueryGraphCompositionProgramStep,
    WorthQueryGraphCompositionProgramStepKind, WorthQueryMutationSymbolIdentity,
    WorthQueryMutationTargetCollectionIdentity, WorthQueryRuntimeError,
    WorthQuerySymbolicTargetReference, WorthQueryWriteCommand,
};
use worth_relational::facade::identity::KindId;

#[derive(Clone, Debug, Default)]
pub struct WorthQueryGraphCompositionBuilder {
    commands: Vec<WorthQueryWriteCommand>,
    program_steps: Vec<WorthQueryGraphCompositionProgramStep>,
    declared_symbols: BTreeSet<WorthQueryMutationSymbolIdentity>,
    symbolic_entity_declaration_count: usize,
    symbolic_relation_declaration_count: usize,
    error: Option<WorthQueryGraphCompositionDenial>,
}

impl WorthQueryGraphCompositionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert_entity(
        &mut self,
        symbol: impl Into<String>,
        collection: impl Into<String>,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryGraphEntitySymbol, WorthQueryRuntimeError> {
        self.require_clean()?;
        let collection = collection.into();
        let reference = self.declare_symbol(symbol, &collection)?;
        let component_index = self.commands.len();
        let command = declaration(WorthQueryAspectMutationBuilder::new())
            .build_insert_symbolic_reference(reference.clone(), collection)?;
        self.commands.push(command);
        self.program_steps
            .push(WorthQueryGraphCompositionProgramStep::new(
                component_index,
                WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration,
                reference.target_collection_identity().cloned(),
                Some(reference.symbol().to_string()),
            ));
        self.symbolic_entity_declaration_count += 1;
        Ok(WorthQueryGraphEntitySymbol::new(reference))
    }
    pub fn insert_relation(
        &mut self,
        collection: impl Into<String>,
        declaration: impl FnOnce(
            WorthQueryGraphRelationMutationBuilder,
        ) -> WorthQueryGraphRelationMutationBuilder,
    ) -> Result<(), WorthQueryRuntimeError> {
        self.insert_relation_with_kind_id(collection, None, declaration)
    }

    pub fn insert_relation_with_kind_id(
        &mut self,
        collection: impl Into<String>,
        relation_kind_id: impl Into<Option<KindId>>,
        declaration: impl FnOnce(
            WorthQueryGraphRelationMutationBuilder,
        ) -> WorthQueryGraphRelationMutationBuilder,
    ) -> Result<(), WorthQueryRuntimeError> {
        self.require_clean()?;
        let component_index = self.commands.len();
        let collection = collection.into();
        let relation_kind_id = relation_kind_id.into();
        let command = declaration(WorthQueryGraphRelationMutationBuilder::new())
            .into_inner()
            .build_insert(collection.clone())?;
        self.commands.push(command);
        self.program_steps.push(graph_program_step(
            component_index,
            WorthQueryGraphCompositionProgramStepKind::RelationDeclaration,
            Some(graph_program_declared_collection(collection)?),
            None,
            relation_kind_id,
        ));
        self.symbolic_relation_declaration_count += 1;
        Ok(())
    }
    pub fn update_entity(
        &mut self,
        symbol: &WorthQueryGraphEntitySymbol,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<(), WorthQueryRuntimeError> {
        self.require_clean()?;
        let component_index = self.commands.len();
        let command = declaration(WorthQueryAspectMutationBuilder::new())
            .build_update_symbolic(symbol.reference())?;
        self.commands.push(command);
        self.program_steps
            .push(WorthQueryGraphCompositionProgramStep::new(
                component_index,
                WorthQueryGraphCompositionProgramStepKind::SymbolicEntityFollowupMutation,
                symbol.reference().target_collection_identity().cloned(),
                Some(symbol.symbol().to_string()),
            ));
        Ok(())
    }
    pub fn insert_symbolic_relation(
        &mut self,
        symbol: impl Into<String>,
        collection: impl Into<String>,
        declaration: impl FnOnce(
            WorthQueryGraphRelationMutationBuilder,
        ) -> WorthQueryGraphRelationMutationBuilder,
    ) -> Result<WorthQueryGraphRelationSymbol, WorthQueryRuntimeError> {
        self.insert_symbolic_relation_with_kind_id(symbol, collection, None, declaration)
    }

    pub fn insert_symbolic_relation_with_kind_id(
        &mut self,
        symbol: impl Into<String>,
        collection: impl Into<String>,
        relation_kind_id: impl Into<Option<KindId>>,
        declaration: impl FnOnce(
            WorthQueryGraphRelationMutationBuilder,
        ) -> WorthQueryGraphRelationMutationBuilder,
    ) -> Result<WorthQueryGraphRelationSymbol, WorthQueryRuntimeError> {
        self.require_clean()?;
        let collection = collection.into();
        let relation_kind_id = relation_kind_id.into();
        let reference = self.declare_symbol(symbol, &collection)?;
        let component_index = self.commands.len();
        let command = declaration(WorthQueryGraphRelationMutationBuilder::new())
            .into_inner()
            .build_insert_symbolic_reference(reference.clone(), collection)?;
        self.commands.push(command);
        self.program_steps.push(graph_program_step(
            component_index,
            WorthQueryGraphCompositionProgramStepKind::SymbolicRelationDeclaration,
            reference.target_collection_identity().cloned(),
            Some(reference.symbol().to_string()),
            relation_kind_id,
        ));
        self.symbolic_relation_declaration_count += 1;
        Ok(WorthQueryGraphRelationSymbol::new(
            reference,
            relation_kind_id,
        ))
    }
    pub fn update_relation(
        &mut self,
        symbol: &WorthQueryGraphRelationSymbol,
        declaration: impl FnOnce(
            WorthQueryGraphRelationMutationBuilder,
        ) -> WorthQueryGraphRelationMutationBuilder,
    ) -> Result<(), WorthQueryRuntimeError> {
        self.require_clean()?;
        let component_index = self.commands.len();
        let command = declaration(WorthQueryGraphRelationMutationBuilder::new())
            .into_inner()
            .build_update_symbolic(symbol.reference())?;
        self.commands.push(command);
        self.program_steps.push(graph_program_step(
            component_index,
            WorthQueryGraphCompositionProgramStepKind::SymbolicRelationFollowupMutation,
            symbol.reference().target_collection_identity().cloned(),
            Some(symbol.symbol().to_string()),
            symbol.relation_kind_id(),
        ));
        Ok(())
    }
    pub fn delete_relation(
        &mut self,
        symbol: &WorthQueryGraphRelationSymbol,
        declaration: impl FnOnce(WorthQueryDeleteMutationBuilder) -> WorthQueryDeleteMutationBuilder,
    ) -> Result<(), WorthQueryRuntimeError> {
        self.require_clean()?;
        let component_index = self.commands.len();
        let command = declaration(WorthQueryDeleteMutationBuilder::new())
            .build_delete_symbolic(symbol.reference())?;
        self.commands.push(command);
        self.program_steps.push(graph_program_step(
            component_index,
            WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement,
            symbol.reference().target_collection_identity().cloned(),
            Some(symbol.symbol().to_string()),
            symbol.relation_kind_id(),
        ));
        Ok(())
    }
    pub fn finish(
        self,
    ) -> Result<
        (
            Vec<WorthQueryWriteCommand>,
            WorthQueryGraphCompositionBreadth,
            WorthQueryGraphCompositionProgram,
        ),
        WorthQueryRuntimeError,
    > {
        if let Some(error) = self.error {
            return Err(WorthQueryRuntimeError::GraphCompositionDenied(error));
        }
        if self.commands.is_empty() {
            return Err(graph_composition_error(
                WorthQueryGraphCompositionDenialKind::EmptyComposition,
                None,
                None,
                "graph composition must declare at least one operation",
            )?);
        }
        let breadth = WorthQueryGraphCompositionBreadth::new(
            self.commands.len(),
            self.symbolic_entity_declaration_count,
            self.symbolic_relation_declaration_count,
        );
        let program = WorthQueryGraphCompositionProgram::new(self.program_steps, &breadth)?;
        Ok((self.commands, breadth, program))
    }
    fn require_clean(&self) -> Result<(), WorthQueryRuntimeError> {
        if let Some(error) = &self.error {
            return Err(WorthQueryRuntimeError::GraphCompositionDenied(
                error.clone(),
            ));
        }
        Ok(())
    }
    fn declare_symbol(
        &mut self,
        symbol: impl Into<String>,
        collection: &str,
    ) -> Result<WorthQuerySymbolicTargetReference, WorthQueryRuntimeError> {
        let reference =
            WorthQuerySymbolicTargetReference::new(symbol).map_err(WorthQueryRuntimeError::from)?;
        let symbol_identity = reference.symbol_identity().clone();
        if !self.declared_symbols.insert(symbol_identity) {
            let symbol = reference.symbol().to_string();
            let message =
                format!("graph composition symbol `{symbol}` was declared more than once");
            let denial = WorthQueryGraphCompositionDenial::new(
                WorthQueryGraphCompositionDenialKind::DuplicateSymbolDeclaration,
                Some(symbol),
                Some(graph_program_declared_collection(collection)?),
                message,
            )?;
            self.error = Some(denial.clone());
            return Err(WorthQueryRuntimeError::GraphCompositionDenied(denial));
        }
        reference
            .in_target_collection(collection)
            .map_err(WorthQueryRuntimeError::from)
    }
    fn push_existing_target_step(
        &mut self,
        command: WorthQueryWriteCommand,
        kind: WorthQueryGraphCompositionProgramStepKind,
        declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    ) {
        let component_index = self.commands.len();
        self.commands.push(command);
        self.program_steps
            .push(WorthQueryGraphCompositionProgramStep::new(
                component_index,
                kind,
                declared_collection,
                None,
            ));
    }
}

fn graph_program_step(
    component_index: usize,
    kind: WorthQueryGraphCompositionProgramStepKind,
    declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    declared_symbol: Option<String>,
    relation_kind_id: Option<KindId>,
) -> WorthQueryGraphCompositionProgramStep {
    let step = WorthQueryGraphCompositionProgramStep::new(
        component_index,
        kind,
        declared_collection,
        declared_symbol,
    );
    match relation_kind_id {
        Some(relation_kind_id) => step.with_relation_kind_id(relation_kind_id),
        None => step,
    }
}

fn graph_program_declared_collection(
    collection: impl Into<String>,
) -> Result<WorthQueryMutationTargetCollectionIdentity, IdentityDerivationDenial> {
    Ok(WorthQueryMutationTargetCollectionIdentity::new(
        "graph-composition-program",
        collection,
    )?)
}
