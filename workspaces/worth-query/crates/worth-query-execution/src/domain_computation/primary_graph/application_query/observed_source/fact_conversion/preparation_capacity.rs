use std::mem::size_of;

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

// Variant, native entity coordinates, locator authority and path bookkeeping.
// Variable aspect/field comparisons are priced from initialized source names.
const FIXED_LOCATOR_VISITS: u64 = 12;

#[cfg(test)]
impl<Query> WorthQueryObservedSource<Query> {
    /// Keeps capacity assertions with their private preparation owner. The
    /// integration caller supplies an actually issued source; no proof escapes.
    pub(in crate::domain_computation::primary_graph) fn assert_fact_materialization_capacity_for_test(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
    ) {
        use worth_relational::facade::mvcc::CompanionPreflightBudget;
        let mut funded = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: u64::MAX,
            maximum_preparation_bytes: u64::MAX,
        });
        drop(
            self.admit_fact_materialization(layout, &mut funded, "cross-root source")
                .expect("real source facts fit funded preparation"),
        );
        let work = funded.charged_work();
        let bytes = funded.charged_bytes();
        assert!(work > 0 && bytes > 0);
        for (budget, expected) in [
            (
                CompanionPreflightBudget {
                    maximum_work_visits: work - 1,
                    maximum_preparation_bytes: bytes,
                },
                Kind::WorkBudgetExceeded,
            ),
            (
                CompanionPreflightBudget {
                    maximum_work_visits: work,
                    maximum_preparation_bytes: bytes - 1,
                },
                Kind::PreparationMemoryExceeded,
            ),
        ] {
            let mut short = InvalidationEditAdmission::new(budget);
            let denial = self
                .admit_fact_materialization(layout, &mut short, "cross-root source")
                .err()
                .expect("short materialization admission must refuse");
            assert_eq!(denial.kind(), expected);
        }
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph::application_query::observed_source) fn admit_fact_materialization(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        admission: &mut InvalidationEditAdmission,
        subject: &str,
    ) -> Result<
        super::PreparedSourceFactMaterialization<'_, Query>,
        WorthQuerySourceExpectationDenial,
    > {
        self.validate_completeness(subject)?;
        let footprint = self.source_meaning.footprint();
        let selected = footprint.root_selection.as_deref();
        let selected_count = match selected {
            Some(selection) => selection
                .entities
                .len()
                .checked_add(selection.aspects.len())
                .and_then(|n| n.checked_add(selection.adjacencies.len()))
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded, subject))?,
            None => 0,
        };
        let count = footprint
            .entities
            .len()
            .checked_add(footprint.aspects.len())
            .and_then(|n| n.checked_add(footprint.adjacencies.len()))
            .and_then(|n| n.checked_add(selected_count))
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded, subject))?;
        // Admit the actual first walk before inspecting variable source rows.
        admission
            .charge_external_work(to_u64(count, Kind::WorkBudgetExceeded, subject)?)
            .map_err(|_| denial(Kind::WorkBudgetExceeded, subject))?;

        let field_count = footprint
            .aspects
            .len()
            .checked_add(selected.map_or(0, |selection| selection.aspects.len()))
            .ok_or_else(|| denial(Kind::PreparationMemoryExceeded, subject))?;
        admission
            .admit_read_scratch(mul(
                to_u64(field_count, Kind::PreparationMemoryExceeded, subject)?,
                to_u64(
                    size_of::<super::ValidatedSourceField<'_>>(),
                    Kind::PreparationMemoryExceeded,
                    subject,
                )?,
                subject,
            )?)
            .map_err(|_| denial(Kind::PreparationMemoryExceeded, subject))?;
        let mut fields = Vec::with_capacity(field_count);

        let mut aspect_count = 0u64;
        let mut adjacency_count = 0u64;
        let mut endpoint_count = 0u64;
        let mut copied_name_bytes = 0u64;
        let mut locator_name_work = 0u64;
        for aspect in footprint.aspects.iter().chain(
            selected
                .into_iter()
                .flat_map(|selection| &selection.aspects),
        ) {
            aspect_count = add(aspect_count, 1, subject)?;
            let entity = to_u64(
                aspect.entity_name.len(),
                Kind::PreparationMemoryExceeded,
                subject,
            )?;
            let aspect_name = to_u64(
                aspect.aspect.as_str().len(),
                Kind::PreparationMemoryExceeded,
                subject,
            )?;
            let field = to_u64(
                aspect.field.as_str().len(),
                Kind::PreparationMemoryExceeded,
                subject,
            )?;
            copied_name_bytes = add(
                copied_name_bytes,
                add(aspect_name, field, subject)?,
                subject,
            )?;
            let lookup_work = mul_work(
                btree_comparison_visits(layout.aspect_contract_count(), subject)?,
                add_work(add_work(entity, aspect_name, subject)?, 1, subject)?,
                subject,
            )?;
            let lookup_work = add_work(
                lookup_work,
                add_work(entity, aspect_name, subject)?,
                subject,
            )?;
            admission
                .charge_external_work(lookup_work)
                .map_err(|_| denial(Kind::WorkBudgetExceeded, subject))?;
            admission
                .admit_read_scratch(add(entity, aspect_name, subject)?)
                .map_err(|_| denial(Kind::PreparationMemoryExceeded, subject))?;
            let contract = layout
                .aspect_contract(&aspect.entity_name, &aspect.aspect)
                .ok_or_else(|| denial(Kind::SourceContractMismatch, subject))?;
            let declared_fields = match contract.shape() {
                worth_foundational::facade::AspectShape::Struct(shape) => shape.fields().len(),
                _ => 0,
            };
            let validation_work = add_work(
                32,
                mul_work(
                    to_u64(declared_fields, Kind::WorkBudgetExceeded, subject)?,
                    add_work(field, 1, subject)?,
                    subject,
                )?,
                subject,
            )?;
            admission
                .charge_external_work(validation_work)
                .map_err(|_| denial(Kind::WorkBudgetExceeded, subject))?;
            fields.push(super::ValidatedSourceField::resolve(aspect, contract)?);
            locator_name_work = add_work(
                locator_name_work,
                add_work(aspect_name, field, subject)?,
                subject,
            )?;
        }
        for adjacency in footprint.adjacencies.iter().chain(
            selected
                .into_iter()
                .flat_map(|selection| &selection.adjacencies),
        ) {
            adjacency_count = add(adjacency_count, 1, subject)?;
            endpoint_count = add(
                endpoint_count,
                to_u64(
                    adjacency.endpoints.len(),
                    Kind::PreparationMemoryExceeded,
                    subject,
                )?,
                subject,
            )?;
        }

        let rows = to_u64(count, Kind::PreparationMemoryExceeded, subject)?;
        let fact_slots = mul(
            rows,
            to_u64(size_of::<Fact>(), Kind::PreparationMemoryExceeded, subject)?,
            subject,
        )?;
        let unique_slots = fact_slots;
        let key_slots = mul(
            rows,
            to_u64(
                size_of::<super::SourceFactLocator>(),
                Kind::PreparationMemoryExceeded,
                subject,
            )?,
            subject,
        )?;
        let key_backing = add(key_slots, copied_name_bytes, subject)?;
        let map_nodes = if count == 0 {
            0
        } else {
            add(rows / 5, 2, subject)?
        };
        let map_node_bytes = add(
            mul(
                11,
                to_u64(
                    size_of::<(super::SourceFactLocator, usize)>(),
                    Kind::PreparationMemoryExceeded,
                    subject,
                )?,
                subject,
            )?,
            mul(
                20,
                to_u64(size_of::<usize>(), Kind::PreparationMemoryExceeded, subject)?,
                subject,
            )?,
            subject,
        )?;
        let map_backing = mul(map_nodes, map_node_bytes, subject)?;
        let endpoint_slots = mul(
            mul(endpoint_count, add(adjacency_count, 2, subject)?, subject)?,
            to_u64(
                size_of::<EntityId>(),
                Kind::PreparationMemoryExceeded,
                subject,
            )?,
            subject,
        )?;
        let locator_path_slots = mul(
            aspect_count,
            to_u64(
                size_of::<worth_foundational::facade::FieldKey>(),
                Kind::PreparationMemoryExceeded,
                subject,
            )?,
            subject,
        )?;
        let locator_path_slots = mul(locator_path_slots, 2, subject)?;
        let preparation_bytes = [
            fact_slots,
            unique_slots,
            key_backing,
            map_backing,
            endpoint_slots,
            locator_path_slots,
            copied_name_bytes,
        ]
        .into_iter()
        .try_fold(0u64, |sum, bytes| add(sum, bytes, subject))?;

        let comparison_visits = btree_comparison_visits(count, subject)?;
        // Primitive locators never compare aspect or field text. For a named
        // locator, each comparison is bounded by that inserted key's own text.
        let locator_copy_work = add_work(
            mul_work(rows, FIXED_LOCATOR_VISITS, subject)?,
            locator_name_work,
            subject,
        )?;
        let key_comparison_work = mul_work(comparison_visits, locator_copy_work, subject)?;
        let endpoint_sort_work = mul_work(
            adjacency_count,
            mul_work(endpoint_count, bit_length(endpoint_count), subject)?,
            subject,
        )?;
        let copy_work = [
            copied_name_bytes,
            copied_name_bytes,
            endpoint_count,
            rows,
            locator_copy_work,
            key_comparison_work,
            endpoint_sort_work,
        ]
        .into_iter()
        .try_fold(0u64, |sum, work| add_work(sum, work, subject))?;
        admission
            .charge_external_work(copy_work)
            .map_err(|_| denial(Kind::WorkBudgetExceeded, subject))?;
        admission
            .admit_read_scratch(preparation_bytes)
            .map_err(|_| denial(Kind::PreparationMemoryExceeded, subject))?;
        Ok(super::PreparedSourceFactMaterialization {
            source: self,
            fields,
            fact_count: count,
        })
    }
}

