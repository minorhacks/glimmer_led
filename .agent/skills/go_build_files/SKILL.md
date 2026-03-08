# Skill: Update Go Build Files

## Description

This skill updates the Go build files for the project.

## When to Activate
- When non-stdlib import statements are added/removed/modified in any Go source
  file

## Instructions

1. Ensure that `BUILD.bazel` files are up-to-date with imports. Run:

   ```
   bazel run //:go_deps.update
   ```
