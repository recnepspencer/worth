# CountedSource fixture correction boundary and plan

CountedSource owns immutable, producer-written member bytes for its full Arc
lifetime; region cells/record-history custody keep that source alive. The new
request-funded capture cannot fall back to legacy retain_members. Implement the
actual provider method with try_new_from_source_for_request and a unit payload,
as in ImmutableCheckpointMemberSource. Implement read_member_owned against the
same map, preserving limit checks, read counters and corruption semantics while
allocating its returned bytes through ExecutionRequestArrayBuilder before copy.
Keep legacy fixture reads for unrelated legacy callers. No production admission,
API, budget or assertion changes. Qualify the three existing resident-history
regressions in one Native packet, review and commit this fixture independently.

Final source scope includes CountedSource plus its HistoryUnavailable and
RetryAfterFailure delegates and a CountedPlacementSource with its actual Drop
owner. The placement provider lives beside the counted archive in region_fixture.rs;
member_custody_tests.rs contains only its two original private-region assertions
and imports the narrow fixture type. All49 dependent fixture cases pass; the two
placement cases rerun after this move and pass. Final independent review2 approves.
