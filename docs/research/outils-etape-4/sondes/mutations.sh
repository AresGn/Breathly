inject() { f=crates/$1/src/lib.rs; { head -n 1 "$f"; cat; tail -n +2 "$f"; } > "$f.new"; mv "$f.new" "$f"; }
mutate_clean() { :; }
mutate_fmt() { printf 'pub fn   ugly( a:u32 )->u32{a}\n' >> crates/rules/src/lib.rs; }
mutate_pedantic() { inject rules <<'X'
pub fn no_must_use(a: u32) -> u32 { a + 1 }
X
}
mutate_all() { inject rules <<'X'
/// x
#[must_use]
pub fn needless(a: u32) -> u32 { return a + 1; }
X
}
mutate_unwrap_prod() { inject rules <<'X'
/// x
///
/// # Panics
/// Si `s` n'est pas un nombre.
#[must_use]
pub fn parse(s: &str) -> u32 { s.parse().unwrap() }
X
}
mutate_unwrap_test() { sed -i 's/assert_eq!(double(2), 4);/let v: u32 = "4".parse().unwrap(); assert_eq!(double(2), v);/' crates/rules/src/lib.rs; }
mutate_expect_prod() { inject rules <<'X'
/// x
///
/// # Panics
/// Si `s` n'est pas un nombre.
#[must_use]
pub fn parse(s: &str) -> u32 { s.parse().expect("nombre") }
X
}
mutate_cognitive() { inject rules <<'X'
/// x
#[must_use]
#[allow(clippy::collapsible_if, clippy::comparison_chain)]
pub fn tangled(a: u32, b: u32) -> u32 {
    let mut n = 0;
    if a > 1 { if b > 1 { if a > b { n += 1; } else if b > a { n += 2; } else { n += 3; } } }
    for i in 0..a { if i % 2 == 0 { if b > i { n += 1; } else { n += 2; } } }
    while n > 100 { if a > 3 { n -= 1; } }
    if a > 5 && b > 5 || a < 2 && b < 2 { n += 7; }
    n
}
X
}
mutate_too_many_lines() { { echo '/// x'; echo '#[must_use]'; echo 'pub fn long() -> u32 {'; echo '    let mut n = 0;'; for i in $(seq 1 45); do echo "    n += $i;"; done; echo '    n'; echo '}'; } | inject rules; }
mutate_too_many_args() { inject rules <<'X'
/// x
#[must_use]
pub fn four(a: u32, b: u32, c: u32, d: u32) -> u32 { a + b + c + d }
X
}
UNSAFE_FN='/// x
#[must_use]
pub fn raw() -> u32 {
    let x = 5u32;
    let p: *const u32 = &raw const x;
    unsafe { p.read() }
}'
mutate_unsafe_rules() { echo "$UNSAFE_FN" | inject rules; }
mutate_unsafe_platform_bare() { echo "$UNSAFE_FN" | inject platform; }
mutate_unsafe_platform_allowed() { inject platform <<'X'
/// x
#[must_use]
pub fn raw() -> u32 {
    let x = 5u32;
    let p: *const u32 = &raw const x;
    // SAFETY: `p` vient d'une référence valide vers une variable locale vivante.
    #[allow(unsafe_code, reason = "lecture d'un pointeur local valide")]
    unsafe { p.read() }
}
X
}
mutate_unsafe_allow_in_rules() { inject rules <<'X'
/// x
#[must_use]
pub fn raw() -> u32 {
    let x = 5u32;
    let p: *const u32 = &raw const x;
    #[allow(unsafe_code)]
    unsafe { p.read() }
}
X
}
ENUM='/// x
#[derive(Clone, Copy)]
pub enum Risk { Green, Yellow, Red }
'
mutate_wildcard() { { echo "$ENUM"; cat <<'X'
/// x
#[must_use]
pub fn weight(r: Risk) -> u32 { match r { Risk::Green => 1, _ => 2 } }
X
} | inject rules; }
mutate_wildcard_ok() { { echo "$ENUM"; cat <<'X'
/// x
#[must_use]
pub fn weight(r: Risk) -> u32 { match r { Risk::Green => 1, Risk::Yellow => 2, Risk::Red => 3 } }
X
} | inject rules; }
mutate_ignored_test() { sed -i 's/#\[test\]/#[test]\n    #[ignore = "plus tard"]/' crates/rules/src/lib.rs; }
mutate_process_command() { inject rules <<'X'
/// x
/// # Errors
/// Si le programme ne se lance pas.
pub fn run() -> std::io::Result<std::process::Output> { std::process::Command::new("ls").output() }
X
}

