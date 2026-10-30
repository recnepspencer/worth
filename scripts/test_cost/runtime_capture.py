"""Persist prebuilt nextest result events before its process exits."""
import subprocess


def execute(command, cwd, environment, log):
    output = []
    with subprocess.Popen(command, cwd=cwd, env=environment, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, text=True, encoding='utf-8',
                          errors='replace') as child:
        for line in child.stdout:
            log.write(line)
            log.flush()
            output.append(line)
        child.stdout.close()
        code = child.wait()
    return subprocess.CompletedProcess(command, code, ''.join(output), '')
