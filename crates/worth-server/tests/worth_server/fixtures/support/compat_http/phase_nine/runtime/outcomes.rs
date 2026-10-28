use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerBinaryDownload, WorthServerBinaryDownloadOutcome,
    WorthServerCompatibilityInspection, WorthServerCompatibilityRead,
    WorthServerCompatibilityUpload, WorthServerCompatibilityUploadOutcome,
    WorthServerQueryHandoffDenial,
};

pub(crate) fn compat_read_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<WorthServerCompatibilityRead>,
) -> WorthServerCompatibilityRead {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility read success, got {other:?}"),
    }
}

pub(crate) fn compat_inspection_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<
        WorthServerCompatibilityInspection,
    >,
) -> WorthServerCompatibilityInspection {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility inspection success, got {other:?}"),
    }
}

pub(crate) fn compat_upload_success(
    outcome: WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload>,
) -> WorthServerCompatibilityUpload {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility upload success, got {other:?}"),
    }
}

pub(crate) fn compat_upload_denied(
    outcome: WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload>,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility upload denial, got {other:?}"),
    }
}

pub(crate) fn compat_download_success(
    outcome: WorthServerBinaryDownloadOutcome<WorthServerBinaryDownload>,
) -> WorthServerBinaryDownload {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected binary download success, got {other:?}"),
    }
}

pub(crate) fn compat_download_denied(
    outcome: WorthServerBinaryDownloadOutcome<WorthServerBinaryDownload>,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected binary download denial, got {other:?}"),
    }
}
