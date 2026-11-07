//! One admitted schema graph is shared by source roles and selected roots.
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::region_directory::corrupt;
use crate::durability::data::{RecoveryPlan, RelationalRecoveryStop};
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
        plan: &mut RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<Self>, RelationalRecoveryStop> {
        let Some(original) = &plan.original_source else {
            return Ok(None);
        };
        let directory_copies = plan.checkpoint.as_ref().map_or(0, |checkpoint| {
            usize::from(checkpoint.directory.is_some())
                + usize::from(checkpoint.canonical_directory.is_some())
        });
        let mut graph = Self {
            original: original.clone(),
            schemas: ReadmittedSourceSchemaCatalog::prepare(request, directory_copies)?,
        };
        for image in &original.current.schemas {
            ReadmittedSourceSchemaCatalog::insert_image(
                &mut graph.schemas,
                image,
                &plan.config.schema.registry,
                request,
            )?;
        }
        if let Some(checkpoint) = &plan.checkpoint {
            for image in &checkpoint.branch_root_schema_images {
                ReadmittedSourceSchemaCatalog::insert_image(
                    &mut graph.schemas,
                    image,
                    &plan.config.schema.registry,
                    request,
                )?;
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
                ReadmittedSourceSchemaCatalog::insert_image(
                    &mut graph.schemas,
                    image,
                    &plan.config.schema.registry,
                    request,
                )?;
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
        if let Some(checkpoint) = &mut plan.checkpoint {
            for directory in checkpoint
                .directory
                .iter_mut()
                .chain(checkpoint.canonical_directory.iter_mut())
            {
                // A plan clone retains its original immutable directory. The
                // attempt owns only this fixed-size binding; intrinsic manifest,
                // history and actual selected-read custody stay shared.
                *directory = crate::durability::data::NativeCheckpointDirectoryEdge::new(
                    directory.with_source_schemas(graph.schemas.clone(), request)?,
                );
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
}
