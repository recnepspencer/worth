from pathlib import Path
import subprocess,json,hashlib
root=Path('target/lane-evidence')
head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
p='workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_installation/construction.rs'
code=subprocess.run(['git','diff','--quiet','7d0e5189bc','codex/m3-p3-lifecycle','--',p]).returncode
assert code==1
print('D13 shared checkpoint construction owner differs on Lifecycle: exit1',p)
print(subprocess.check_output(['git','diff','--numstat','7d0e5189bc','codex/m3-p3-lifecycle','--',p],text=True).strip())
p='crates/worth-foundational/src/identities/category_ids.rs'
source=Path(p).read_text()
assert 'mod canonical_digest_wire;' not in source
matches=subprocess.check_output(['rg','-n','CanonicalDigestId','crates/worth-foundational/src','--glob','*.rs'],text=True)
(root/'queue3-named-digest-search.txt').write_text(matches,encoding='utf-8')
wire='crates/worth-foundational/src/identities/category_ids/canonical_digest_wire.rs'
assert not Path(wire).exists()
integration=subprocess.check_output(['git','show','codex/m3-host-home:'+wire],text=True)
assert 'impl Serialize for CanonicalDigestId' in integration and "Deserialize<'de> for CanonicalDigestId" in integration
print('Current CanonicalDigestId encoding leaf absent; integration leaf supplies named Serialize/Deserialize:',wire)
print('Full exact-name search retained in queue3-named-digest-search.txt')
reader=Path('crates/worth-relational/src/history/access/envelope_reads.rs').read_text()
assert 'pub(crate) fn demand_commit_envelope' in reader
shared=Path('crates/worth-relational/src/history/data/canonical_commit_envelope/shared.rs').read_text()
assert 'cannot_retain_a_weak_allocation' in shared and 'cannot_extract_the_owner' in shared
print('Actual envelope-demand visibility is pub(crate); public shared envelope explicitly compile-fences Weak and Arc extraction.')
for p in ['tools/boundary-check/config/src-court-baseline.json','crates/worth-relational/src/history/access/envelope_reads.rs']:
 assert subprocess.run(['git','diff','--quiet','HEAD','--',p]).returncode==0
print('No baseline, product source or observer door modified.')
record={'head':head,'lifecycle':subprocess.check_output(['git','rev-parse','codex/m3-p3-lifecycle'],text=True).strip(),'integration_read_only_ref':subprocess.check_output(['git','rev-parse','codex/m3-host-home'],text=True).strip(),'scope':'read-only prerequisite probes, not runtime qualification'}
(root/'queue-owner-preflight-identity.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
