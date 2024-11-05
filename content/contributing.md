# Contributing

## Setup

Clone the repository

```bash
git clone 'https://github.com/evant/mastodon-api-compatibility.git'
cd mastodon-api-compatibility
```

You need rust to build & run the project. The most convenient way to install it
is via [rustup](https://rustup.rs/).

You can then build the website locally with

```bash
cargo run
```

You can serve the website with any static file server. If you have python you can run

```bash
cd public
python3 -m http.server
```

To re-run after changes you can use
[cargo-watch](https://crates.io/crates/cargo-watch).

Tip: if you aren't editing the rust code, you can build with release.

```bash
cargo install cargo-watch
cargo watch -x 'run --release'
```

## Data format

The data is stored in a series of toml files in the `data` directory. There is a
`software.toml` file that defines which software is available. Everything else
defines the api.

### software.toml

Has a table to software with some data for each. The table key is what to use to
reference it in other files.

```toml
 # the software key
[mastodon]
# (required) The name to display.
name = "Mastodon"
# (optional) A default link to the software's documentation.
# Used when an api-specific link isn't provided.
docs = "https://docs.joinmastodon.org/"
```

### [api-name].toml

Each file defines a section of the api. The file name is used as the name in the
side-bar. Note that you can nest files in directories to create sub-sections.
For example, if you have `accounts.toml` and `accounts/bookmarks.toml`,
bookmarks will be nested under accounts.

If you aren't familiar with toml, you can read about it [here](https://toml.io). A key thing to
note is these two table formats are equivalent:

```toml
id = { mastodon = "0.0.0", gotosocial = "0.1.0" }
```

```toml
id.mastodon = "0.0.0"
id.gotosocial = "0.1.0"
```

There are two types of top-level sections. `api` and `entity`.

#### Api

```toml
[[api]]
# (required) The request, this is recommend but not required to be unique.
# Includes the http verb and the path.
request = "GET /api/v1/accounts/:id"
# (optional) The software that supports this api.
# You can either use the short-form: mastodon = "0.0.0"
# or the long form: mastodon = { version = "0.0.0", docs = "..." }.
# More details about version definitions below.
[api.software]
mastodon.version = "0.0.0"
mastodon.docs = "https://docs.joinmastodon.org/methods/accounts/#get"
mastodon.note = "returns 410 if account is suspended in 2.4.0, returns with suspended=true instead of 410 in 3.3.0"
# (optional) The parameters for this api.
# Each parameter includes a table of software versions.
[api.params]
id.mastodon = "0.0.0"
# (optional) The responses for this api.
# This example has the response declared inline.
[api.response]
id.mastodon = "0.1.0"
username.mastodon = "0.1.0"
```

If only one response is defined, it's assumed to be for a 200 response. You may define multiple
responses with an `http-code`.
(Note the additional square baces to turn it into an array).

```toml
[[api]]
request = "GET /api/v1/accounts/:id"
[api.software]
mastodon.version = "0.0.0"
[[api.response]]
id.mastodon = "0.1.0"
username.mastodon = "0.1.0"
[[api.response]]
http-code = 404
error.mastodon = "0.1.0"
```

A response may also have nested keys or array.

- You can append `[]` to indicate an array,
- and `[name]` to indicate a nested key.
- If the nested keys are an arbitrary map, you can use `[:key][name]`.

```toml
# An array: [field, field, ..]
"fields[]".mastodon = "2.4.0"
# A nested key: "source": { "privacy": "public" }
"source[privacy]".mastodon = "1.5.0"
# An arbitrary map: "details": { "some-key": { "name": "value" } }
"details[:key][name]".mastodon = "3.4.0"
```

#### Versions

You can define a software version in a couple ways. The shortest form is just a
string.

```toml
mastodon = "0.0.0"
```

The long form allows you to define additional information.

```toml
# (required) The version.
mastodon.version = "0.0.0"
# (optional) A documentation link for this api.
mastodon.docs = "https://docs.joinmastodon.org/"
# (optional) The version this api was deprecated.
mastodon.deprecated = "1.2.3"
# (optional) The version this api was removed.
mastodon.removed = "2.2.3"
# (optional) An arbitrary note about this api or attribute.
mastodon.note = "behavior changed in 1.4.3"
```

Since only a couple of these are likely to be used at a time, you can use the
inline table format.

```toml
id.mastodon = { version = "0.0.0", deprecated = "4.0.0" }
```

#### Entity

While above can technically define any api response, it is often useful to pull
out common attributes so they can be re-used. You can do this by defining an
entity.

```toml
# Defines an entity with key 'account'. Entity keys must be globally unique.
[entity.account]
# (optional) The name to display. If not provided, it is derived from the key.
name = "Account"
# (optional) The software that supports this entity.
[entity.account.software]
mastodon.version = "0.0.0"
# (optional) The attributes for this entity. These are declared the same way as in
# a response.
[entity.account.attributes]
id.mastodon = "0.0.0"
username.mastodon = "0.0.0"
```

You can then reference this entity for a specific attribute.

```toml
[[api]]
request = "GET /api/v1/accounts/verify_credentials"
[api.response]
role.entity = "role"
role.version = "0.0.0"
```

Or an entire response.

```toml
[[api]]
request = "GET /api/v1/accounts/:id"
response = "account"
```

This works with arrays as well.

```toml
[[api]]
request = "GET /api/v1/accounts/:id"
[api.response]
"fields[]".entity = "field"
"fields[]".mastodon = "2.4.0"
```

If the whole api response is an array, you can use the following syntax.

```toml
[[api]]
request = "GET /api/v1/accounts"
response = "account[]"
```

Finally, entities can be used to define enum values. This is done by defining
`values` instead of `attributes`.

```toml
[entity.visibility]
name = "Visibility"
[entity.visibility.values]
public.mastodon = "0.0.0"
```
