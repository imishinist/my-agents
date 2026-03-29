## Architecture Decision Records

### Purpose

Record important design decisions and their rationale for future reference.

### When to Record

- Adopting a new library or framework
- Choosing an architecture pattern (monolith vs microservices, etc.)
- Selecting a database or storage solution
- Deciding on API design approach
- Choosing authentication/authorization method
- Deciding on deployment strategy
- Overriding a previous design decision

### Record Format

Store records in `docs/adr/` using this format:

```
# ADR-{number}: {title}

## Status
Proposed / Accepted / Deprecated / Superseded (by ADR-XXX)

## Context
Background and problem that necessitated this decision

## Decision
What was chosen

## Rationale
Why this choice was made. Alternatives considered and their evaluation

## Consequences
Impact and trade-offs resulting from this decision
```

### Completion Checklist

When making design-related changes:

1. Check if existing ADRs are affected
2. Determine if a new ADR is needed
3. Update the status of existing ADRs if necessary
