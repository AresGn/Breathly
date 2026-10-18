# Breathly

> *Let your PC breathe.*

Breathly is a free, open-source, lightweight tool that helps anyone — technical or not — understand **why their PC is slow** and fix it safely, on Windows and Linux (macOS after v1.0).

- 🩺 **Diagnose** — RAM, swap, CPU, disk, heat, and what's eating your resources, explained in plain words. A low-disk banner appears under 2 GB free.
- 🧹 **Free up space safely** — caches, old versions and leftovers, sorted by risk (🟢 safe · 🟡 review · 🔴 personal). Nothing is deleted without your confirmation, and your files go to the trash first.
- 💤 **Let tabs nap** — keep all your tabs, even pinned ones, while unused ones sleep and stop eating RAM. Breathly learns which tabs you use every day.
- 🌐 **One browser at a time** — get a gentle heads-up when a second browser opens.
- 🔎 **Forgotten apps** — heavy apps you have not used for a while are flagged; you decide whether to close them. Breathly is not an antivirus: unknown processes are flagged "to examine", never declared safe or infected.

Made for everyone whose PC is slow or full: developers, creatives (designers, video editors), office workers and families. Simple mode by default, details on demand.

Built with **Rust** and **Tauri** so the tool itself stays light.

## Status

🚧 **Early design phase.** Nothing to install yet. Read the product requirements (in French): [`docs/PRD.md`](docs/PRD.md).

Until v1.0 Breathly never connects to the network: download new versions from the [GitHub Releases](https://github.com/AresGn/Breathly/releases) page.

| Version | Scope |
|---|---|
| v0.1 | Diagnosis (disk, RAM, CPU, heat) + safe disk cleanup + low-disk banner (Windows, Linux) |
| v0.2 | Background service, tray icon, multiple-browser alert, forgotten-app and low-disk notifications |
| v0.3 | Tab sleeping extension (Firefox, Zen, Chrome, Brave, Edge) |
| v0.4 | Startup apps, admin-level cleanups (one-off elevation), system tuning such as compressed swap |
| v1.0 | signed installers (Windows, Linux), extensions on stores |

## Contributing

Contributions are welcome — cleanup rules, browser definitions and translations will be plain TOML files, no Rust required. A contribution guide will come with v0.1.

## License

[MIT](LICENSE)
