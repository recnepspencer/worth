//! Actual record/content associations own byte custody; identity alone does not.
use crate::internal_port::content::attachments::RelationalContentAttachmentsOwnerPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;
pub mod reconstruction;
use super::{
    control, field_references, frame_admission::ContentFrameAdmission,
    resolution::ContentResolution, RelationalContentAttachments, RelationalContentDenial as Denial,
    RelationalContentFrame,
};
use crate::{
    identity::data::VersionId, schema::data::RelationalSchemaRegistry,
    storage::overlay::CurrentPartitionAccess, transactions::data::primitives::RecordRef,
};
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest};
use worth_foundational::facade::ContentRefId;

type Occurrence = (ContentRefId, Option<RecordRef>);
type Identities = ExecutionOrderedMap<ContentRefId, ()>;
#[derive(Clone, Debug, Default)]
pub struct ContentDirectory {
    occurrences: ExecutionOrderedMap<Occurrence, RelationalContentFrame>,
    records: ExecutionOrderedMap<RecordRef, Identities>,
}
impl ContentDirectory {
    pub fn find(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
    ) -> Result<Option<RelationalContentFrame>, Denial> {
        control::run(execution, |control| self.find_controlled(control, identity))
    }
    pub fn find_controlled(
        &self,
        control: &mut super::RelationalContentControl<'_, '_, '_>,
        identity: ContentRefId,
    ) -> Result<Option<RelationalContentFrame>, Denial> {
        let height = 2 * (usize::BITS - self.occurrences.len().leading_zeros()) as u64 + 2;
        control.checkpoint(
            height * (size_of::<Occurrence>() + size_of::<RelationalContentFrame>()) as u64,
        )?;
        match self.occurrences.first_at_or_after(&(identity, None)) {
            Some(((found, Some(_)), frame)) if *found == identity => Ok(Some(frame.clone())),
            Some(((found, None), _)) if *found == identity => Err(Denial::EncodingInconsistency),
            _ => Ok(None),
        }
    }
    pub fn read(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        expected: &super::RelationalContentFormat,
    ) -> Result<RelationalContentFrame, Denial> {
        let frame = self
            .find(execution, identity)?
            .ok_or(Denial::MissingContent)?;
        require_format(execution, &frame, expected)?;
        Ok(frame)
    }
    pub fn prepare(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        schema: &RelationalSchemaRegistry,
        attachments: &RelationalContentAttachments,
        state: &dyn crate::storage::overlay::CurrentPartitionAccess,
        changed: &[RecordRef],
        version: crate::identity::data::VersionId,
        source: Option<&Self>,
    ) -> Result<Self, Denial> {
        let staged =
            ContentFrameAdmission::admit(execution, schema.owner_content_formats(), attachments)?;
        let mut next = self.clone();
        let mut resolved = ExecutionOrderedMap::default();
        let mut resolution = ContentResolution::new(
            schema.owner_content_formats(),
            &staged,
            self,
            source,
            &mut resolved,
        );
        for record in changed {
            next.replace_record(execution, schema, state, version, record, &mut resolution)?;
        }
        Ok(next)
    }
    fn replace_record(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        schema: &RelationalSchemaRegistry,
        state: &dyn CurrentPartitionAccess,
        version: VersionId,
        record: &RecordRef,
        resolution: &mut ContentResolution<'_>,
    ) -> Result<(), Denial> {
        self.remove_record(execution, record)?;
        let mut identities = Identities::default();
        field_references::visit(
            execution,
            schema,
            state,
            version,
            record,
            &mut |identity, format| {
                let frame = resolution.resolve(execution, identity)?;
                require_format(execution, &frame, format)?;
                visit_directory(execution, identities.len())?;
                if !identities.contains_key(&identity) {
                    identities
                        .insert_cloned_native_for_request(&identity, &(), *execution)
                        .map_err(map_denial)?;
                    visit_directory(execution, self.occurrences.len())?;
                    self.occurrences
                        .insert_cloned_native_for_request(
                            &(identity, Some(record.clone())),
                            &frame,
                            *execution,
                        )
                        .map_err(map_denial)?;
                }
                Ok(())
            },
        )?;
        if !identities.is_empty() {
            visit_directory(execution, self.records.len())?;
            self.records
                .insert_cloned_native_for_request(record, &identities, *execution)
                .map_err(map_denial)?;
        }
        Ok(())
    }
    fn remove_record(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        record: &RecordRef,
    ) -> Result<(), Denial> {
        visit_directory(execution, self.records.len())?;
        let Some(prior) = self.records.get(record).cloned() else {
            return Ok(());
        };
        for identity in prior.keys() {
            visit_directory(execution, self.occurrences.len())?;
            if !self
                .occurrences
                .remove_for_request(&(*identity, Some(record.clone())), *execution)
                .map_err(map_denial)?
            {
                return Err(Denial::EncodingInconsistency);
            }
        }
        visit_directory(execution, self.records.len())?;
        if !self
            .records
            .remove_for_request(record, *execution)
            .map_err(map_denial)?
        {
            return Err(Denial::EncodingInconsistency);
        }
        Ok(())
    }
    pub fn publication_records(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        prior: &Self,
        changed: &[RecordRef],
    ) -> Result<RelationalContentAttachments, Denial> {
        let mut selected =
            ExecutionOrderedMap::<ContentRefId, super::RelationalContentRecord>::default();
        for record in changed {
            visit_directory(execution, self.records.len())?;
            let Some(identities) = self.records.get(record) else {
                continue;
            };
            for identity in identities.keys() {
                if prior.find(execution, *identity)?.is_some() {
                    continue;
                }
                visit_directory(execution, selected.len())?;
                if !selected.contains_key(identity) {
                    // This includes genuine historical custody restored through Source,
                    // even when the transaction did not encode a new attachment.
                    let frame = self
                        .find(execution, *identity)?
                        .ok_or(Denial::EncodingInconsistency)?;
                    selected
                        .insert_cloned_native_for_request(identity, &frame.record(), *execution)
                        .map_err(map_denial)?;
                }
            }
        }
        let mut records = RelationalContentAttachments::default();
        for record in selected.values() {
            records.append_record(execution, record)?;
        }
        Ok(records)
    }

