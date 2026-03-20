# Document Sync Rule

## Purpose

Keep documentation in sync when agent configs or templates change.

## Rules

### When Documentation Updates Are Required

- **Agent added/changed**: Update README.md agent list when files under `agents/` change
- **Steering template added/changed**: Update README.md and create-agent prompt template list when `steering-templates/` changes
- **Makefile changed**: Update README.md setup instructions if install/uninstall procedures change
- **Directory structure changed**: Update the structure section in README.md

### Target Documents

- `README.md`
- `agents/prompts/create-agent.md` (template list and flow definitions)
