# font-awesome-embed

Rust proc macro crate that embeds Font Awesome SVGs into binaries at compile time.

## Usage

```rust
use font_awesome_embed::fa;

// Returns &'static str containing the SVG
let svg: &str = fa!("house", solid);
```

### Maud integration

With the `maud` feature enabled:

```rust
use font_awesome_embed::maud::fa;

html! {
    button { (fa!("house", solid)) " Home" }
}
```

## Features

- `test-icons` — Returns a static placeholder SVG for all icons (no network or token needed)
- `maud` — Enables `maud` module with a convenience macro that wraps icons in `PreEscaped`

## Configuration

Set `FONT_AWESOME_TOKEN` to your Font Awesome API token. All icons (free and pro) are fetched via the Font Awesome GraphQL API at compile time.
