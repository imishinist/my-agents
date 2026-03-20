# Sandbox Awareness

## Purpose

Detect when the agent is running inside a sandboxed environment (Docker container, macOS seatbelt, systemd-run, etc.) and communicate limitations to the user instead of retrying hopelessly.

## Rules

### Permission Error Signals

Treat the following as potential sandbox indicators:

| Signal | Typical Source |
|---|---|
| `Operation not permitted` | seatbelt, seccomp, read-only filesystem |
| `Permission denied` | filesystem ACL, dropped capabilities |
| `EPERM` / `EACCES` | syscall-level denial |
| `Read-only file system` | container with read-only root |
| `cannot execute binary file` | missing architecture or noexec mount |
| `socket: operation not permitted` | network namespace restriction |
| `ioctl: Operation not permitted` | device access blocked |

A single occurrence may be a normal permission issue. Two or more distinct signals from the list above strongly suggest a sandbox.

### Decision Flow

1. **Observe** — A command or file operation fails with one of the signals above.
2. **Correlate** — Check whether other operations are also restricted (e.g., network access, writing to `/tmp`, running `sudo`). Multiple restrictions confirm a sandbox.
3. **Assess** — Determine if a workaround exists within the allowed scope (e.g., write to a different path, use a different command).
4. **Act or Escalate**
   - If a workaround exists, apply it and note the constraint.
   - If no workaround exists, stop and inform the user.

### Communicating to the User

When the agent determines that a sandbox is blocking progress:

- State clearly that the environment appears to be sandboxed.
- Name the specific operation that failed and the error received.
- Explain that this restriction cannot be bypassed from within the agent.
- Ask the user to either adjust the sandbox policy or perform the blocked operation manually.

Example:

> This environment appears to be running inside a sandbox (received "Operation not permitted" when attempting to write to /etc/hosts). This restriction cannot be bypassed from within the agent. Could you either adjust the sandbox policy or perform this step manually?

### What NOT to Do

- Do not retry the same operation repeatedly after a sandbox-related error.
- Do not attempt privilege escalation (`sudo`, `su`, capability changes).
- Do not silently skip the blocked operation and continue as if it succeeded.
