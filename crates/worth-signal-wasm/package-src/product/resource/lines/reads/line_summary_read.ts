import { readLineDiagnosticsSummary } from "./line_diagnostics_summary_read.js";
import { readLineDownload } from "./line_download_read.js";
import { readLineProcessing } from "./line_processing_read.js";
import { readLineRequest } from "./line_request_read.js";
import { readLineUpload } from "./line_upload_read.js";

function readLineSummary(materialization, options = undefined) {
  const diagnostics = readLineDiagnosticsSummary(materialization, options);
  return Object.freeze({
    current: diagnostics.current,
    request: readLineRequest(materialization),
    processing: readLineProcessing(materialization),
    upload: readLineUpload(materialization),
    download: readLineDownload(materialization),
    diagnostics,
    explainability: diagnostics.explainability,
  });
}

function captureLineSummaryInputs(materialization) {
  const binding = materialization.binding;
  binding.valueSignal();
  binding.processingSignal();
  binding.uploadSignal();
  binding.downloadSignal();
  binding.statusSignal();
  binding.freshnessSignal();
  binding.diagnosticsSignal();
}

function readReactiveLineSummary(materialization) {
  captureLineSummaryInputs(materialization);
  return readLineSummary(materialization, { includeExplainability: false });
}

export { readLineSummary, readReactiveLineSummary, captureLineSummaryInputs };