use WorthQuerySourceExpectationDenialKind as Kind;

fn denial(kind: Kind, subject: &str) -> WorthQuerySourceExpectationDenial {
    WorthQuerySourceExpectationDenial::new(kind, subject)
}

fn to_u64(
    value: usize,
    kind: Kind,
    subject: &str,
) -> Result<u64, WorthQuerySourceExpectationDenial> {
    u64::try_from(value).map_err(|_| denial(kind, subject))
}

fn add(left: u64, right: u64, subject: &str) -> Result<u64, WorthQuerySourceExpectationDenial> {
    left.checked_add(right)
        .ok_or_else(|| denial(Kind::PreparationMemoryExceeded, subject))
}

fn mul(left: u64, right: u64, subject: &str) -> Result<u64, WorthQuerySourceExpectationDenial> {
    left.checked_mul(right)
        .ok_or_else(|| denial(Kind::PreparationMemoryExceeded, subject))
}

fn add_work(
    left: u64,
    right: u64,
    subject: &str,
) -> Result<u64, WorthQuerySourceExpectationDenial> {
    left.checked_add(right)
        .ok_or_else(|| denial(Kind::WorkBudgetExceeded, subject))
}

fn mul_work(
    left: u64,
    right: u64,
    subject: &str,
) -> Result<u64, WorthQuerySourceExpectationDenial> {
    left.checked_mul(right)
        .ok_or_else(|| denial(Kind::WorkBudgetExceeded, subject))
}

fn bit_length(value: u64) -> u64 {
    u64::from(u64::BITS - value.leading_zeros()).max(1)
}

fn btree_comparison_visits(
    entries: usize,
    subject: &str,
) -> Result<u64, WorthQuerySourceExpectationDenial> {
    let mut levels = 1u64;
    let mut threshold = 11usize;
    while entries >= threshold {
        levels = add_work(levels, 1, subject)?;
        let Some(next) = threshold
            .checked_add(1)
            .and_then(|n| n.checked_mul(6))
            .and_then(|n| n.checked_sub(1))
        else {
            break;
        };
        threshold = next;
    }
    let keys = to_u64(entries.min(11), Kind::WorkBudgetExceeded, subject)?;
    mul_work(levels, keys.max(1), subject)
}
