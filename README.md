# Breathly

> *Let your PC breathe.*

Breathly is a free, open-source, lightweight tool that helps anyone — technical or not — understand **why their PC is slow** and fix it safely, on Windows and Linux (macOS after v1.0).

- 🩺 **Diagnose** — RAM, swap, disk, heat, and what's eating your resources, explained in plain words.
- 🧹 **Free up space safely** — caches, old versions and leftovers, sorted by risk (🟢 safe · 🟡 review · 🔴 personal). Nothing is deleted without your confirmation, and your files go to the trash first.
- 💤 **Let tabs nap** — keep all your tabs, even pinned ones, while unused ones sleep and stop eating RAM. Breathly learns which tabs you use every day.
- 🌐 **One browser at a time** — get a gentle heads-up when a second browser opens.

Built with **Rust** and **Tauri** so the tool itself stays light.

## Status

🚧 **Early design phase.** Nothing to install yet. Read the product requirements (in French): [`docs/PRD.md`](docs/PRD.md).

| Version | Scope |
|---|---|
| v0.1 | Diagnosis + safe disk cleanup (Windows, Linux) |
| v0.2 | Background service, tray icon, multiple-browser alert |
| v0.3 | Tab sleeping extension (Firefox, Zen, Chrome, Brave, Edge) |
| v0.4 | Startup apps, admin-level cleanups |
| v1.0 | signed installers (Windows, Linux), extensions on stores |

## Contributing

Contributions are welcome — cleanup rules, browser definitions and translations will be plain TOML files, no Rust required. A contribution guide will come with v0.1.

## License

[MIT](LICENSE)
