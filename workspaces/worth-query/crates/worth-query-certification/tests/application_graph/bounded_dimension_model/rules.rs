//! The two bounded-dimension rule implementations the Relational invariant
//! owner actually runs.
//!
//! Each rule reads the proposed dimension of every touched part and decides on
//! it. Neither knows anything about programs, branches or activation: whether a
//! rule is asked at all is decided for it, and what it answers is decided only
//! by the value in front of it.

use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantExecutionError, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantPreparationError, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantVerdict,
};

use super::schema::{BoundedDimensionSchema, Part, PartDimensionField};

/// The greatest dimension `bounded-dimension-v1` admits.
pub const V1_CEILING: u64 = 10;
/// The least dimension `bounded-dimension-v2` admits.
pub const V2_FLOOR: u64 = 5;
/// The greatest dimension `bounded-dimension-v2` admits.
pub const V2_CEILING: u64 = 20;

type PartDimensions = Vec<WorthQueryApplicationInvariantEntity<BoundedDimensionSchema, Part>>;
type PartDimensionBinding =
    WorthQueryApplicationInvariantFieldBinding<BoundedDimensionSchema, Part, u64>;

/// `bounded-dimension-v1`: a part may not be wider than [`V1_CEILING`].
pub struct BoundedDimensionV1Rule {
    dimension: PartDimensionBinding,
}

/// `bounded-dimension-v2`: a part must be between [`V2_FLOOR`] and
/// [`V2_CEILING`] inclusive.
pub struct BoundedDimensionV2Rule {
    dimension: PartDimensionBinding,
}

pub fn resolve_first_rule(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, BoundedDimensionSchema>,
) -> Result<BoundedDimensionV1Rule, String> {
    Ok(BoundedDimensionV1Rule {
        dimension: resolve_dimension(resolver)?,
    })
}

pub fn resolve_second_rule(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, BoundedDimensionSchema>,
) -> Result<BoundedDimensionV2Rule, String> {
    Ok(BoundedDimensionV2Rule {
        dimension: resolve_dimension(resolver)?,
    })
}

impl WorthQueryApplicationInvariantRule<BoundedDimensionSchema> for BoundedDimensionV1Rule {
    type Scope = PartDimensions;

    fn prepare_scope(
        &self,
        planner: &mut worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner<
            '_,
            '_,
            BoundedDimensionSchema,
        >,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        plan_touched_parts(planner, &self.dimension)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, BoundedDimensionSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        for dimension in proposed_dimensions(context, &self.dimension, scope)? {
            if !matches!(dimension, Some(value) if value <= V1_CEILING) {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

impl WorthQueryApplicationInvariantRule<BoundedDimensionSchema> for BoundedDimensionV2Rule {
    type Scope = PartDimensions;

    fn prepare_scope(
        &self,
        planner: &mut worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner<
            '_,
            '_,
            BoundedDimensionSchema,
        >,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        plan_touched_parts(planner, &self.dimension)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, BoundedDimensionSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        for dimension in proposed_dimensions(context, &self.dimension, scope)? {
            if !matches!(dimension, Some(value) if (V2_FLOOR..=V2_CEILING).contains(&value)) {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

fn resolve_dimension(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, BoundedDimensionSchema>,
) -> Result<PartDimensionBinding, String> {
    resolver
        .typed_field(PartDimensionField::reference())
        .ok_or_else(|| "the part dimension field is not installed".to_owned())
}

/// Declares the touched parts and the one field either rule reads.
fn plan_touched_parts(
    planner: &mut worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner<
        '_,
        '_,
        BoundedDimensionSchema,
    >,
    dimension: &PartDimensionBinding,
) -> Result<PartDimensions, WorthQueryApplicationInvariantPreparationError> {
    let proposed = planner.proposed();
    let parts = proposed
        .touched_entities(dimension)
        .map_err(|error| WorthQueryApplicationInvariantPreparationError::new(error.to_string()))?;
    for part in &parts {
        proposed.field(dimension, part).map_err(|error| {
            WorthQueryApplicationInvariantPreparationError::new(error.to_string())
        })?;
    }
    Ok(parts)
}

/// Reads the dimension each touched part would carry if this candidate landed.
fn proposed_dimensions(
    context: &WorthQueryApplicationInvariantContext<'_, '_, BoundedDimensionSchema>,
    dimension: &PartDimensionBinding,
    scope: &PartDimensions,
) -> Result<Vec<Option<u64>>, WorthQueryApplicationInvariantExecutionError> {
    let proposed = context.proposed();
    scope
        .iter()
        .map(|part| {
            proposed.field(dimension, part).map_err(|error| {
                WorthQueryApplicationInvariantExecutionError::new(error.to_string())
            })
        })
        .collect()
}
