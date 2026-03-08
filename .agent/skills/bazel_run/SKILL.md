# Skill: Run programs built with Bazel

## Description
Use this skill to run any program built with Bazel.

## When to Activate
- When you want to run a program built with Bazel.

## Instructions

1. Identify the corresponding BUILD target for the program you want to run.
2. Run the program under `bazel run`, adding a `--` after the target name to pass arguments to the program.

## Examples

```bash
bazel run //:glimmer_server -- --port=8080
```
