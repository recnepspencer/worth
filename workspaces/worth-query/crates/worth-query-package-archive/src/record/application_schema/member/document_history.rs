//! Member51 carries declared history meaning, never live document custody.
use super::super::{query_binding_reference, wire_vocabulary};
use super::*;
use crate::record::{
    application_query::disclosure,
    sequence::{decode_sequence, write_sequence},
};
use worth_query_declaration::facade::application_history::{
    ApplicationHistoryModelDomain as Domain, ApplicationHistoryNavigationDeclaration,
    ApplicationHistoryOperation, ApplicationHistoryRetentionDeclaration as Retention,
    ErasedApplicationHistoryDefinition as Definition,
    WorthQueryPortableApplicationHistoryNavigationParts as NavigationParts,
    WorthQueryPortableApplicationHistoryParts as Parts,
    WorthQueryPortableApplicationHistoryRetentionParts as RetentionParts,
};

pub(super) fn require_shape(member: &ApplicationSchemaMember) -> Result<(), Denial> {
    if let ApplicationSchemaMember::ApplicationHistory { definition } = member {
        definition
            .validate()
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    }
    Ok(())
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &Definition,
) -> Result<(), Denial> {
    let parts = definition.parts();
    let retention = parts.window.is_some() || parts.bookmarks.is_some();
    output.u16(if retention {
        3
    } else if parts.navigation.is_some() {
        2
    } else {
        1
    })?;
    output.u16(match parts.model_domain {
        Domain::WholeComposite => 1,
        Domain::ScopeEntityRecord => 2,
    })?;
    output.text(&parts.identity)?;
    wire_vocabulary::write_type_identity(output, &parts.history_type)?;
    query_binding_reference::write(output, &parts.access)?;
    disclosure::write(output, &parts.metadata_disclosure)?;
    // Operation order is authored meaning, not a canonical-sort opportunity.
    write_sequence(output, &parts.operations, |output, operation| {
        output.text(&operation.binding)?;
        output.text(&operation.label)
    })?;
    if retention {
        output.u16(
            u16::from(parts.navigation.is_some())
                | (u16::from(parts.window.is_some()) << 1)
                | (u16::from(parts.bookmarks.is_some()) << 2),
        )?;
    }
    if let Some(navigation) = &parts.navigation {
        let navigation = navigation.parts();
        output.text(&navigation.binding)?;
        output.text(&navigation.capability)?;
        wire_vocabulary::write_type_identity(output, &navigation.capability_type)?;
        output.text(&navigation.operation)?;
        wire_vocabulary::write_type_identity(output, &navigation.input_type)?;
    }
    for policy in [&parts.window, &parts.bookmarks].into_iter().flatten() {
        let policy = policy.parts();
        output.text(&policy.binding)?;
        output.text(&policy.capability)?;
        wire_vocabulary::write_type_identity(output, &policy.capability_type)?;
        output.text(&policy.operation)?;
        wire_vocabulary::write_type_identity(output, &policy.input_type)?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    let version = input.u16()?;
    if !matches!(version, 1..=3) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    budget.require_nesting_depth(2)?;
    let model_domain = match input.u16()? {
        1 => Domain::WholeComposite,
        2 => Domain::ScopeEntityRecord,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let identity = input.text()?.to_owned();
    let history_type = wire_vocabulary::decode_type_identity(input)?;
    let access = query_binding_reference::decode(input)?;
    let metadata_disclosure = disclosure::decode(input, budget)?;
    let operations = decode_sequence(input, budget, 8, |input, _| {
        Ok(ApplicationHistoryOperation {
            binding: input.text()?.to_owned(),
            label: input.text()?.to_owned(),
        })
    })?;
    let flags = if version == 3 {
        let flags = input.u16()?;
        if flags & !7 != 0 || flags & 6 == 0 {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        flags
    } else {
        u16::from(version == 2)
    };
    let navigation = if flags & 1 != 0 {
        Some(
            ApplicationHistoryNavigationDeclaration::from_untrusted_parts(NavigationParts {
                binding: input.text()?.to_owned(),
                capability: input.text()?.to_owned(),
                capability_type: wire_vocabulary::decode_type_identity(input)?,
                operation: input.text()?.to_owned(),
                input_type: wire_vocabulary::decode_type_identity(input)?,
            }),
        )
    } else {
        None
    };
    let window = if flags & 2 != 0 {
        Some(decode_retention(input)?)
    } else {
        None
    };
    let bookmarks = if flags & 4 != 0 {
        Some(decode_retention(input)?)
    } else {
        None
    };
    let member = ApplicationSchemaMember::ApplicationHistory {
        definition: Definition::from_untrusted_parts(Parts {
            identity,
            history_type,
            access,
            metadata_disclosure,
            model_domain,
            operations,
            navigation,
            window,
            bookmarks,
        }),
    };
    require_shape(&member)?;
    Ok(member)
}

fn decode_retention(input: &mut BinaryInput<'_>) -> Result<Retention, Denial> {
    Ok(Retention::from_untrusted_parts(RetentionParts {
        binding: input.text()?.to_owned(),
        capability: input.text()?.to_owned(),
        capability_type: wire_vocabulary::decode_type_identity(input)?,
        operation: input.text()?.to_owned(),
        input_type: wire_vocabulary::decode_type_identity(input)?,
    }))
}
