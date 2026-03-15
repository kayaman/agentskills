---
name: semver
description: Enforces Semantic Versioning 2.0.0 (semver.org) rules. Use when choosing a version bump type for a release, validating version strings, tagging releases, handling pre-release or build metadata, managing deprecations, or advising on version precedence.
---

# Semantic Versioning 2.0.0

Reference: https://semver.org/spec/v2.0.0.html

## Version Format

```
MAJOR.MINOR.PATCH[-pre-release][+build]
```

| Segment | When to increment | Reset rule |
|---|---|---|
| MAJOR (X) | Backward-incompatible API change | Resets MINOR and PATCH to 0 |
| MINOR (Y) | New backward-compatible functionality or deprecation notice | Resets PATCH to 0 |
| PATCH (Z) | Backward-compatible bug fix only | — |

## Normative Rules

1. A public API MUST be declared (in code or documentation).
2. Version MUST be `X.Y.Z` — non-negative integers, no leading zeroes, each MUST increase numerically.
3. Once a version is released, its contents MUST NOT be modified. Any change MUST be a new version.
4. `0.y.z` is for initial development — anything MAY change, public API SHOULD NOT be considered stable.
5. `1.0.0` defines the first stable public API.
6. PATCH MUST be incremented for backward-compatible bug fixes only.
7. MINOR MUST be incremented when new backward-compatible functionality is introduced or any public API is deprecated. MAY include patch-level changes.
8. MAJOR MUST be incremented for any backward-incompatible change. MAY include minor and patch-level changes.
9. A released version MUST NOT be altered — even to fix a bug. Release a new version instead.

## Pre-release Versions

- Append a hyphen and dot-separated identifiers: `1.0.0-alpha`, `1.0.0-alpha.1`, `1.0.0-0.3.7`
- Identifiers MUST use only `[0-9A-Za-z-]`, MUST NOT be empty, numeric identifiers MUST NOT have leading zeroes.
- Pre-release has **lower** precedence than the associated normal version: `1.0.0-alpha < 1.0.0`

## Build Metadata

- Append a `+` and dot-separated identifiers: `1.0.0+20130313`, `1.0.0-beta+exp.sha.5114f85`
- Build metadata MUST be ignored when determining precedence.

## Precedence

Compare identifiers left to right: MAJOR → MINOR → PATCH → pre-release (build metadata ignored).

Pre-release precedence example:
```
1.0.0-alpha < 1.0.0-alpha.1 < 1.0.0-alpha.beta < 1.0.0-beta < 1.0.0-beta.2 < 1.0.0-beta.11 < 1.0.0-rc.1 < 1.0.0
```

Pre-release identifier comparison rules:
- Numeric-only identifiers are compared numerically.
- Alphanumeric identifiers are compared lexically in ASCII order.
- Numeric identifiers have lower precedence than alphanumeric identifiers.
- A larger set of pre-release fields has higher precedence when all preceding identifiers are equal.

## Deprecation Workflow

1. Issue a new **MINOR** release that marks functionality as deprecated (with documentation update).
2. Remove the functionality in a subsequent **MAJOR** release.
3. Never remove deprecated functionality without a prior minor release that contains the deprecation notice.

## Common Pitfalls

| Pitfall | Correct behaviour |
|---|---|
| `v1.2.3` is not a SemVer string | The SemVer string is `1.2.3`; the `v` prefix is a tag naming convention, not part of the spec |
| Modifying a released version | MUST NOT be done — release a new version |
| Leading zeroes (`01.2.3`, `1.02.3`) | MUST NOT appear in any numeric identifier |
| Bumping MAJOR for every deprecation | Deprecations warrant a MINOR bump; removal warrants a MAJOR bump |
| Skipping the deprecation MINOR release | At least one MINOR release with the deprecation MUST precede removal |

## Initial Development Guidance

- Start at `0.1.0` and increment MINOR for each subsequent release.
- Increment PATCH for backward-compatible bug fixes within the `0.y.z` range.
- Promote to `1.0.0` when the software is used in production or has a stable API that users depend on.

## Validation Regex

**Named groups (PCRE / Python / Go):**
```
^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

**Numbered groups (ECMAScript / JavaScript):**
```
^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

## Checklist

When releasing a new version, verify:

- [ ] The change type is correctly identified (bug fix → PATCH, new feature/deprecation → MINOR, breaking change → MAJOR)
- [ ] Version string contains no leading zeroes
- [ ] Previously released version contents have not been modified
- [ ] If deprecating, a MINOR release is made before any future MAJOR removal
- [ ] Git tag uses the `vX.Y.Z` convention (e.g. `git tag v1.2.3`) while the SemVer string itself is `1.2.3`
- [ ] CHANGELOG or release notes document the nature of the change