    pub fn unreferenced_attachment(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        attachments: &RelationalContentAttachments,
    ) -> Result<Option<ContentRefId>, Denial> {
        control::run(execution, |control| {
            control.checkpoint(0)?;
            for record in attachments.records() {
                control.checkpoint(1)?;
                if self.find_controlled(control, record.identity())?.is_none() {
                    return Ok(Some(record.identity()));
                }
            }
            Ok(None)
        })
    }
}
pub fn require_format(
    execution: &ExecutionRequest<'_, '_>,
    frame: &RelationalContentFrame,
    expected: &super::RelationalContentFormat,
) -> Result<(), Denial> {
    frame.require_contract(
        execution,
        expected.descriptor().name(),
        expected.descriptor().version(),
        expected.maximum_frame_bytes(),
    )
}

pub fn visit_directory(execution: &ExecutionRequest<'_, '_>, count: usize) -> Result<(), Denial> {
    let height = 2 * (usize::BITS - count.leading_zeros()) as u64 + 2;
    control::run(execution, |control| {
        control.checkpoint(
            height * (size_of::<Occurrence>() + size_of::<RelationalContentFrame>()) as u64,
        )
    })
}

pub fn map_denial(cause: ExecutionOrderedMapDenial) -> Denial {
    use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
    use worth_execution::ExecutionAllocationDenialClass as Class;
    Denial::Execution(match cause {
        ExecutionOrderedMapDenial::Request(cause) => cause.into(),
        ExecutionOrderedMapDenial::CapacityOverflow => Cause::ChargedBytesOverflow,
        ExecutionOrderedMapDenial::Allocation(cause) => match cause.classify() {
            Class::Request(cause) => cause.into(),
            Class::Physical(_) => Cause::AllocationFailed,
        },
    })
}
