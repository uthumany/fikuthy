# Security policy

## Supported versions

Security fixes are provided for the latest published release.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private
vulnerability reporting feature under **Security → Report a vulnerability**.
Include affected versions, reproduction steps, impact, and any suggested
mitigation. We will acknowledge a complete report within five business days.

Never include live credentials, personal data, or signing material in a report.

## Security boundaries

UTHARNESS defaults to read-only SAFE mode. Shell execution requires explicit
approval, but approval is not a sandbox. Run untrusted workloads inside an
operating-system sandbox or disposable environment.
