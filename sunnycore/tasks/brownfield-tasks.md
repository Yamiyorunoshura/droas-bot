<input>
  <context>
    1. Implementation plan document: {root}/docs/implementation-plan/{task_id}-plan.md
    2. Review results document: {root}/docs/review-results/{task_id}-review.md
  </context>
  <templates>
    1. Development notes template: {root}/sunnycore/templates/dev-notes-tmpl.yaml
  </templates>
</input>

<output>
  1. Code fixes implemented based on review findings
  2. Comprehensive development notes: {root}/docs/dev-notes/{task_id}-notes.md
</output>

<constraints, importance="Critical">
  - Must validate all code changes through testing before proceeding
  - Must maintain backward compatibility for brownfield systems
  - Must document every fix with clear rationale and testing evidence
  - Must follow established coding standards and patterns
  - Must ensure integration tests pass before finalizing
</constraints>

<workflow, importance="Critical">
  <stage id="0: plan-creation" level_of_think="think">
    ## Step 0: Plan Creation & Context Analysis
    
    - Read all input documents thoroughly
    - Use sequential-thinking tool to deeply analyze the implementation plan structure
    - Use plan tool to create a systematic approach for brownfield development
    - Validate understanding of existing system architecture and constraints
    
    <questions>
      - What are the key architectural constraints in this brownfield system?
      - Which review findings have the highest priority for implementation?
      - How should we sequence the fixes to minimize disruption?
    </questions>
    
    <checks>
      - [ ] All input documents have been read and understood
      - [ ] Implementation plan aligns with brownfield development best practices
      - [ ] Risk assessment for each proposed change is documented
    </checks>
  </stage>

  <stage id="1: problem-analysis" level_of_think="think hard">
    ## Step 1: Comprehensive Problem Analysis
    
    - Conduct thorough review of all identified issues from review results
    - Use sequential-thinking tool to deeply analyze root causes of existing problems
    - Use sequential-thinking tool to develop systematic problem-solving strategies
    - Categorize issues by impact, complexity, and dependencies
    
    <questions>
      - What is the root cause behind each identified issue?
      - Which issues are symptoms vs. underlying architectural problems?
      - How do these issues relate to brownfield development challenges?
      - What are the potential ripple effects of each fix?
    </questions>
    
    <checks>
      - [ ] All review findings have been analyzed for root causes
      - [ ] Issues are properly categorized and prioritized
      - [ ] Dependencies between fixes have been mapped
      - [ ] Risk assessment for each fix is complete
    </checks>
  </stage>

  <stage id="2: systematic-fix-implementation" level_of_think="think harder">
    ## Step 2: Systematic Fix Implementation
    
    - Begin with highest priority issue based on impact and dependencies
    - Use sequential-thinking tool to develop step-by-step fix implementation
    - Test code after each incremental change to ensure desired behavior
    - Maintain comprehensive documentation of changes and rationale
    - Iterate through fixes systematically until all issues are resolved
    
    <questions>
      - Does this fix address the root cause or just the symptom?
      - How does this change affect system stability and performance?
      - Are there any breaking changes that need special handling?
      - What tests should be run to validate this fix?
    </questions>
    
    <checks>
      - [ ] Each fix has been tested individually
      - [ ] No regressions have been introduced
      - [ ] Fix documentation includes rationale and testing evidence
      - [ ] Code follows established patterns and standards
      - [ ] All issues have been systematically addressed
    </checks>
  </stage>

  <stage id="3: development-documentation" level_of_think="ultra think">
    ## Step 3: Comprehensive Development Documentation
    
    - Execute full integration testing suite to ensure system integrity
    - If integration tests fail, use sequential-thinking tool to diagnose and resolve issues
    - Reference dev-notes template structure for consistent documentation
    - Document each fix with detailed rationale, implementation approach, and testing results
    - Use sequential-thinking tool to transform technical insights into clear markdown format
    - Write comprehensive development notes to {root}/docs/dev-notes/{task_id}-notes.md
    
    <questions>
      - Are all integration tests passing successfully?
      - Is the documentation clear enough for future maintenance?
      - Have we captured lessons learned for future brownfield projects?
      - Does the documentation include before/after comparisons?
      - Are edge cases and potential future issues documented?
    </questions>
    
    <checks>
      - [ ] All integration tests are passing
      - [ ] Development notes follow template structure
      - [ ] Each fix is thoroughly documented with rationale
      - [ ] Testing evidence is included for all changes
      - [ ] Future maintenance considerations are noted
      - [ ] Documentation is in proper markdown format
    </checks>
  </stage>
</workflow>

<example>
  **Example Fix Documentation Structure:**
  
  ### Issue: Memory Leak in User Service
  **Root Cause:** Improper cleanup of database connections
  **Fix Implementation:** Added connection pooling with proper disposal
  **Testing Results:** Memory usage reduced by 40% under load testing
  **Code Changes:** [Link to specific commit/PR]
  **Lessons Learned:** Importance of resource cleanup in long-running services
</example>
