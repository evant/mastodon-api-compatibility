The following outlines the software `name` and `version` format for the
[nodeinfo](https://github.com/jhass/nodeinfo) endpoint for various fedi
software.

```admonish warning
Instances may customize the version so this list is not exhaustive. You should
never match against a version exactly. In most cases the version will start
with the api version it is compatible with.
```

## Mastodon

### Name

`mastodon`

### Version

Uses semantic versioning. Pre-releases are appended with `-alpha.[n]`,
`-beta.[n]`, or `-rc.[n]`. Nightlies are appended with `-nightly.[YYYY-MM-DD]`.

#### Examples
- `4.3.1`
- `4.4.0-alpha.1`
- `4.3.0-beta.2`
- `4.3.0-rc.1`
- `4.4.0-nightly.2024-10-28`

## Mastodon Glitch Edition (glitch-soc)

### Name

`mastodon`

### Version

Uses a Mastodon version with `+glitch` appended. The mastodon version may be a pre-release or nightly.

#### Examples
- `4.1.4+glitch`
- `4.3.0-alpha.3+glitch`
- `4.3.0-nightly.2024-08-25+glitch`

## GoToSocial

### Name

`gotosocial`

### Version

Versions `0.14.0` and above use semantic versioning. Pre-releases are appended
a git hash `+git-[hash]` which may be prefix with `-rc[n]` for release
candidates or `-SNAPSHOT` for nightlies.

### Examples

#### 0.14.0+

- `0.17.0`
- `0.14.0+git-7bc536d`
- `0.15-rc1+git-65b5366`
- `0.17.0-SNAPSHOT+git-f037663`
