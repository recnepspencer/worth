//! The one owned-schema lowering owner, shared by capture, readmission and Home.
use super::*;

pub(super) enum RetainedContracts<'a> {
    Extend(Option<&'a RelationalBranchRootSchemaAuthority>),
    Exact(Vec<AspectContract>),
}

impl RelationalBranchRootSchemaAuthority {
    pub(super) fn compile_owned(
        allocation_id: u64,
        mut registry: RelationalSchemaRegistry,
        expected: &SchemaAuthoritySnapshot,
        descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
        retained: RetainedContracts<'_>,
        funding: Option<Arc<compiled_retention::SchemaCompilationFunding>>,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<Arc<Self>, crate::execution::RelationalExecutionDenialCause> {
        registry
            .content_formats
            .bind_carried_snapshot(&expected.content_formats);
        let aspect_plans = crate::schema::lower_aspect_plans_for_compilation(&registry, control)?;
        let retained_aspect_contracts = match retained {
            RetainedContracts::Extend(previous) => {
                retain_aspect_contracts(previous, &aspect_plans, control)?
            }
            RetainedContracts::Exact(contracts) => contracts,
        };
        debug_assert!(
            /* Proof: Every entry is lowered directly from the accepted immutable registry; the private plan table cannot be supplied independently. */
            plans_match_snapshot_for_compilation(&aspect_plans, expected, control)?
        );
        let authority_digest =
            crate::schema::data::schema_authority_snapshot_digest_for_compilation(
                expected, control,
            )?;
        control.checkpoint()?;
        Ok(Arc::new(Self {
            allocation_id,
            allocation_bytes: Default::default(),
            primary_schema_version_id: expected.primary_schema_version_id,
            authority_digest,
            continuity_authority: expected.clone(),
            registry: Arc::new(registry),
            aspect_plans: Arc::new(aspect_plans),
            retained_aspect_contracts: Arc::new(super::RetainedContracts::new(
                retained_aspect_contracts,
            )),
            descriptor_semantics_version,
            relation_integrity_revisions: snapshot_match::retain_revisions(expected, control)?,
            recorded_image: Default::default(),
            _funding: funding,
        }))
    }
}
