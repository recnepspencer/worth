import { canonicalizeLineValue } from "../lines/state/line_value_canonical_json.js";

// The digest identifies a response by the JSON it serializes to, so two
// payloads that reach the wire identically digest identically. Responses are
// canonicalized before they reach the plan (see `canonicalizeLineValue`), so
// for a committed response this is a pure re-serialization; a value JSON
// cannot represent is refused there with the reason.
function createMutationResponsePayloadDigest(value) {
  return [
    "mutation-response-payload",
    JSON.stringify(canonicalizeLineValue(value, "$response")),
  ].join("|");
}

export { createMutationResponsePayloadDigest };
