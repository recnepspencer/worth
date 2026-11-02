//! One admitted schema graph is shared by source roles and selected roots.
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::region_directory::corrupt;
use crate::durability::data::{DurableBranchRootSchemaImage, RecoveryPlan, RelationalRecoveryStop};
use crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(super) struct OriginalSchemaGraph {
    pub(super) original:
        Arc<crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs>,
    pub(super) schemas: Arc<ReadmittedSourceSchemaCatalog>,
}
impl OriginalSchemaGraph {
    pub(super) fn admit(
        plan: &RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<Self>, RelationalRecoveryStop> {
        let Some(original) = &plan.original_source else {
            return Ok(None);
        };
        let mut graph = Self {
            original: original.clone(),
            schemas: ReadmittedSourceSchemaCatalog::prepare(request)?,
        };
        for image in &original.current.schemas {
            graph.insert(image, &plan.config.schema.registry, request)?;
        }
        if let Some(checkpoint) = &plan.checkpoint {
            for image in &checkpoint.branch_root_schema_images {
                graph.insert(image, &plan.config.schema.registry, request)?;
            }
        }
        for commit in &plan.tail_log {
            let input = commit
                .envelope()
                .recorded_original_inputs()
                .ok_or_else(|| corrupt("Home tail lacks original canonical inputs"))?;
            if let Some(image) = &input.result_image {
                if image.carrier_digest() != input.result_schema
                    || input.prestate_schema == input.result_schema
                {
                    return Err(corrupt("Home tail result carrier attribution disagrees").into());
                }
                graph.insert(image, &plan.config.schema.registry, request)?;
            } else if input.prestate_schema != input.result_schema {
                return Err(corrupt(
                    "schema-changing canonical publication lacks its result carrier",
                )
                .into());
            }
            graph.get(input.prestate_schema)?;
            let result = graph.get(input.result_schema)?;
            if !result.matches(&commit.envelope().schema_authority)
                || result.descriptor_semantics_version()
                    != commit.envelope().descriptor_semantics_version
            {
                return Err(
                    corrupt("Home tail schema role disagrees with its canonical envelope").into(),
                );
            }
        }
        for transition in plan.branch_transitions.transitions() {
            graph.get(
                transition
                    .schema_carrier()
                    .ok_or_else(|| corrupt("Home transition lacks its original schema role"))?,
            )?;
        }
        if let Some(checkpoint) = &plan.checkpoint {
            for directory in checkpoint
                .directory
                .iter()
                .chain(checkpoint.canonical_directory.iter())
            {
                if let Err(schemas) = directory.source_schemas.set(graph.schemas.clone())
                    && !directory
                        .source_schemas
                        .get()
                        .is_some_and(|existing| Arc::ptr_eq(existing, &schemas))
                {
                    return Err(corrupt(
                        "selected directory already belongs to another schema reconstruction",
                    )
                    .into());
                }
            }
        }
        Ok(Some(graph))
    }

    pub(super) fn get(
        &self,
        digest: [u8; 32],
    ) -> Result<Arc<RelationalBranchRootSchemaAuthority>, RelationalRecoveryStop> {
        self.schemas
            .rows
            .get(&digest)
            .cloned()
            .ok_or_else(|| corrupt("original role names a missing schema carrier").into())
    }

    fn insert(
        &mut self,
        image: &DurableBranchRootSchemaImage,
        configured: &crate::schema::data::RelationalSchemaRegistry,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRecoveryStop> {
        crate::execution::check_request(request).map_err(RelationalRecoveryStop::Request)?;
        if let Some(prior) = self.schemas.rows.get(&image.carrier_digest()) {
            if prior.recorded_image(request)? != *image {
                return Err(corrupt("same source carrier has differing full schema DATA").into());
            }
            return Ok(());
        }
        let (registry, contracts) = image.readmit_registry().map_err(corrupt)?;
        let expected = registry.authority_snapshot();
        let registry = if configured.authority_snapshot() == expected {
            configured.clone()
        } else {
            registry
        };
        let schema = RelationalBranchRootSchemaAuthority::readmit_exact(
            self.schemas.rows.len() as u64 + 1,
            registry,
            contracts,
            &expected,
            image.descriptor_semantics_version(),
        )
        .ok_or_else(|| corrupt("original schema carrier executable admission failed"))?;
        schema
            .retain_readmitted_image(image.clone())
            .map_err(corrupt)?;
        Arc::get_mut(&mut self.schemas)
            .ok_or_else(|| corrupt("source schema graph was shared before admission completed"))?
            .rows
            .insert_cloned_native_for_request(&image.carrier_digest(), &schema, *request)
            .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
        Ok(())
    }
}
