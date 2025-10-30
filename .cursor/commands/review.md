# Code Review

Review code changes for architecture quality, modularity, bug potential, and refactoring opportunities. Focus on Tauri + React architecture with Rust backend and TypeScript frontend.

**IMPORTANT: When executing this command, you MUST automatically fix all discovered issues and warnings, not just report them.**

## Review Process

### 1. Analyze Current Changes

**Step 1: Identify Modified Files**

Use these commands to understand what has been modified:

```bash
# For uncommitted changes
git status
git diff --name-only

# For staged changes
git diff --cached --name-only

# Compare with a branch (e.g., main)
git diff --name-only main...HEAD
```

**Step 2: Analyze Changes**

For each modified file:

- Use `read_file` tool to read the file content
- Use `grep` tool to find usages and dependencies
- Use `codebase_search` to understand module relationships
- Identify affected modules and their boundaries
- **IMPORTANT: Fix issues immediately when discovered, do not just record them**

### 2. Architecture Quality Checklist

#### High Cohesion

- **Single Responsibility**: Each module/function should have one clear purpose
- **Related Logic Together**: Functions/data that belong together are in the same module
- **No Mixed Concerns**: Business logic, UI, and data access are separated
- **Clear Module Boundaries**: Each module has well-defined responsibilities

**Check for:**

- Functions that do multiple unrelated things
- Modules that mix concerns (e.g., UI + business logic + data fetching)
- Functions that could be split into smaller, focused functions
- React components that contain business logic (should use hooks instead)
- Rust modules mixing different concerns (e.g., API + storage + business logic)

#### Low Coupling

- **Minimal Dependencies**: Modules depend on abstractions, not concrete implementations
- **Interface-Based Design**: Dependencies are through interfaces/types, not concrete classes
- **No Circular Dependencies**: Module A shouldn't depend on B if B depends on A
- **Loose Inter-Module Coupling**: Changes in one module shouldn't require changes in others

**Check for:**

- Direct imports of implementation details from other modules
- Frontend directly accessing backend internals (should use Tauri commands)
- Shared mutable state between modules
- Hard-coded dependencies between modules
- Tight coupling between React components
- Rust modules directly depending on concrete implementations

#### Modularity

- **Clear Module Boundaries**: Each module has a well-defined public API
- **Encapsulation**: Internal implementation details are hidden
- **Reusability**: Modules can be used independently
- **Testability**: Each module can be tested in isolation

**Check for:**

- Public APIs that expose too much internal detail
- Modules that are hard to test due to tight coupling
- Code duplication that could be extracted into shared modules
- Utility functions that should be in separate modules
- React hooks that could be reused
- Rust functions that could be extracted into separate modules

### 3. Project-Specific Considerations

#### Tauri Architecture

- **Frontend-Backend Separation**: Frontend (React/TypeScript) should not have direct file system access
- **Tauri Commands**: Rust commands should be well-defined with proper error handling
  - Commands should use `#[tauri::command]` macro
  - Return `Result<T, E>` or `anyhow::Result`
  - Handle errors gracefully and return meaningful error messages
- **Event Communication**: Use Tauri events properly, avoid tight coupling
  - Events should be used for one-way communication (backend → frontend)
  - Avoid bidirectional event loops
- **Resource Management**: Properly handle Tauri resource IDs and cleanup
  - Remove event listeners on component unmount
  - Clean up resources in Rust (files, connections, etc.)

**Check for:**

- Frontend trying to access file system directly (should use Tauri commands)
- Missing error handling in Tauri commands
- Event listeners not being cleaned up
- Resource leaks (file handles, network connections)

#### Rust-Specific Patterns

- **Async/Await**: Proper use of Tokio runtime
  - Avoid `block_on` in async contexts (causes panic: "Cannot start a runtime from within a runtime")
  - Use `await` for async operations
  - Use `tauri::async_runtime::spawn` for background tasks
- **Error Handling**: Use `Result<T, E>` or `anyhow::Result`
  - Avoid `unwrap()` in production code (use `?` operator or proper error handling)
  - Propagate errors properly up the call stack
- **Memory Safety**: No unsafe code without justification
  - Proper lifetime management
  - Avoid unnecessary cloning
  - Use `Arc<Mutex<T>>` for shared state across async tasks
