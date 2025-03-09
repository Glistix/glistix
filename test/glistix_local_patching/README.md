# Patch Project

Test patching dependencies and their transitive dependencies with local packages.

Root has a dependency which it patches to `local_dep`, and `local_dep` has a dependency
which root patches to `other_local_dep`. Root then calls a function from (patched) `local_dep`
which calls a function from (patched) `other_local_dep` to print a message.

## Quick start

```sh
glistix run
glistix test
```
