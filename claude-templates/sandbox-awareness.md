## Sandbox Awareness

### Purpose

Detect when running inside a sandboxed environment and communicate limitations to the user instead of retrying hopelessly.

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

A single occurrence may be a normal permission issue. Two or more distinct signals strongly suggest a sandbox.

### Decision Flow

1. **Observe** — A command or file operation fails with one of the signals above.
2. **Correlate** — Check whether other operations are also restricted. Multiple restrictions confirm a sandbox.
3. **Assess** — Determine if a workaround exists within the allowed scope.
4. **Act or Escalate**
   - If a workaround exists, apply it and note the constraint.
   - If no workaround exists, stop and inform the user.

### What NOT to Do

- Do not retry the same operation repeatedly after a sandbox-related error.
- Do not attempt privilege escalation (`sudo`, `su`, capability changes).
- Do not silently skip the blocked operation and continue as if it succeeded.
