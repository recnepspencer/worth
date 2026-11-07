from pathlib import Path
import subprocess,json,hashlib,re
root=Path('target/lane-evidence')
paths=set(subprocess.check_output(['git','diff','--name-only','--','crates','workspaces'],text=True,stderr=subprocess.DEVNULL).splitlines())
paths.update(subprocess.check_output(['git','ls-files','--others','--exclude-standard','--','crates','workspaces'],text=True).splitlines())
assert len(paths)==5,sorted(paths)
for path in sorted(paths):
 assert subprocess.run(['git','diff','--quiet','7d0e5189bc','codex/m3-p3-lifecycle','--',path]).returncode==0,path
 print('Conditional outside-area grant eligible:',path)
for pattern in ['crates/worth-relational/src/branch/root_*','crates/worth-relational/src/branch/root_*/**','crates/worth-relational/src/durability/authority/checkpoint_complete/original_image.rs','crates/worth-relational/src/durability/authority/checkpoint_complete/branches.rs','tools/boundary-check/config/src-court-baseline.json']:
 assert subprocess.run(['git','diff','--quiet','b8d9b28608','--',pattern]).returncode==0,pattern
print('Forbidden root owners, transferred files and baseline unchanged.')
old='crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs'
new='crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/cold_envelope_custody.rs'
name='recovered_nonhead_envelope_custody_ends_with_the_last_inflight_read'
before=subprocess.check_output(['git','show','HEAD:'+old],text=True)
assert len(re.findall(r'fn '+name+r'\(',before))==1
assert len(re.findall(r'fn '+name+r'\(',Path(old).read_text()+Path(new).read_text()))==1
names=lambda s:re.findall(r'#\[test\]\s*fn\s+(\w+)',s)
from collections import Counter
assert Counter(names(before))==Counter(names(Path(old).read_text())+names(Path(new).read_text()))
print('All five original lifetime test names preserved exactly once across old/new files.')
for path in paths:
 data=Path(path).read_text()
 assert len(data.splitlines())<=400,path
 if subprocess.run(['git','ls-files','--error-unmatch',path],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode==0:
  changes=subprocess.check_output(['git','diff','--unified=0','--',path],text=True,stderr=subprocess.DEVNULL)
  added='\n'.join(l for l in changes.splitlines() if l.startswith('+') and not l.startswith('+++'))
 else:added=data
 assert '#[allow' not in added and '#[ignore' not in added,path
print('No added allow/ignore attribute; no source file exceeds400 lines.')
rows=json.loads((root/'queue4-clippy-diagnostics.json').read_text())
assert len(rows)==10
for row in rows:
 assert row['master_diff_exit']==1 and row['lifecycle_diff_exit']==0,row
 assert subprocess.run(['git','diff','--quiet','HEAD','--',row['path']]).returncode==0,row
print('All10 Clippy sites remain untouched master-changed rule3 skips.')
(root/'queue4-source-state.json').write_text(json.dumps({'review_base_head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'files':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in sorted(paths)}},indent=2)+'\n',encoding='utf-8')
print('Frozen5 source hashes for independent partial-migration review.')
