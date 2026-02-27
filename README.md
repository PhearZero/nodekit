## nodekit

Manage Algorand nodes from the command line

> [!IMPORTANT]
> This project is just for fun and is currently a work in progress.

### Getting Started

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition supported)
- [Trunk](https://trunkrs.dev/#install) (for web/WASM mode)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`

#### Cloning the Repository

```bash
git clone https://github.com/phearzero/nodekit.git
cd nodekit
```

### Running the CLI (Native)

To run the Nodekit TUI natively in your terminal:

```bash
# Run with default settings (connects to localhost:8080)
cargo run --bin nodekit

# Run with custom node configuration
cargo run --bin nodekit -- --url http://your-node:8080 --token YOUR_TOKEN
```

### Running the Web UI (WASM)

Nodekit can be served as a web application using [Trunk](https://trunkrs.dev/).
By default it uses http://localhost:8080 with AAAA... token. It is hard coded for now

```bash
# From the project root
cd nodekit
trunk serve -p 8081
```

Then open `http://localhost:8081` in your browser.

