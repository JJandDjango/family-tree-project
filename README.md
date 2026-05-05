# family-tree-project

Interactive family tree viewer built in Rust + Leptos, compiled to WebAssembly, deployed via Cloudflare Pages.

## Stack

- **Rust 1.95** + **Leptos 0.8** (CSR mode) → WebAssembly
- **Trunk** for build orchestration
- Pure SVG rendering with a recursive tree layout (parent centered above children)
- Tree data bundled at compile time via `include_str!`
- **Cloudflare Pages** hosting, auto-deployed on push to `main`

## Run locally

```bash
cd crates/family-tree-app
trunk serve
```

Visit `http://127.0.0.1:8080`.

## Project layout

```
family-tree-project/
├── Cargo.toml              # workspace
├── rust-toolchain.toml     # Rust 1.95.0 + wasm32 target
├── build.sh                # Cloudflare Pages entrypoint
├── DEPLOY.md               # CFP dashboard config
└── crates/family-tree-app/
    ├── Cargo.toml
    ├── Trunk.toml
    ├── index.html
    ├── styles.css
    ├── public/tree.json    # the family data
    └── src/
        ├── main.rs         # Leptos app, components, SVG render
        ├── data.rs         # Person/Database structs (serde)
        └── layout.rs       # Recursive tree layout
```

## Updating the tree

Edit `crates/family-tree-app/public/tree.json`. Rebuild via `trunk serve` (dev) or `trunk build` (prod). The JSON is bundled into the WASM binary at compile time.

## Data shape

```json
{
  "root": "person_id",
  "people": {
    "person_id": {
      "name": "Full Name",
      "birth": "1900",
      "death": "1980",
      "spouse": "other_id",
      "children": ["child_a", "child_b"],
      "notes": "Free-form text shown in the side panel."
    }
  }
}
```

The `root` field names the topmost person. Children of a node are listed under that node's `children` array. Spouses are referenced by id and rendered inline on the primary person's card. The current layout treats the tree as rooted at one person; ancestor charts (looking up from a descendant) are not yet supported.
