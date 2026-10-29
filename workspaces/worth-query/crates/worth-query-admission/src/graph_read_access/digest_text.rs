use crate::admission_digest::AdmittedHashStop;
use std::fmt::{self, Write};

struct CountedText {
    bytes: Option<usize>,
}

impl Write for CountedText {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.bytes = self.bytes.and_then(|bytes| bytes.checked_add(value.len()));
        self.bytes.map(|_| ()).ok_or(fmt::Error)
    }
}

pub(crate) enum AdmittedDigestTextStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

impl<Stop> From<AdmittedHashStop<Stop>> for AdmittedDigestTextStop<Stop> {
    fn from(stop: AdmittedHashStop<Stop>) -> Self {
        match stop {
            AdmittedHashStop::Admission(stop) => Self::Admission(stop),
            AdmittedHashStop::AccountingOverflow => Self::AccountingOverflow,
        }
    }
}

/// The same writer renders ordinary and admitted digest text. The first pass
/// counts bytes without materializing intermediate strings; the second writes
/// into exactly the preclaimed backing.
pub(crate) fn admitted_digest_text<Stop>(
    count_work: u64,
    mut render: impl FnMut(&mut dyn Write) -> fmt::Result,
    mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<String, AdmittedDigestTextStop<Stop>> {
    let mut admission = None;
    let mut charge = |work: u64, bytes: u64| -> Result<(), ()> {
        match admit(work, bytes) {
            Ok(()) => Ok(()),
            Err(stop) => {
                admission = Some(stop);
                Err(())
            }
        }
    };
    match drive_admitted_digest_text(count_work, &mut render, &mut charge) {
        Ok(text) => Ok(text),
        Err(DigestDriveStop::AccountingOverflow) => Err(AdmittedDigestTextStop::AccountingOverflow),
        Err(DigestDriveStop::Admission) => Err(AdmittedDigestTextStop::Admission(
            admission.expect("refused digest admission stores its stop"),
        )),
    }
}

enum DigestDriveStop {
    Admission,
    AccountingOverflow,
}

#[inline(never)]
fn drive_admitted_digest_text(
    count_work: u64,
    render: &mut dyn FnMut(&mut dyn Write) -> fmt::Result,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), ()>,
) -> Result<String, DigestDriveStop> {
    admit(count_work, 0).map_err(|_| DigestDriveStop::Admission)?;
    let mut counted = CountedText { bytes: Some(0) };
    render(&mut counted).map_err(|_| DigestDriveStop::AccountingOverflow)?;
    let bytes = counted.bytes.ok_or(DigestDriveStop::AccountingOverflow)?;
    let cost = u64::try_from(bytes).map_err(|_| DigestDriveStop::AccountingOverflow)?;
    admit(cost, cost).map_err(|_| DigestDriveStop::Admission)?;
    let mut output = String::with_capacity(bytes);
    render(&mut output).expect("String formatting cannot fail");
    debug_assert_eq!(output.len(), bytes);
    Ok(output)
}

pub(crate) fn admitted_text_clone<Stop>(
    value: &str,
    mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<String, AdmittedDigestTextStop<Stop>> {
    let bytes =
        u64::try_from(value.len()).map_err(|_| AdmittedDigestTextStop::AccountingOverflow)?;
    admit(bytes, bytes).map_err(AdmittedDigestTextStop::Admission)?;
    Ok(value.to_owned())
}
