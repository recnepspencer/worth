//! Conditional owned-selection retirement is inert transported meaning.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyOwnedSelectionRetirement as Rule;
pub(super) fn write(output: &mut dyn BinaryEncodingSink, rules: &[Rule]) -> Result<(), Denial> {
    write_sequence(output, rules, |output, rule| {
        for part in [
            rule.trigger.entity(),
            rule.trigger.aspect(),
            rule.trigger.field(),
            rule.binding.as_str(),
            &rule.member,
            &rule.owners_relation,
            &rule.scope,
            &rule.owner,
            &rule.selection_relation,
            &rule.selected,
        ] {
            output.text(part)?;
        }
        super::super::foundational_aspect::write_scalar_type(output, rule.scalar_family)?;
        output.u64(
            u64::try_from(rule.maximum_owners.get())
                .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        )
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<Rule>, Denial> {
    // Ten text-length prefixes, the scalar-family tag and the nonzero u64 bound.
    decode_sequence(input, budget, 50, |input, _| {
        let trigger =
            ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
        let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.into());
        let member = input.text()?.into();
        let owners_relation = input.text()?.into();
        let scope = input.text()?.into();
        let owner = input.text()?.into();
        let selection_relation = input.text()?.into();
        let selected = input.text()?.into();
        let scalar_family = super::super::foundational_aspect::decode_scalar_type(input)?;
        let maximum_owners = std::num::NonZeroUsize::new(
            usize::try_from(input.u64()?).map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        )
        .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
        Ok(Rule {
            trigger,
            binding,
            scalar_family,
            member,
            owners_relation,
            scope,
            owner,
            selection_relation,
            selected,
            maximum_owners,
        })
    })
}
