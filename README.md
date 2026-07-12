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

// Extra CSS classes on the <svg> element via `class = ...`
let svg: &str = fa!("xmark", solid, class = "text-xl text-red-400");
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

## CI and Docker builds

Provide `FONT_AWESOME_TOKEN` wherever the consuming crate is compiled: an Actions
secret in CI workflows, and a BuildKit secret in Dockerfiles
(`RUN --mount=type=secret,id=FONT_AWESOME_TOKEN ... cargo build`, with
`flyctl deploy --build-secret` / `docker build --secret` at the call site).

**Do not commit the cache directory.** Not redistributing Font Awesome's SVGs —
in particular Pro icons — is a design goal of this crate: the repo holds only icon
*names*, and the SVGs are fetched under your own license at build time. `FA_CACHE_DIR`
exists so ephemeral environments can *persist* the cache between builds (e.g. point it
inside a directory your CI already caches, like `target/`), not so it can be checked in.