- **Lock Ordering**: Avoid deadlocks with multiple mutexes
  - Use `try_lock()` when blocking might cause issues
  - Consistent lock ordering
  - Avoid holding locks across await points

**Check for:**

- `block_on` calls inside async functions
- `unwrap()` calls that could panic
- Missing error handling with `?` operator
- Deadlock potential with multiple mutexes
- Async cancellation not handled properly

#### TypeScript/React-Specific Patterns

- **Type Safety**: No `any` types, proper React hook dependencies
  - Use proper TypeScript types for all props and state
  - Avoid `as any` type assertions
  - Proper generic types for reusable components
- **Component Design**: Props interfaces, proper memoization
  - Define interfaces for component props
  - Use `memo()` for expensive components
  - Avoid unnecessary re-renders
- **State Management**: Avoid prop drilling, use context/hooks appropriately
  - Use React Context for shared state
  - Custom hooks for complex logic
  - Avoid lifting state too high
- **Hook Dependencies**: Proper dependency arrays in useEffect/useMemo/useCallback
  - Follow exhaustive-deps rule
  - Avoid missing dependencies
  - Clean up effects properly

**Check for:**

- `any` types or unsafe type assertions
- Missing dependencies in hook arrays
- Components not cleaning up effects (event listeners, timers)
- Props drilling through multiple levels
- Unnecessary re-renders

### 4. Bug Potential Analysis

#### Common Bug Patterns

- **Null/Undefined Checks**: Proper null/undefined handling
  - Rust: Use `Option<T>`, check with `match` or `if let`
  - TypeScript: Use optional chaining `?.`, nullish coalescing `??`
- **Error Handling**: All error paths are handled appropriately
  - Rust: Use `Result<T, E>` and propagate errors
  - TypeScript: Handle promise rejections, try-catch blocks
- **Edge Cases**: Boundary conditions, empty inputs, overflow cases
  - Empty arrays, null values, zero-length strings
  - Array bounds checking
  - Number overflow/underflow
- **Race Conditions**: Async operations, concurrent access, state updates
  - Multiple async operations modifying same state
  - Lock contention in Rust
  - React state updates in async callbacks
- **Resource Management**: File handles, network connections, memory leaks
  - Unclosed file handles
  - Event listeners not removed
  - Memory leaks from closures keeping references

#### Tauri-Specific Bug Patterns

- **block_on in async context**: Never use `block_on` inside async functions
  - This causes panic: "Cannot start a runtime from within a runtime"
  - Use `await` instead, or `tauri::async_runtime::spawn` for background tasks
- **Tauri command errors**: Proper error propagation from Rust to TypeScript
  - Commands should return `Result<T, E>` or `anyhow::Result`
  - Frontend should handle errors gracefully
- **Event listener cleanup**: Remove event listeners on component unmount
  - Use `return` function in `useEffect` to clean up
  - Remove global event listeners properly
- **Resource ID management**: Properly handle Tauri resource IDs
  - Don't leak resource IDs
  - Proper cleanup of resources

#### Rust Async-Specific Issues

- **Runtime panic**: Avoid creating runtime from within runtime
  - Never use `block_on` in async context
  - Use `tokio::runtime::Handle::current()` if needed
- **Lock ordering**: Avoid deadlocks with multiple mutexes
  - Use consistent lock ordering
  - Consider using `try_lock()` when blocking might cause issues
- **Async cancellation**: Handle cancellation properly in async tasks
  - Use `tokio::select!` for cancellation
  - Clean up resources on cancellation

**Check for:**

- Missing error handling in async functions
- Unhandled edge cases (empty arrays, null values, etc.)
- Potential race conditions in async code
- Resource leaks (unclosed files, event listeners, etc.)
- TypeScript `any` types or unsafe type assertions
- Rust `unwrap()` calls that could panic
- `block_on` in async contexts
- Missing cleanup in React effects

### 5. Refactoring Opportunities

#### Code Smells to Identify

- **Long Functions**:
  - TypeScript: > 50 lines should be considered for splitting
  - Rust: > 80 lines (Rust functions can be longer due to pattern matching)
- **Deep Nesting**: More than 3-4 levels of nesting suggests refactoring needed
  - Use early returns
  - Extract functions
  - Use guard clauses
