# Pile TypeScript (pnpm) : outils exécutés dans le dossier de l'application (voir tools/stack.sh).
# Scripts attendus dans package.json : lint, typecheck, test (et build en CI pour une application web).
STACK_MARKER="package.json"
stack_setup()  { pnpm install --frozen-lockfile; }
stack_lint()   { pnpm lint; }
stack_static() { pnpm typecheck; }
stack_test()   { pnpm test; }
stack_audit()  { pnpm audit --audit-level=high; }
