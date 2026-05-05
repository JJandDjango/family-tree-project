# Deploy

Cloudflare Pages hosts the family-tree-app demo.

## Pages dashboard config

| Setting | Value |
|---|---|
| Repo | `JJandDjango/family-tree-project` |
| Production branch | `main` |
| Framework preset | None |
| Root directory | `/` (repo root) |
| Build command | `bash build.sh` |
| Build output directory | `crates/family-tree-app/dist` |
| Environment variables | (none) |

## How the build works

`build.sh` is the Pages entrypoint. It:

1. Installs Rust `1.95.0` via rustup (minimal profile) if `cargo` is not already on PATH.
2. Adds the `wasm32-unknown-unknown` target.
3. Downloads the pinned `trunk 0.21.14` binary tarball from the trunk-rs GitHub release (no `cargo install`).
4. Runs `trunk build` from `crates/family-tree-app/`, producing `dist/`.

Toolchain pins live at the top of `build.sh` — bump them in lockstep with the local dev environment.

## Local development

```bash
cd crates/family-tree-app
trunk serve
```

Then visit `http://127.0.0.1:8080`.

To update the family tree data: edit `crates/family-tree-app/public/tree.json` and rebuild. The JSON is bundled into the WASM binary at compile time via `include_str!`.
