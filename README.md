# Pixel Sorter (WIP)

A web app for image pixel sorting using WebAssembly in a web worker.

For explanation and credits see this [summary of what pixel sorting is](http://satyarth.me/articles/pixel-sorting/).

![Example image with sorted pixels](assets/example.jpg)

# Demo

https://pixel-sorter.plonq.org

## Tech Stack

- [Rust](https://www.rust-lang.org)
- [Yew](https://yew.rs) - frontend framework

## Development

If you don't have [Trunk](https://trunkrs.dev) installed, install with:

```bash
cargo install trunk
```

Use Trunk to start a local dev server:

```bash
trunk serve
```

### Deployment

Use Trunk to build for production:

```bash
trunk build --release
```

Output will be in the `dist` directory, which can be statically hosted
anywhere.
