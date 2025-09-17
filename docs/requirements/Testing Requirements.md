## Testing Requirements

```yaml
testing-requirements:
  unit-testing:
    coverage-target: "80%"
    framework: "Rust test + image pipeline helpers"
  integration-testing:
    approach: "Mock Discord interactions; sandbox guild for end-to-end flows"
    test-environments: ["local", "CI"]
  acceptance-testing:
    criteria: "All FR acceptance criteria met in a sandbox guild"
    responsible-party: "Product Owner + Developer"
  performance-testing:
    load-scenarios:
      - "20 concurrent preview commands for 5 minutes"
    success-criteria: "Meets NFR-P-001 and NFR-P-002 targets"
```

