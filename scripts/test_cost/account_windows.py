"""Account a suspended Windows process and its descendants in an unlimited job."""
import ctypes
from ctypes import wintypes
import subprocess


class BasicAccounting(ctypes.Structure):
    _fields_ = [('TotalUserTime', ctypes.c_int64), ('TotalKernelTime', ctypes.c_int64),
                ('ThisPeriodTotalUserTime', ctypes.c_int64), ('ThisPeriodTotalKernelTime', ctypes.c_int64),
                ('TotalPageFaultCount', wintypes.DWORD), ('TotalProcesses', wintypes.DWORD),
                ('ActiveProcesses', wintypes.DWORD), ('TotalTerminatedProcesses', wintypes.DWORD)]


def run(command):
    import _winapi
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
    kernel.CreateJobObjectW.restype = wintypes.HANDLE
    kernel.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
    kernel.AssignProcessToJobObject.restype = wintypes.BOOL
    kernel.QueryInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int,
                                                ctypes.c_void_p, wintypes.DWORD, ctypes.c_void_p]
    kernel.QueryInformationJobObject.restype = wintypes.BOOL
    kernel.ResumeThread.argtypes = [wintypes.HANDLE]
    kernel.ResumeThread.restype = wintypes.DWORD
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    job = kernel.CreateJobObjectW(None, None)
    if not job:
        return subprocess.call(command), None, 'job-unavailable'
    startup = subprocess.STARTUPINFO()
    startup.dwFlags = subprocess.STARTF_USESTDHANDLES
    startup.hStdInput = _winapi.GetStdHandle(_winapi.STD_INPUT_HANDLE)
    startup.hStdOutput = _winapi.GetStdHandle(_winapi.STD_OUTPUT_HANDLE)
    startup.hStdError = _winapi.GetStdHandle(_winapi.STD_ERROR_HANDLE)
    try:
        process, thread, _, _ = _winapi.CreateProcess(
            None, subprocess.list2cmdline(command), None, None, True, 4, None, None, startup)
    except BaseException:
        kernel.CloseHandle(job)
        raise
    # Assign before the first instruction. No CPU, memory or process limits are
    # set: the job exists solely to account descendants without a spawn race.
    assigned = bool(kernel.AssignProcessToJobObject(job, process))
    try:
        if kernel.ResumeThread(thread) == 0xFFFFFFFF:
            _winapi.TerminateProcess(process, 127)
            raise OSError(ctypes.get_last_error(), 'could not resume test process')
        _winapi.WaitForSingleObject(process, _winapi.INFINITE)
        code = _winapi.GetExitCodeProcess(process)
        info = BasicAccounting()
        if assigned and kernel.QueryInformationJobObject(job, 1, ctypes.byref(info), ctypes.sizeof(info), None):
            cpu = (info.TotalUserTime + info.TotalKernelTime) / 10_000_000
            posture = 'complete' if info.ActiveProcesses == 0 else 'descendants-still-active'
            return code, cpu, posture
        return code, None, 'accounting-unavailable'
    finally:
        _winapi.CloseHandle(thread)
        _winapi.CloseHandle(process)
        kernel.CloseHandle(job)
