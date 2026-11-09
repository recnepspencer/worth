"""Diagnostic only: first actual WorkCeiling branch, existing dev binary."""
import ctypes as c, ctypes.wintypes as w, json,struct,sys,subprocess
from pathlib import Path
k=c.WinDLL('kernel32',use_last_error=True);d=c.WinDLL('dbghelp',use_last_error=True);P=c.c_void_p
class SI(c.Structure):
 _fields_=[('cb',w.DWORD),('reserved',w.LPWSTR),('desktop',w.LPWSTR),('title',w.LPWSTR),('x',w.DWORD),('y',w.DWORD),('xs',w.DWORD),('ys',w.DWORD),('xc',w.DWORD),('yc',w.DWORD),('fill',w.DWORD),('flags',w.DWORD),('show',w.WORD),('reserved2',w.WORD),('r2',P),('stdin',w.HANDLE),('stdout',w.HANDLE),('stderr',w.HANDLE)]
class PI(c.Structure):_fields_=[('process',w.HANDLE),('thread',w.HANDLE),('pid',w.DWORD),('tid',w.DWORD)]
class Event(c.Structure):_fields_=[('code',w.DWORD),('pid',w.DWORD),('tid',w.DWORD),('padding',w.DWORD),('data',c.c_byte*160)]
class Line(c.Structure):_fields_=[('size',w.DWORD),('key',P),('number',w.DWORD),('file',c.c_char_p),('address',c.c_ulonglong)]
k.CreateProcessW.argtypes=[w.LPCWSTR,w.LPWSTR,P,P,w.BOOL,w.DWORD,P,w.LPCWSTR,c.POINTER(SI),c.POINTER(PI)]
k.OpenThread.argtypes=[w.DWORD,w.BOOL,w.DWORD];k.OpenThread.restype=w.HANDLE
for name in ['GetThreadContext','SetThreadContext']:getattr(k,name).argtypes=[w.HANDLE,P]
k.WaitForDebugEvent.argtypes=[c.POINTER(Event),w.DWORD];k.ContinueDebugEvent.argtypes=[w.DWORD,w.DWORD,w.DWORD]
k.TerminateProcess.argtypes=[w.HANDLE,w.UINT];k.GetStdHandle.argtypes=[w.DWORD];k.GetStdHandle.restype=w.HANDLE;k.CloseHandle.argtypes=[w.HANDLE]
for name in ['ReadProcessMemory','WriteProcessMemory']:getattr(k,name).argtypes=[w.HANDLE,P,P,c.c_size_t,c.POINTER(c.c_size_t)]
k.FlushInstructionCache.argtypes=[w.HANDLE,P,c.c_size_t]
d.SymInitializeW.argtypes=[w.HANDLE,w.LPCWSTR,w.BOOL];d.SymSetOptions.argtypes=[w.DWORD]
d.SymGetLineFromName64.argtypes=[w.HANDLE,c.c_char_p,c.c_char_p,w.DWORD,c.POINTER(w.LONG),c.POINTER(Line)]
d.SymGetLineFromAddr64.argtypes=[w.HANDLE,c.c_ulonglong,c.POINTER(w.DWORD),c.POINTER(Line)]
d.SymFromAddr.argtypes=[w.HANDLE,c.c_ulonglong,c.POINTER(c.c_ulonglong),P]
d.StackWalk64.argtypes=[w.DWORD,w.HANDLE,w.HANDLE,P,P,P,P,P,P]
meta=json.loads(Path(sys.argv[1]).read_text());binary,test=sys.argv[2:]
exe=next(v['binary-path'] for v in meta['rust-binaries'].values() if v['binary-name']==binary)
command=subprocess.list2cmdline([exe,'--exact',test,'--nocapture']);print('DEBUG_COMMAND',command,flush=True)
si=SI();si.cb=c.sizeof(si);si.flags=0x100;si.stdin=k.GetStdHandle(-10&0xffffffff);si.stdout=k.GetStdHandle(-11&0xffffffff);si.stderr=k.GetStdHandle(-12&0xffffffff)
pi=PI();args=c.create_unicode_buffer(command);assert k.CreateProcessW(None,args,None,None,True,2,None,None,c.byref(si),c.byref(pi)),c.get_last_error()

