from pathlib import Path
import json,subprocess
r=Path.cwd();base=Path('C:/programming/worth worktees/worktrees');peer=base/'m3-p3-lifecycle'
prefix='workspaces/worth-query/crates/worth-query-execution/src/domain_computation/'
files=['primary_graph/application_runtime_core.rs','primary_graph/application_runtime/installation/assembly.rs','execution_runtime/product_world/home_attachment.rs','primary_graph/provider/outstanding_dispatch.rs','primary_graph/provider/committed_dispatch_outbox.rs','primary_graph/provider/committed_dispatch_outbox/restoration.rs','primary_graph/provider/committed_dispatch_outbox/original_row.rs','application_aftermath/external_effect/outbox.rs','application_aftermath/external_effect/identity_derivation.rs','application_aftermath/external_effect/causal_event.rs','application_aftermath/external_effect/dispatch.rs','application_aftermath/external_effect/classification.rs','application_aftermath/external_effect/observation.rs','application_aftermath/external_effect/posture.rs','primary_graph/application_query/observed_source/source_identity/runtime_commitment.rs','primary_graph/application_query/observed_source/source_identity.rs','primary_graph/application_runtime/inbound_occurrence.rs','primary_graph/provider/inbound_terminal_index/rebuild.rs']
paths=[prefix+x for x in files]+['workspaces/worth-query/crates/worth-query-host/src/application_home/support.rs','workspaces/worth-query/crates/worth-query-host/src/application_home/opening/service.rs']
paths += [prefix+'primary_graph/'+x for x in ['application_attempt/idempotency/recorded_intent.rs', 'application_attempt/idempotency/recorded_intent_tests.rs', 'provider.rs', 'provider/installation.rs', 'application_runtime/installation/home_attachment.rs', 'application_runtime/installation/graph_publication.rs', 'provider/committed_dispatch_outbox/record_read.rs', 'provider/committed_dispatch_outbox/restoration_tests.rs', 'provider/idempotency/snapshot_resolution/source_receipt.rs', 'provider/idempotency/snapshot_resolution/original_receipt.rs', 'provider/idempotency/snapshot_resolution/original_receipt/sealing.rs', 'provider/idempotency/snapshot_resolution/original_receipt/outbox.rs']]
paths=sorted(set(paths))
rows=[]
for path in paths:
 assert Path('\\\\?\\'+str((r/path).absolute())).is_file(),path
 diff=['git','-C',str(peer),'diff','--quiet','e787b51b3e','--',path];status=['git','-C',str(peer),'status','--porcelain','--',path]
 d=subprocess.run(diff);s=subprocess.check_output(status,text=True);assert d.returncode in (0,1)
 row={'path':path,'peer':str(peer),'peer_HEAD':subprocess.check_output(['git','-C',str(peer),'rev-parse','HEAD'],text=True).strip(),'diff_command':diff,'diff_exit':d.returncode,'status_command':status,'status':s,'clear_of_PC':d.returncode==0 and not s};rows.append(row);print(json.dumps(row),flush=True)
(r/'target/lane-evidence/h8-query-grants.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
