Decompose the following epic into implementable features.

## Epic
**Title:** {{epic_title}}
**Description:** {{epic_description}}

## Requirements

Break this epic into features that:
- Are independently implementable by a single developer agent
- Have clear boundaries (specific files/modules to create or modify)
- Include explicit dependency relationships
- Can be parallelized where possible (minimize sequential dependencies)

## Output Format

Respond with ONLY a JSON object in this exact format:

```json
{
  "features": [
    {
      "title": "Short feature title",
      "specification": "Detailed specification in markdown. Include:\n- What to implement\n- Expected behavior\n- API contracts / interfaces if applicable\n- Test requirements",
      "priority": "high|medium|low",
      "depends_on_titles": ["Title of dependency feature"],
      "allowed_paths": ["src/module/**", "tests/module/**"]
    }
  ]
}
```

## Guidelines

- Each feature should be completable in one focused session
- Features with no dependencies should be marked as high priority
- Keep the dependency graph as shallow as possible (prefer wide over deep)
- Specify `allowed_paths` as glob patterns indicating which files the worker may modify
- `depends_on_titles` references other feature titles in this same list
- Order features logically: foundational first, dependent later
