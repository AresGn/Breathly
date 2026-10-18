# Pile rust : outils exécutés dans le dossier de l'application (voir tools/stack.sh).
# Éprouvé par sondes négatives le 2026-10-18 (docs/research/2026-10-18-outils-etape-4.md) avec cargo-audit 0.22.2
# et cargo-deny 0.20.2 : `cargo install cargo-audit --version 0.22.2 --locked` et idem pour cargo-deny 0.20.2.
STACK_MARKER="Cargo.toml"
stack_setup()  { cargo fetch --locked; }
stack_lint()   { cargo fmt --all -- --check; }
stack_fix()    { cargo fmt --all; }
stack_static() { cargo clippy --workspace --all-targets --all-features --locked -- -D warnings; }
# Architecture : bibliothèques réseau interdites (SEC1), sens des dépendances entre crates (PRD §5), puis gardes
# que le compilateur ne contrôle pas (tools/check-rust.py : lints des crates, #[ignore], cfg selon l'OS).
stack_arch() {
  command -v cargo-deny >/dev/null 2>&1 || { echo "rust : cargo-deny absent (cargo install cargo-deny --version 0.20.2 --locked)"; return 1; }
  cargo deny --locked check bans sources
  python3 -I "$(git rev-parse --show-toplevel)/tools/check-rust.py"
}
stack_test()   { cargo test --workspace --all-features --locked; }
stack_audit()  { if command -v cargo-audit >/dev/null 2>&1; then cargo audit; else echo "rust : cargo-audit absent (cargo install cargo-audit --version 0.22.2 --locked)"; return 1; fi; }
