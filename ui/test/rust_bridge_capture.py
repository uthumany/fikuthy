import fcntl
import os
import pty
import select
import struct
import termios
import time
import shutil
import re

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BIN = os.path.join(ROOT, "target", "release", "utharness")

def set_size(fd):
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 36, 120, 0, 0))

pid, fd = pty.fork()
if pid == 0:
    env = os.environ.copy()
    env.update({"TERM": "xterm-256color", "COLORTERM": "truecolor", "UTHARNESS_COLOR": "truecolor", "UTHARNESS_UI_ENTRY": os.path.join(ROOT, "ui", "dist", "index.js")})
    os.execve(BIN, [BIN, "tui"], env)
set_size(fd)
time.sleep(1.8)
os.write(fd, b"\x03")
data = bytearray()
end = time.time() + 0.6
while time.time() < end:
    ready, _, _ = select.select([fd], [], [], 0.05)
    if ready:
        try:
            data.extend(os.read(fd, 65536))
        except OSError:
            break
exit_code = None
try:
    _, wait_status = os.waitpid(pid, 0)
    exit_code = os.waitstatus_to_exitcode(wait_status)
except ChildProcessError:
    pass
text = data.decode("utf-8", errors="replace")
plain_ansi = re.sub(r"\x1b\[[0-9;?]*[ -/]*[@-~]", "", text)
print("ink marker:", "UTHARNESS AGENT — focus mode" in plain_ansi)
print("ascii marker:", "UTHARNESS" in plain_ansi)
print("prompt marker:", "Type your message" in plain_ansi or "Type a message" in plain_ansi)
print("exit status:", exit_code)
print("exit marker:", exit_code == 0)
plain = ''.join(char if char.isprintable() or char in '\\n\\r\\t' else ' ' for char in plain_ansi)
print('visible sample:', '\\n'.join(line for line in plain.splitlines() if line.strip())[-18:])
