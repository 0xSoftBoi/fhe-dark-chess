```markdown
# fhe-dark-chess Development Patterns

> Auto-generated skill from repository analysis

## Overview
This skill teaches you the development patterns and conventions used in the `fhe-dark-chess` JavaScript codebase. The repository focuses on implementing chess logic with a focus on privacy (FHE: Fully Homomorphic Encryption), and is written without a framework. You'll learn about file organization, code style, import/export patterns, and how to write and run tests in this project.

## Coding Conventions

### File Naming
- Files use **PascalCase** (e.g., `GameBoard.js`, `MoveValidator.js`).

### Imports
- Use **relative import paths**.
  ```js
  import { ChessPiece } from './ChessPiece';
  ```

### Exports
- Use **named exports** exclusively.
  ```js
  // In ChessPiece.js
  export function ChessPiece() { ... }
  ```

### Commit Messages
- Commit messages are **freeform**, sometimes prefixed (e.g., `onchain`).
- Average commit message length: ~86 characters.

## Workflows

### Adding a New Module
**Trigger:** When you need to add a new feature or logic component.
**Command:** `/add-module`

1. Create a new file using PascalCase (e.g., `NewFeature.js`).
2. Implement your logic using named exports.
   ```js
   export function NewFeature() { ... }
   ```
3. Import the module where needed using a relative path.
   ```js
   import { NewFeature } from './NewFeature';
   ```
4. Write a corresponding test file (see Testing Patterns).

### Writing a Test
**Trigger:** When you add or update functionality.
**Command:** `/write-test`

1. Create a test file with the pattern `*.test.*` (e.g., `GameBoard.test.js`).
2. Write your test cases using your preferred testing framework (framework is not specified).
3. Ensure your test covers all exported functions.
4. Run your test suite (see below).

### Running Tests
**Trigger:** Before committing or merging code.
**Command:** `/run-tests`

1. Identify all files matching `*.test.*`.
2. Run them using your chosen test runner (framework is not specified; use `node` or a test tool as appropriate).
3. Review output and fix any failing tests.

## Testing Patterns

- **Test files** are named with the pattern `*.test.*` (e.g., `MoveValidator.test.js`).
- The testing framework is **unknown**; structure your tests according to your team's conventions or introduce a framework as needed.
- Place test files alongside the modules they test or in a dedicated test directory.

**Example:**
```js
// GameBoard.test.js
import { GameBoard } from './GameBoard';

test('should initialize board correctly', () => {
  const board = new GameBoard();
  // assertions here
});
```

## Commands
| Command        | Purpose                                 |
|----------------|-----------------------------------------|
| /add-module    | Scaffold a new module with conventions  |
| /write-test    | Create a test file for a module         |
| /run-tests     | Run all test files in the codebase      |
```
