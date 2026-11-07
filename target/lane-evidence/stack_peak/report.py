"""Reconcile both observed samples and exact-address frame attribution."""
import hashlib,json,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';out=e/'stack-peak-runs';sys.path.insert(0,str(e));from whole_binary_frames import canonical
source=json.loads((e/'stack-peak-frames-identified.json').read_text());frames={b['binary']:{f['rva']:f for f in b['large_functions']} for b in source['binaries']}
rows=[]
for profile in ['dev','release']:
 for name in ['invariant_reads','required_feature','shared_execution','worth-query-consumer-root','assessment','transfer']:
  key=profile+'-'+name;meta=json.loads((out/(key+'.json')).read_text());paint=json.loads((out/(key+'-paint.json')).read_text());paths=json.loads((out/(key+'-path.json')).read_text());label=profile+'-'+('bank' if 'test' in meta['artifact'] else meta['entry'])
  assert len(paint)==len(paths)==(5 if name in ('assessment','transfer') else 1)
  for a,b in zip(paint,paths):
   assert a['index']==b['index'] and b['stable'] and b['watch_hits']==1
   assert a['peak_bytes']-a['limits']['entry_baseline']==b['peak_bytes']-b['limits']['entry_baseline']
   assert a['limits']['reserve_bytes']==b['limits']['reserve_bytes']==64*1024**2
  with open(meta['artifact']['exe'],'rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest()==meta['exe_sha256']
  chosen=max(paths,key=lambda p:p['peak_bytes']-p['limits']['entry_baseline']);enriched=[]
  for f in chosen['call_path']:
   row=f.copy()
   if 'function_rva' in f:
    matched=frames[label][f['function_rva']];row['exact_names']=sorted({canonical(n)[0] or n for n in matched['names']})
    if matched.get('coff_matches'):row['attribution']='COFF CRC,size,byte-match outside relocations'
    elif row['exact_names']:row['attribution']='exact-address PDB'
   enriched.append(row)
  peak=max(p['peak_bytes'] for p in paint+paths)
  rows.append({'profile':profile,'name':name,'peak_bytes':peak,'fits_1MiB':peak<=1048576,'path_sample_peak_bytes':chosen['peak_bytes'],'path_sample_index':chosen['index'],'body_depth':chosen['peak_bytes']-chosen['limits']['entry_baseline'],'sample_peaks':[[p['peak_bytes'] for p in paint],[p['peak_bytes'] for p in paths]],'call_path':enriched,'metadata':meta})
(e/'stack-peak-summary.json').write_text(json.dumps(rows,indent=2))
for row in rows:print(row['profile'],row['name'],row['peak_bytes'],'path',row['path_sample_peak_bytes'],'fits',row['fits_1MiB'])
