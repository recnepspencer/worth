"""Attribute only observed stack frames; no build and no nearest-symbol guesses."""
import json,sys,subprocess
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';sys.path.insert(0,str(e))
from lifecycle_compiled_frames import Frames
old=json.loads((e/'layout-final-frames-identified.json').read_text())['binaries'];old={b['binary']:b for b in old};binaries={}
for result in sorted((e/'stack-peak-runs').glob('*-path.json')):
 key=result.stem.removesuffix('-path');meta=json.loads((e/'stack-peak-runs'/(key+'.json')).read_text());exe=meta['artifact']['exe'];label=meta['profile']+'-'+('bank' if 'test' in meta['artifact'] else meta['entry']);rows=binaries.setdefault(label,{'binary':label,'exe':exe,'large_functions':[]});known={x['rva']:x for x in rows['large_functions']};pe=Frames(exe);ranges={b:u for b,end,u in pe.functions}
 for process in json.loads(result.read_text()):
  for f in process['call_path']:
   if 'function_rva' not in f:continue
   rva=f['function_rva']
   row=known.setdefault(rva,{'rva':rva,'frame_bytes':pe.unwind(ranges[rva]),'names':[]})
   if f.get('symbol_exact') and f.get('symbol_path') not in row['names']:row['names'].append(f['symbol_path'])
 if meta['profile']=='dev':
  src=old[label.removeprefix('dev-')];src={x['rva']:x for x in src['large_functions']}
  for rva,row in known.items():
   if rva in src and not row['names']:row.update(src[rva])
 rows['large_functions']=list(known.values())
 del pe
inp=e/'stack-peak-frames.json';inp.write_text(json.dumps({'binaries':list(binaries.values())},indent=2))
subprocess.check_call([sys.executable,str(e/'pdb_public_frames.py'),str(inp),str(e/'stack-peak-frames-publics.json')])
