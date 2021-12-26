# mozaic4

Because third time's the charm!

# pwcli
Note: this project is under active development. All file and configuration formats will take some time to stabilize, so be prepared for breakage when you upgrade to a new version.
## Building

The cli comes with a local webserver for visualizing matches.
Therefore, you'll have to build the web application first, so that it can be embedded in the binary.

You will need:
- rust
- wasm-pack
- npm

First, build the frontend:
```bash
cd web/pw-frontend
npm install
npm run build-wasm
npm run build
```

Then build the backend:
```bash
cd planetwars-cli
cargo build --bin pwcli --release
```

You can install the binary by running
```bash
cargo install --path .
```

## Getting started
First, initialize your workspace:
```bash
pwcli init my-planetwars-workspace
```
This creates all required files and directories for your planetwars workspace:
- `pw_workspace.toml`: workspace configuration
- `maps/`:  for storing maps
- `matches/`: match logs will be written here
- `bots/simplebot/` an example bot to get started 

All subsequent commands should be run from the root directory of your workspace.

Try playing an example match:
```bash
pwcli run-match hex simplebot simplebot
```

You can now watch a visualization of the match in the web interface:
```bash
pwcli serve
```

You can now try writing your own bot by copying the `simplebot` example. Don't forget to add it in your workspace configuration!
