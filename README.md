# 🛡️ Cyber Defender — Rust + WebAssembly

An original browser arcade game built with **Rust + WebAssembly** and HTML Canvas.

You control a cyber defense ship, destroy incoming drones, survive as long as possible, and beat your local high score.

## ✨ Features

- Rust game logic compiled to WebAssembly
- HTML5 Canvas rendering
- 3 enemy types with different durability/speed
- Increasing difficulty and levels
- Score + local high score
- 3 lives
- Keyboard, mouse, and touch controls
- Pause/resume
- Restart after game over
- Particle effects and screen shake
- No backend
- No database
- No API
- High score stored locally in browser storage
- GitHub Pages deployment workflow included

## 🎮 Controls

| Action | Keyboard | Mouse / Touch |
|---|---|---|
| Move | WASD / Arrow Keys | Move/drag pointer |
| Fire | Space | Hold mouse / touch |
| Pause | P / Esc | — |
| Restart | R after game over | — |

## 🧰 Requirements

Install:

- Rust: https://www.rust-lang.org/tools/install
- `wasm-pack`: https://rustwasm.github.io/wasm-pack/installer/

Install wasm-pack:

```bash
cargo install wasm-pack
```

## ▶️ Run locally

From the project root:

```bash
wasm-pack build --target web --out-dir web/pkg
```

Then serve the `web` folder with a local HTTP server.

For example, with Python:

```bash
cd web
python -m http.server 8080
```

Open:

```text
http://localhost:8080
```

> Do not open `web/index.html` directly with `file://`. Browsers block some WebAssembly module loading behavior when a page is opened as a local file.

## 🌐 GitHub Pages

This repository includes:

```text
.github/workflows/deploy-pages.yml
```

The workflow automatically:

1. Checks out the repository
2. Installs Rust
3. Installs wasm-pack
4. Builds the WebAssembly package
5. Uploads the `web` directory
6. Deploys it to GitHub Pages

### First-time GitHub setup

1. Create a GitHub repository.
2. Upload/push all project files.
3. Open **Settings → Pages**.
4. Under **Build and deployment**, choose **GitHub Actions**.
5. Push to the `main` branch.
6. Open the **Actions** tab and wait for `Deploy to GitHub Pages` to finish.
7. GitHub will show the Pages URL.

## 📁 Structure

```text
cyber-defender/
├── .github/
│   └── workflows/
│       └── deploy-pages.yml
├── src/
│   └── lib.rs
├── web/
│   ├── index.html
│   └── style.css
├── Cargo.toml
└── README.md
```

`web/pkg/` is generated automatically by `wasm-pack`, so it is intentionally not included in the source ZIP.

## 💾 Save system

The game stores the best score in the browser's local storage under:

```text
cyber_defender_best
```

There is no account system or online leaderboard.

## 📜 License

MIT. Use, modify, and learn from the project freely.
