# Revset Tree & Diff Language + Tree-based Copy Tracking

This document summarizes the architectural shift in Jujutsu from a commit-centric diff and copy-tracking system to a first-class, tree-based system. This transition enables accurate diffing and rename detection for virtual merges and arbitrary tree expressions.

## 1. Core Objective

The goal was to decouple Jujutsu's diffing, status, and copy-tracking engines from the `Commit` object. By standardizing on `TreeId` and `MergedTree` as the primary primitives, we allow the engine to operate on:
- Virtual merge states (e.g., `merged_of(A, B)`).
- Specific conflict sides.
- Synthetic trees (rebased or reverted states).
- Arbitrary revset expressions coerced into trees.

## 2. Revset Language Extensions

We introduced a three-tier type hierarchy in the revset parser to handle queries safely:

1. **`UserQueryExpression`**: The top-level container.
2. **`UserTreeExpression`**: Produces a `MergedTree`.
3. **`UserDiffExpression`**: Produces a diff between two trees.

### New Built-in Functions

#### Trees
- `tree(<revset>)`: Coerces a revset (single commit) into its tree.
- `merged_of(<trees...>)`: Produces a virtual merge tree from multiple sources.
- `merged_parents(<revset>)`: Produces the merged state of a commit's parents.
- `conflict_side(<revset>, <n>)`: Extracts a specific side of a conflict.
- `rebase_of(<base_tree>, <revsets>)`: Previews a rebase state.

#### Diffs
- `diff(<from_tree>, <to_tree>)`: A formal diff expression between any two trees.
- `invert(<diff>)`: Inverts a diff.

> [!NOTE]
> A `Revset` can be implicitly coerced into a `Diff` (usually `diff(parents(A), A)`) when passed to commands like `jj diff -r`.

## 3. Tree-based Copy Tracking Refactor

Previously, copy/rename detection was tied to `CommitId`. This was refactored to use `TreeId` directly.

### Backend API Changes
- **`CopyRecord`**: Removed `target_commit` and `source_commit` fields. These were metadata that the diff engine did not use and which didn't exist for virtual trees.
- **`Backend::get_copy_records`**: Changed signature from `(&[RepoPathBuf], &CommitId, &CommitId)` to `(&[RepoPathBuf], &TreeId, &TreeId)`.
- **`GitBackend`**: Refactored to use `gix` tree diffing natively, resolving `TreeId` to `gix::Tree` objects without requiring a commit object.

### CLI Implementation
- **Copy Collection Logic**: In `cli/src/commands/diff.rs` and `cli/src/diff_util.rs`, the logic now iterates over all pairs of `TreeId`s within the source and destination `MergedTree`s.
- This ensures that if a destination is a virtual merge (multiple `TreeId`s), copy records are checked against the source tree for each component of the merge.

## 4. CLI Interface Changes

- **`jj file show`**: Removed the `--merge` flag. It now accepts a `TreeExpression`. To show a merged file, use `jj file show "merged_of(A, B)"`.
- **`jj diff`**: Removed the `--merge` flag.
    - `--from` and `--to` now evaluate as `TreeExpression`.
    - `-r` evaluates as a `DiffExpression`.
- **`jj status`**: Updated to use the tree-based copy tracking engine, enabling rename detection in more complex scenarios.

## 5. Technical Implementation Details for Recreatability

### Type Hierarchy (Pseudo-code)
```rust
enum UserQueryExpression {
    Revset(RevsetExpression),
    Tree(UserTreeExpression),
    Diff(UserDiffExpression),
}

enum UserTreeExpression {
    CommitTree(RevsetExpression),
    Merged(Vec<UserTreeExpression>),
    // ...
}
```

### Copy Record Collection (CLI)
When diffing two `MergedTree` objects (let's call them `From` and `To`):
1. Get the list of `TreeId`s from `From` and `To`.
2. For every `f_id` in `From` and `t_id` in `To`:
    - Call `store.get_copy_records(paths, f_id, t_id)`.
3. Aggregate and deduplicate the resulting `CopyRecord`s.

### Files Modified and Rationale
- `lib/src/backend.rs`: Trait definition update.
- `lib/src/git_backend.rs`: Direct `gix` tree diffing implementation.
- `cli/src/commands/diff.rs`: Logic to handle tree pairs instead of single commits.
- `cli/src/diff_util.rs`: Helper utilities for tree-based copy record retrieval.

## 6. Verification Results
- **Unit Tests**: Updated `test_git_backend.rs` to verify that `get_copy_records` works with arbitrary `TreeId`s.
- **Integration Tests**: Verified that `jj diff --from merged_of(A, B) --to C` correctly identifies renames occurring relative to the virtual merge state.
- **Type Safety**: The revset parser now prevents logically invalid queries (e.g., `parents(tree(A))`) at compile-time/parse-time.
