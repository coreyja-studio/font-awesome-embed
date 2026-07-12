# font-awesome-embed

Rust proc macro crate that embeds Font Awesome SVGs into binaries at compile time. Supports every Font Awesome 7 family — including Pro packs like Notdog, Pixel, Chisel, Etch, and Slab — via the Font Awesome GraphQL API.

## Usage

```rust
use font_awesome_embed::fa;

// Returns &'static str containing the SVG
let svg: &str = fa!("house", solid);

// FA7 family packs via `family = ...`
let svg: &str = fa!("star", solid, family = notdog);
let svg: &str = fa!("star", regular, family = pixel);
```

### Maud integration

With the `maud` feature enabled:

```rust
use font_awesome_embed::maud::fa;

html! {
    button { (fa!("house", solid)) " Home" }
    span { (fa!("star", solid, family = notdog)) }
}
```

## Setup

1. Add the dependency (git, not yet on crates.io):

   ```toml
   [dependencies]
   font-awesome-embed = { git = "https://github.com/coreyja-studio/font-awesome-embed", features = ["maud"] }
   ```

2. Set `FONT_AWESOME_TOKEN` to a Font Awesome API token (from
   [fontawesome.com/account/general](https://fontawesome.com/account/general) — needs the
   `svg_icons_pro` scope for Pro families). With mise, in `.mise.toml`:

   ```toml
   [env]
   FONT_AWESOME_TOKEN = "{{ exec(command='mull secrets get font-awesome') }}"
   ```

3. In CI, either provide the token as a secret or build with the `test-icons`
   feature to skip the network entirely (placeholder SVGs, no token needed):

   ```sh
   cargo test --features test-icons
   ```

## Families and styles

Font Awesome 7 has two independent dimensions. Style is the second positional
argument; family defaults to `classic` and is overridden with `family = ...`
(or the `FA_DEFAULT_FAMILY` env var).

| Family | Available styles |
|---|---|
| `classic` (default) | `solid`, `regular`, `light`, `thin`, `brands` |
| `duotone`, `sharp`, `sharp_duotone` | `solid`, `regular`, `light`, `thin` |
| `chisel`, `jelly`, `jelly_duo`, `jelly_fill`, `pixel`, `slab`, `slab_duo`, `slab_press`, `slab_press_duo` | `regular` |
| `etch`, `mosaic`, `notdog`, `notdog_duo`, `vellum` | `solid` |
| `graphite` | `thin` |
| `thumbprint` | `light` |
| `utility`, `utility_duo`, `utility_fill`, `whiteboard` | `semibold` |

Requesting a combination the release doesn't ship (e.g. `fa!("star", solid, family = pixel)`)
is a compile error naming the family and style.

## Features

- `test-icons` — Returns a static placeholder SVG for all icons (no network or token needed)
- `maud` — Enables the `maud` module with a convenience macro that wraps icons in `PreEscaped`

## Configuration

| Env var | Effect |
|---|---|
| `FONT_AWESOME_TOKEN` | Font Awesome API token used at compile time (required on cache miss unless `test-icons`) |
| `FA_VERSION` | Font Awesome release to fetch from (default: pinned, currently `7.3.0`) |
| `FA_DEFAULT_FAMILY` | Family used when no `family = ...` is given (default: `classic`) |
| `FA_CACHE_DIR` | Cache directory override (default: `$OUT_DIR/fa-cache`, else a shared temp dir) |

Fetched SVGs are cached on disk (keyed by release version, family, and style), so each
icon is fetched once per machine, not once per build. SVGs are post-processed for
embedding: `fill="currentColor"`, `aria-hidden="true"`, `width`/`height` of `1em`, and a
`fa-svg` class for global styling.

## Committed cache — build with no token, no network

Point `FA_CACHE_DIR` at a directory inside your repo and commit it. Cache hits skip
the token check and the network entirely, so CI, Docker builds, and PR review apps
need no secret — you only need `FONT_AWESOME_TOKEN` locally the first time you add a
new icon (which writes a new file to the cache; commit it with your change).

In the consuming repo's `.cargo/config.toml` (`relative = true` makes cargo pass an
absolute path, since proc macros make no guarantee about the working directory):

```toml
[env]
FA_CACHE_DIR = { value = ".fa-cache", relative = true }
```

Note: the cache stores Font Awesome's SVGs. Committing Free icons to a public repo
is fine (CC BY 4.0 — keep attribution somewhere reasonable); commit Pro icons only
to repos that are private to your license.
