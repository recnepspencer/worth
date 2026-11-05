import { readLineDescriptor } from "./line_descriptor_read.js";
import { readLineDownload } from "./line_download_read.js";
import { readLineDiagnostics } from "./line_diagnostics_read.js";
import { readLineDiagnosticsSummary } from "./line_diagnostics_summary_read.js";
import { readLineHistory } from "./line_history_read.js";
import { readLineProcessing } from "./line_processing_read.js";
import { readLineRequest } from "./line_request_read.js";
import { readLineSummary } from "./line_summary_read.js";
import { readLineUpload } from "./line_upload_read.js";
import { readLineValue } from "./line_value_read.js";
import { requireCurrentMaterialization } from "../state/line_handle_helpers.js";
import { requireActiveLine } from "../actions/line_activity_guard.js";

export function createLineObservationBindings(lineBacking, handles) {
  return {
    value() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "value");
      return readLineValue(materialization);
    },
    signal() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "signal");
      return handles.value();
    },
    descriptor() {
      const materialization = requireCurrentMaterialization(lineBacking);
      return readLineDescriptor(materialization);
    },
    request() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "request");
      return readLineRequest(materialization);
    },
    summary() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "summary");
      return readLineSummary(materialization);
    },
    summarySignal() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "summarySignal");
      return handles.summary();
    },
    download() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "download");
      return readLineDownload(materialization);
    },
    history() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "history");
      return readLineHistory(materialization);
    },
    processing() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "processing");
      return readLineProcessing(materialization);
    },
    upload() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "upload");
      return readLineUpload(materialization);
    },
    diagnostics() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "diagnostics");
      return readLineDiagnostics(materialization);
    },
    diagnosticsSummary() {
      const materialization = requireCurrentMaterialization(lineBacking);
      requireActiveLine(materialization, "diagnosticsSummary");
      return readLineDiagnosticsSummary(materialization);
    },
  };
}