mutate_allow_no_reason() { inject rules <<'X'
/// x
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn takes(s: String) -> usize { s.len() }
X
}
mutate_allow_with_reason() { inject rules <<'X'
/// x
#[must_use]
#[allow(clippy::needless_pass_by_value, reason = "API publique stable")]
pub fn takes(s: String) -> usize { s.len() }
X
}

mutate_process_in_platform() { inject platform <<'X'
/// x
///
/// # Errors
/// Si le programme ne se lance pas.
#[allow(clippy::disallowed_methods, clippy::disallowed_types, reason = "seul crate autorisé à lancer un programme")]
pub fn run() -> std::io::Result<std::process::Output> { std::process::Command::new("ls").output() }
X
}
mutate_unwrap_in_test_ok() { sed -i 's/assert_eq!(double(2), 4);/let v: u32 = "4".parse().unwrap(); assert_eq!(double(2), v);/' crates/rules/src/lib.rs; }
mutate_dep_platform_rules() { printf '\n[dependencies]\nbreathly-rules = { path = "../rules" }\n' >> crates/platform/Cargo.toml; sed -i 's|^//! Sonde.|//! Sonde.\nuse breathly_rules as _;|' crates/platform/src/lib.rs; }
mutate_dep_rules_platform() { printf '\n[dependencies]\nbreathly-platform = { path = "../platform" }\n' >> crates/rules/Cargo.toml; }
mutate_dep_diagnose_disk() { printf 'breathly-disk = { path = "../disk" }\n' >> crates/diagnose/Cargo.toml; python3 - <<'X'
p="crates/diagnose/Cargo.toml"; s=open(p).read()
# remettre la ligne dans [dependencies]
s=s.replace('breathly-disk = { path = "../disk" }\n',''); s=s.replace('[lints]','breathly-disk = { path = "../disk" }\n\n[lints]'); open(p,"w").write(s)
X
}
mutate_dep_dev_ureq() { printf '\n[dev-dependencies]\nureq = "=2.12.1"\n' >> crates/rules/Cargo.toml; }
mutate_dep_ureq() { printf '\n[dependencies]\nureq = "=2.12.1"\n' >> crates/rules/Cargo.toml; }
mutate_dep_vuln() { printf '\n[dependencies]\nsmallvec = "=1.6.0"\n' >> crates/rules/Cargo.toml; }
mutate_dep_git() { printf '\n[dependencies]\nfoo = { git = "https://example.invalid/foo" }\n' >> crates/rules/Cargo.toml; }
mutate_dep_wildcard() { printf '\n[dependencies]\nsmallvec = "*"\n' >> crates/rules/Cargo.toml; }
mutate_nesting() { inject rules <<'X'
/// x
#[must_use]
pub fn nested(a: u32) -> u32 {
    let mut n = 0;
    if a > 1 {
        if a > 2 {
            if a > 3 {
                if a > 4 {
                    n += 1;
                }
            }
        }
    }
    n
}
X
}
mutate_nesting_ok() { inject rules <<'X'
/// x
#[must_use]
pub fn three(a: u32) -> u32 {
    let mut n = 0;
    if a > 1 {
        n += 1;
        if a > 2 {
            n += 1;
            if a > 3 {
                n += 1;
            }
        }
    }
    n
}
X
}
