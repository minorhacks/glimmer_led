# Skill: Update Go Dependencies

## Description

This skill updates the Go dependencies for the project.

## When to Activate
- When novel Go dependencies are added to the project

## Instructions

1. Ensure that `go.mod` is up-to-date with imports across all Go source files.
   Run:

   ```
   bazel run @rules_go//go -- mod tidy
   ```
2. Ensure that `BUILD.bazel` files are up-to-date with imports. Run:

   ```
   bazel run //:go_deps.update
   ```