import collections,bisect,time
from lifecycle_compiled_frames import Frames
pe=Frames(exe);symbols=json.loads(Path('target/lane-evidence/principal-measure-symbols.json').read_text());byname={x['name']:x for x in symbols}
required=[x for x in symbols if x['name'].startswith('worth_relational::storage::residency::required_symbols::')]
load_rva=byname['worth_relational::storage::residency::region_load::load_full']['rva']
checkpoint_rva=byname['worth_execution::backend::meter::KernelContext::checkpoint']['rva']
work_rva=byname['worth_relational::storage::residency::region_load::materialization_funding::work']['rva']
collect_rva=byname['worth_relational::storage::residency::required_symbols::collect_for_request']['rva']
insert_rva=next(x['rva'] for x in symbols if 'BTreeSet' in x['name'] and 'Symbol' in x['name'] and 'insert' in x['name'])
failure_rva=284821286
original={};points={};stepping={};image_base=None;active=False;captured=False;measure_tid=None;current=None;groups=[]
checks=collections.Counter();work=collections.Counter();insertions=collections.Counter();full_count=0;exit_code=125;before=time.monotonic()
def read(address,size):
 buf=c.create_string_buffer(size);n=c.c_size_t();assert k.ReadProcessMemory(pi.process,P(address),buf,size,c.byref(n)),c.get_last_error();return buf.raw
def word(address):return struct.unpack('<Q',read(address,8))[0]
def write(address,byte):
 buf=c.create_string_buffer(byte);n=c.c_size_t();assert k.WriteProcessMemory(pi.process,P(address),buf,1,c.byref(n)),c.get_last_error();k.FlushInstructionCache(pi.process,P(address),1)
def arm(rva,kind):
 address=image_base+rva
 if address not in original:original[address]=read(address,1)
 points[address]=kind;write(address,b'\xcc')
def restore():
 for address,byte in original.items():write(address,byte)
def caller_pair(sp):
 first=word(sp);size=pe.frame(first-image_base-1) or 0
 return (first-image_base,word(sp+8+size)-image_base)
def context_for(tid):
 th=k.OpenThread(0x1fffff,False,tid);ctx=c.create_string_buffer(1232);struct.pack_into('<I',ctx,48,0x10000b);assert k.GetThreadContext(th,ctx);return th,ctx
