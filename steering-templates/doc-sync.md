# Document Sync Rule

## Purpose

Ensure related documentation is updated whenever code or design changes are made.

## Rules

### When Documentation Updates Are Required

- **API changes**: Update API docs when endpoints are added, modified, or removed
- **Config changes**: Update README or setup guides when env vars or config items change
- **Dependency changes**: Review and update setup instructions when packages are added or removed
- **Directory structure changes**: Update path references in docs when files are moved or renamed
- **Feature changes**: Update usage docs when user-facing features change
- **Infrastructure changes**: Update deploy procedures and architecture diagrams

### Target Documents

Check and update these files if they exist:

- `README.md`
- Files under `docs/`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- API documentation (OpenAPI, GraphQL schema, etc.)
- Architecture Decision Records (ADR)

### Completion Checklist

Before finishing work, verify:

1. Are there documents related to this change?
2. If so, do they reflect the current state?
3. If new features or concepts were added, should new documentation be created?
