# Pile rust : outils exécutés dans le dossier de l'application (voir tools/stack.sh).
# Pack de départ non éprouvé : vérifier chaque commande dans la documentation de l'outil.
STACK_MARKER="Cargo.toml"
stack_setup()  { cargo fetch --locked; }
stack_lint()   { cargo fmt --all -- --check; }
stack_fix()    { cargo fmt --all; }
stack_static() { cargo clippy --all-targets --all-features --locked -- -D warnings; }
stack_arch()   { echo "rust : frontières de couches garanties par les crates du workspace et la visibilité (pub(crate)) ; cargo-deny pour les dépendances"; }
stack_test()   { cargo test --all-features --locked; }
stack_audit()  { if command -v cargo-audit >/dev/null 2>&1; then cargo audit; else echo "rust : cargo-audit absent (cargo install cargo-audit --locked)"; return 1; fi; }