try:
 while True:
  ev=Event();assert k.WaitForDebugEvent(c.byref(ev),60000),c.get_last_error();status=0x10002
  if ev.code==3:
   image_base=struct.unpack_from('<Q',bytes(ev.data),24)[0]
  if ev.code==1:
   raw=bytes(ev.data);code=struct.unpack_from('<I',raw)[0];address=struct.unpack_from('<Q',raw,16)[0]
   if code==0x80000003 and not points and not captured:
    d.SymSetOptions(0x10|0x200|0x4|0x2);assert d.SymInitializeW(pi.process,str(Path(exe).parent),True)
    arm(load_rva,'full')
   elif code==0x80000003 and address in points:
    th,ctx=context_for(ev.tid);sp=struct.unpack_from('<Q',ctx,152)[0];kind=points[address]
    if kind=='full':
     if not active:
      active=True;measure_tid=ev.tid
      for rva,tag in [(checkpoint_rva,'check'),(work_rva,'work'),(collect_rva,'collect'),(insert_rva,'insert'),(failure_rva,'failure')]:arm(rva,tag)
     full_count+=1
    if active and ev.tid==measure_tid:
     if kind=='check':
      units=struct.unpack_from('<Q',ctx,136)[0];assert units<10**12,units
      checks['calls']+=1;checks['units']+=units;checks['last_units']=units
      if checks['calls']%20000==0:print('MEASURE_PROGRESS',dict(checks),flush=True)
     elif kind=='collect':
      current={'full':full_count,'partition_address':struct.unpack_from('<Q',ctx,136)[0],'calls':0,'units':0,'symbols':set(),'inserts':0};groups.append(current)
     elif kind=='work':
      units=struct.unpack_from('<Q',ctx,184)[0];assert units<10**12,units
      pair=caller_pair(sp);work[(pair,units)]+=1
      if current is not None and any(x['rva']<=pair[0]-1<x['rva']+x['size'] and 'collect_for_request::closure' in x['name'] for x in required):current['calls']+=1;current['units']+=units
     elif kind=='insert':
      parent=word(sp)-image_base
      if current is not None and any(x['rva']<=parent-1<x['rva']+x['size'] for x in required):
       symbol=struct.unpack_from('<Q',ctx,136)[0]&0xffffffff;current['symbols'].add(symbol);current['inserts']+=1;insertions[parent]+=1
     elif kind=='failure':
      captured=True;active=False;restore();points.clear();struct.pack_into('<Q',ctx,248,address);struct.pack_into('<I',ctx,68,struct.unpack_from('<I',ctx,68)[0]&~0x100);assert k.SetThreadContext(th,ctx);k.CloseHandle(th)
      print('EXHAUSTION_CAPTURED',dict(checks),'full',full_count,flush=True)
      k.ContinueDebugEvent(ev.pid,ev.tid,status);continue
    # Restore one original instruction, single-step, then re-arm without altering work.
    write(address,original[address]);struct.pack_into('<Q',ctx,248,address);struct.pack_into('<I',ctx,68,struct.unpack_from('<I',ctx,68)[0]|0x100);assert k.SetThreadContext(th,ctx);k.CloseHandle(th);stepping[ev.tid]=address
   elif code==0x80000004 and ev.tid in stepping:
    at=stepping.pop(ev.tid)
    if at in points:write(at,b'\xcc')
    th,ctx=context_for(ev.tid);struct.pack_into('<I',ctx,68,struct.unpack_from('<I',ctx,68)[0]&~0x100);assert k.SetThreadContext(th,ctx);k.CloseHandle(th)
   elif code!=0x80000003:status=0x80010001
  if ev.code==5:exit_code=struct.unpack_from('<I',bytes(ev.data))[0];k.ContinueDebugEvent(ev.pid,ev.tid,status);break
  k.ContinueDebugEvent(ev.pid,ev.tid,status)
except BaseException:
 k.TerminateProcess(pi.process,125);raise
# Resolve each distinct caller once, after all live execution has completed.
def location(rva):
 pc=image_base+rva-1;line=Line();line.size=c.sizeof(line);delta=w.DWORD();file=None;number=None
 if d.SymGetLineFromAddr64(pi.process,pc,c.byref(delta),c.byref(line)):file=line.file.decode(errors='replace');number=line.number
 sym=c.create_string_buffer(16384);struct.pack_into('<I',sym,0,88);struct.pack_into('<I',sym,80,16000);disp=c.c_ulonglong();name='?'
 if d.SymFromAddr(pi.process,pc,c.byref(disp),sym):name=sym.raw[84:84+struct.unpack_from('<I',sym,76)[0]].decode(errors='replace')
 return {'rva':rva,'file':file,'line':number,'symbol':name}
rows=[]
for (pair,units),count in work.items():rows.append({'caller':location(pair[0]),'parent':location(pair[1]),'units_each':units,'calls':count,'units':units*count})
for group in groups:group['unique_symbols']=len(group.pop('symbols'))
result={'checkpoints':dict(checks),'full_entries':full_count,'collections':groups,'funding_work':rows,'symbol_insertions':[{'caller':location(at),'calls':count} for at,count in insertions.items()],'captured_exhaustion':captured,'child_exit':exit_code,'seconds':time.monotonic()-before}
Path('target/lane-evidence/principal-cold-measurement.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
print('MEASUREMENT',json.dumps({k:v for k,v in result.items() if k not in ['funding_work','symbol_insertions']}),flush=True)
raise SystemExit(exit_code if captured else 125)