- **Magic Numbers**: Hard-coded values should be constants
  - Use named constants
  - Extract to configuration
- **Code Duplication**: Repeated patterns should be extracted
  - Extract common logic into functions
  - Use higher-order functions
  - Create reusable components/hooks
- **Large Modules**:
  - TypeScript: > 300 lines may need splitting
  - Rust: > 500 lines (acceptable for Rust modules, but consider splitting)
- **Complex Conditionals**: Long if/else chains or switch statements
  - Use strategy pattern
  - Extract to separate functions
  - Use pattern matching (Rust) or lookup tables
- **Comments Explaining Code**: If code needs comments to explain what it does, refactor for clarity
  - Better naming
  - Extract functions
  - Simplify logic
- **Over-Engineering**: Unnecessary complexity or abstractions
  - Complex patterns where simple solutions would work
  - Abstractions that don't add value
  - "Clever" code that's hard to understand
  - Preemptively solving problems that don't exist yet
  - Over-abstracting for hypothetical future needs
  - Unnecessary design patterns or architectural layers
  - Favor simple, direct solutions over clever ones

#### Optimization Opportunities

- **Performance**: Unnecessary computations, inefficient algorithms
  - Unnecessary re-renders in React
  - Inefficient loops
  - Missing memoization
- **Memory**: Unnecessary allocations, large objects kept in memory
  - Unnecessary cloning in Rust
  - Large objects in React state
  - Memory leaks from closures
- **Network**: Redundant API calls, missing caching
  - Duplicate API requests
  - Missing request deduplication
  - No caching strategy
- **Bundling**: Unused imports, unnecessary dependencies
  - Unused imports in TypeScript/Rust
  - Unnecessary dependencies in package.json/Cargo.toml
- **Simplicity Over Optimization**: Avoid premature optimization
  - Don't optimize code that doesn't need optimization
  - Measure before optimizing - profile first
  - Simple code is often fast enough
  - Complexity should be justified by actual performance needs

### 6. Review Execution

**Step 1: Get Changed Files**

```bash
# Option 1: Git diff (for uncommitted changes)
git diff --name-only

# Option 2: Git diff --cached (for staged changes)
git diff --cached --name-only

# Option 3: Compare with branch
git diff --name-only main...HEAD
```

**Step 2: For each modified file:**

- Use `read_file` tool to read the file and understand its purpose
- Use `grep` tool to find dependencies and usages
- Use `codebase_search` to understand module relationships
- Analyze functions for cohesion and complexity
  - Count lines per function
  - Check nesting depth
  - Identify responsibilities
- Look for potential bugs
  - Check error handling
  - Verify edge cases
  - Look for race conditions
- Identify refactoring opportunities
  - Code duplication
  - Long functions
  - Complex logic

**Step 3: Cross-module Analysis**

- Use `codebase_search` to map dependencies between modules
- Check for circular dependencies
- Verify module boundaries are respected
  - Frontend vs backend separation
  - Component vs hook separation
  - Business logic vs UI separation
- Ensure interfaces are well-defined
  - Tauri command signatures
  - React component props
  - Rust module public APIs

**Step 4: Fix All Issues Automatically**

**CRITICAL: You MUST automatically fix all discovered issues, not just report them!**

For each discovered issue:

1. **Critical Priority Issues** - **MUST fix immediately**
   - Panic risks (unwraps, indexing without bounds check, `block_on` in async)
   - Data loss potential
   - Security vulnerabilities
   - Memory leaks or resource leaks
   - Use `search_replace` or `write` tools to fix code directly

2. **High Priority Issues** - **MUST fix immediately**
   - Bug potential (race conditions, null handling, missing error handling)
   - Performance issues (blocking operations, unnecessary re-renders)
   - Major architectural issues (tight coupling, mixed concerns)
   - Type safety issues (TypeScript `any` types)
   - Use `search_replace` or `write` tools to fix code directly

3. **Medium Priority Issues** - **Fix with priority**
   - Code smells (long functions, duplication, magic numbers)
   - Testability concerns
   - Architectural improvements
   - Code style inconsistencies

4. **Low Priority Issues** - **Fix if possible**
   - Code style improvements
   - Minor optimizations
   - Documentation improvements

**Fix Workflow:**

