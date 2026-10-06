use std::fmt::{self, Write};

use super::{option_name, WorthQueryGraphReadAccessRequirementRow};

impl WorthQueryGraphReadAccessRequirementRow {
    pub(crate) fn equality_inline_bytes(&self) -> Option<u64> {
        // Schema digests participate in Eq even when diagnostic digest text
        // omits them. Include each nested authority's complete initialized
        // carrier; text widths are charged separately by the caller.
        let predicates = self
            .predicate_field_authorities
            .len()
            .checked_mul(size_of::<super::WorthQueryGraphReadPredicateFieldAuthority>())?;
        let ordering = self
            .ordering_field_authorities
            .len()
            .checked_mul(size_of::<super::WorthQueryGraphReadOrderingFieldAuthority>())?;
        let relations = self
            .operation_capability_requirement
            .as_ref()
            .map_or(0, |value| value.matched_relations().len())
            .checked_mul(size_of::<String>())?;
        let bytes = size_of::<Self>()
            .checked_add(predicates)?
            .checked_add(ordering)?
            .checked_add(relations)?;
        u64::try_from(bytes).ok()
    }

    pub(crate) fn digest_visit_work(&self) -> Option<u64> {
        let predicates = u64::try_from(self.predicate_field_authorities.len()).ok()?;
        let ordering = u64::try_from(self.ordering_field_authorities.len()).ok()?;
        let relations = self
            .operation_capability_requirement
            .as_ref()
            .map_or(0, |value| value.matched_relations().len());
        24_u64
            .checked_add(predicates)?
            .checked_add(ordering)?
            .checked_add(u64::try_from(relations).ok()?)
    }

    pub(crate) fn write_semantic_slot_key(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "slot:{}:{}:",
            self.kind.as_str(),
            option_name(self.relation_direction.as_ref())
        )?;
        write_option_usize(output, self.relation_depth)?;
        write!(
            output,
            ":{}:{}:{}:{}:{}:{}:",
            option_name(self.fanout_posture.as_ref()),
            option_name(self.predicate_family.as_ref()),
            option_name(self.ordering_posture.as_ref()),
            option_name(self.traversal_operator.as_ref()),
            option_name(self.lifecycle_class.as_ref()),
            option_name(self.result_pressure.as_ref()),
        )?;
        match &self.operation_capability_requirement {
            Some(value) => value.write_digest_part(output),
            None => output.write_str("none"),
        }
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "requirement:{}:{}:{}:",
            self.kind.as_str(),
            self.rebuild_basis.as_str(),
            self.relation_name.as_deref().unwrap_or("none"),
        )?;
        match &self.relation_authority {
            Some(value) => value.write_digest_part(output)?,
            None => output.write_str("none")?,
        }
        write!(
            output,
            ":{}:",
            option_name(self.relation_direction.as_ref())
        )?;
        write_option_usize(output, self.relation_depth)?;
        write!(
            output,
            ":{}:{}:",
            option_name(self.fanout_posture.as_ref()),
            option_name(self.predicate_family.as_ref()),
        )?;
        for (index, value) in self.predicate_field_authorities.iter().enumerate() {
            if index > 0 {
                output.write_str(",")?;
            }
            value.write_digest_part(output)?;
        }
        write!(output, ":{}:", option_name(self.ordering_posture.as_ref()))?;
        for (index, value) in self.ordering_field_authorities.iter().enumerate() {
            if index > 0 {
                output.write_str(",")?;
            }
            value.write_digest_part(output)?;
        }
        write!(
            output,
            ":{}:{}:{}:",
            option_name(self.traversal_operator.as_ref()),
            option_name(self.lifecycle_class.as_ref()),
            option_name(self.result_pressure.as_ref()),
        )?;
        match &self.operation_capability_requirement {
            Some(value) => value.write_digest_part(output)?,
            None => output.write_str("none")?,
        }
        write!(
            output,
            ":{}:{}:{}:",
            self.invalidation_basis.as_str(),
            self.complexity_contract.as_str(),
            self.memory_estimate_basis.as_str(),
        )?;
        write_option_usize(output, self.maximum_cardinality)
    }
}

fn write_option_usize(output: &mut dyn Write, value: Option<usize>) -> fmt::Result {
    match value {
        Some(value) => write!(output, "{value}"),
        None => output.write_str("none"),
    }
}
