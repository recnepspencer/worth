from pathlib import Path
import subprocess,json,re,datetime
root=Path('target/lane-evidence')
paths=json.loads((root/'funding-final-outside-clippy-files.json').read_text())
rows=[]
for path in paths:
    lifecycle=subprocess.run(['git','diff','--quiet','7d0e5189bc','codex/m3-p3-lifecycle','--',path]).returncode
    master=subprocess.run(['git','diff','--quiet','7d0e5189bc','origin/master','--',path]).returncode
    assert lifecycle in (0,1) and master in (0,1),(path,lifecycle,master)
    rows.append({'path':path,'lifecycle_diff_exit':lifecycle,'master_diff_exit':master,'action':'fix' if lifecycle==master==0 else 'skip'})
(root/'funding-landing-lint-eligibility.json').write_text(json.dumps({'head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'lifecycle':subprocess.check_output(['git','rev-parse','codex/m3-p3-lifecycle'],text=True).strip(),'master':subprocess.check_output(['git','rev-parse','origin/master'],text=True).strip(),'sites':rows},indent=2)+'\n')
for row in rows: print(row['action'],row['lifecycle_diff_exit'],row['master_diff_exit'],row['path'])
patch=(root/'proposed-funding-final-outside-clippy.patch').read_bytes()
chunks=re.split(rb'(?=^--- a/)',patch,flags=re.M)
selected=[]
for chunk in chunks:
    if not chunk: continue
    path=chunk.splitlines()[0].decode()[6:]
    row=next(r for r in rows if r['path']==path)
    if row['action']=='fix': selected.append(chunk)
(root/'funding-landing-eligible-clippy.patch').write_bytes(b''.join(selected))
print('eligible:',len(selected),'skipped:',len(rows)-len(selected))
