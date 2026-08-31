import fcntl
import os
import pty
import select
import struct
import termios
import time
import shutil
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(ROOT, "dist", "index.js")
OUT = os.path.join(ROOT, "screenshots")
os.makedirs(OUT, exist_ok=True)


def size(fd, cols, rows):
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))


def capture(cols, rows, label, keys=b"", arguments=None, settle=2.2, wait_for=None):
    pid, fd = pty.fork()
    if pid == 0:
        env = os.environ.copy()
        env.pop("NO_COLOR", None)
        env.update({"TERM": "xterm-256color", "COLORTERM": "truecolor", "FORCE_COLOR": "3", "UTHARNESS_COLOR": "truecolor", "XDG_STATE_HOME": tempfile.mkdtemp(prefix="utharness-capture-")})
        node = shutil.which("node") or "/usr/bin/env"
        extra = arguments or []
        argv = (["node", BIN] if node != "/usr/bin/env" else ["env", "node", BIN]) + extra
        os.execve(node, argv, env)
    size(fd, cols, rows)
    data = bytearray()

    def drain(seconds):
        end = time.time() + seconds
        while time.time() < end:
            ready, _, _ = select.select([fd], [], [], 0.05)
            if ready:
                try:
                    data.extend(os.read(fd, 65536))
                except OSError:
                    return

    # Keep draining while the asynchronous runtime and git snapshot settle. This
    # avoids capturing a transient cleared frame on slower CI and Termux hosts.
    if wait_for:
        deadline = time.time() + settle
        while wait_for not in data and time.time() < deadline:
            drain(0.25)
    else:
        drain(settle)
    if keys:
        sequence = keys if isinstance(keys, list) else [keys]
        for chunk in sequence:
            os.write(fd, chunk)
            drain(0.35)
    else:
        drain(0.3)
    # Terminate out-of-band: Ctrl+C is an application shortcut and may cancel a
    # running action instead of exiting, which would make visual tests hang.
    try:
        os.kill(pid, 15)
    except ProcessLookupError:
        pass
    deadline = time.time() + 0.6
    while time.time() < deadline:
        finished, _ = os.waitpid(pid, os.WNOHANG)
        if finished == pid:
            break
        time.sleep(0.05)
    else:
        try:
            os.kill(pid, 9)
        except ProcessLookupError:
            pass
        try:
            os.waitpid(pid, 0)
        except ChildProcessError:
            pass
    with open(os.path.join(OUT, f"{label}-{cols}x{rows}.ansi"), "wb") as handle:
        handle.write(data)


mode = os.environ.get("UTHARNESS_CAPTURE", "all")
if mode in ("all", "main"):
    for cols, rows in [(20, 12), (30, 15), (40, 18), (60, 24), (80, 28), (100, 32), (120, 40), (160, 50), (200, 55)]:
        capture(cols, rows, "focus")
    capture(120, 40, "palette", b"\x0b")
    capture(160, 50, "workspace", b"\x02")
if mode in ("all", "setup"):
    ready = b"environment scan complete"
    capture(100, 30, "setup-menu", arguments=["--setup"], settle=20.0, wait_for=ready)
    capture(100, 30, "setup-provider", [b"\r"], ["--setup"], settle=20.0, wait_for=ready)
    capture(100, 30, "setup-auth", [b"\r", b"\r"], ["--setup"], settle=20.0, wait_for=ready)
    capture(100, 30, "setup-secret", [b"\r", b"\r", b"\r", b"nvapi-example-secret"], ["--setup"], settle=20.0, wait_for=ready)
    capture(100, 30, "setup-tools", [b"\x1b[B", b"\x1b[B", b"\r", b"\r", b"\x1b[B", b"\x1b[B", b"\x1b[B", b"\r"], ["--setup"], settle=20.0, wait_for=ready)
    capture(100, 30, "setup-review", [b"\x1b[B", b"\x1b[B", b"\x1b[B", b"\x1b[B", b"\x1b[B", b"\r"], ["--setup"], settle=20.0, wait_for=ready)
print("captured")