1. After identifying an issue, immediately fix it using appropriate tools:
   - Use `search_replace` to modify existing code
   - Use `read_file` to read file content
   - Use `write` to create new files or rewrite files
   - Use `grep` to find related code

2. After fixing, verify:
   - Run `pnpm run lint` to check ESLint errors
   - Run `pnpm run typecheck` to check TypeScript type errors
   - Run `pnpm run format:check` to check formatting issues
   - Run `pnpm run lint:md` to check Markdown formatting
   - Run `read_lints` tool to check all lint errors

3. If issues remain after fixing, continue fixing until all issues are resolved

4. Ensure fixed code still passes tests:
   - Run `pnpm test` to ensure tests pass
   - If tests fail, fix code until tests pass

**Fix Principles:**

- Do not just report issues - you must actually fix the code
- Prioritize fixing Critical and High priority issues
- After fixing, must run verification tools to ensure no issues remain
- Fixes must conform to project code standards and best practices
- If fixes require major refactoring, refactor first then fix

### 7. Issue Priority Levels

**Critical** (Must fix before merge):

- Panic risks (unwraps, indexing without bounds check, `block_on` in async)
- Data loss potential
- Security vulnerabilities
- Breaking functionality
- Memory leaks or resource leaks

**High** (Should fix soon):

- Bug potential (race conditions, null handling, missing error handling)
- Performance issues (blocking operations, unnecessary re-renders)
- Major architectural issues (tight coupling, mixed concerns)
- Type safety issues (`any` types in TypeScript)

**Medium** (Consider fixing):

- Code smells (long functions, duplication, magic numbers)
- Testability concerns (hard to test due to coupling)
- Minor architectural improvements
- Code style inconsistencies

**Low** (Nice to have):

- Code style improvements
- Minor optimizations
- Documentation improvements
- Refactoring suggestions

## Output Format

Review report should be organized as follows (concise and focused):

```markdown
## Code Review Summary

### Overall Assessment
[Brief summary of code quality, architectural improvements, highlights, etc.]

### Fixed Issues
1. **[`file.ts:line`]** [Issue Title] - **FIXED**
   - **Fix**: [What was fixed]
   - **Verification**: [What verification tools were run, results]

### High Priority Issues (Fixed)
1. **[`file.ts:line`]** [Issue Title] - **FIXED**
   - **Fix**: [What was fixed]
   - **Verification**: [What verification tools were run, results]

### Medium Priority Issues (Fixed)
1. **[`file.tsx:line`]** [Issue Title] - **FIXED**
   - **Fix**: [What was fixed]

### Verification Results
- ESLint: [Pass/Fail]
- TypeScript: [Pass/Fail]
- Formatting: [Pass/Fail]
- Markdown: [Pass/Fail]
- Tests: [Pass/Fail]

### Fix Summary
- Critical issues: [X] found, [X] fixed
- High issues: [X] found, [X] fixed
- Medium issues: [X] found, [X] fixed
- Low issues: [X] found, [X] fixed
```

**IMPORTANT: Report must show all issues have been fixed and provide verification results.**

## Notes

- **CRITICAL REQUIREMENT: Must fix all discovered issues, not just report them**
- Focus on the actual changes made, not the entire codebase
- Be constructive and specific
- **Auto-fix all issues**: Don't just report problems - fix them immediately
- **Priority order**: Critical → High → Medium → Low
- **Verification required**: After fixing, run lint, typecheck, format checks, and tests
- **Code Simplicity**: Prefer simple, straightforward solutions over clever or complex ones
  - Write code that is easy to understand and maintain
  - Avoid unnecessary abstractions or design patterns
  - Don't add complexity "just in case" - solve actual problems as they arise
  - Simple code is easier to debug, test, and modify
  - Question overly complex solutions - can this be done more simply?
- Consider the project's existing patterns and conventions:
  - Rust: Edition 2024, use `anyhow::Result`, snake_case for functions
  - TypeScript: Strict mode, functional components, hooks for state
  - Tauri: Use commands for backend communication, events for notifications
- **Fix workflow**:
  1. Identify issue → 2. Fix code → 3. Verify with tools → 4. Run tests → 5. Report fixed
- **Never skip fixes**: All Critical and High priority issues must be fixed before reporting
- **Verify before reporting**: Always run `make check` or equivalent after all fixes
