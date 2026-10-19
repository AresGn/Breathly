//! Essai jetable pour Windows (tickets #49 et #69). Ne fait partie ni du produit ni de l'espace de travail.
//!
//! Il travaille seulement dans un dossier temporaire (%TEMP%\breathly-essai-<n>) qu'il supprime à la fin, et écrit un
//! rapport (`rapport-essai-windows.txt`, dans le dossier courant) dont les chemins et le nom d'utilisateur sont masqués.
//! Option `--corbeille` : ajoute des essais avec la vraie corbeille de Windows (des fichiers d'essai y passent, puis
//! sont supprimés de la corbeille ; rien d'autre n'y est touché).

#[cfg(not(windows))]
fn main() {
    println!("Cet essai est prévu pour Windows.");
}

#[cfg(windows)]
fn main() {
    essai::run();
}

#[cfg(windows)]
mod essai {
    use std::ffi::{c_void, OsString};
    use std::fs::{self, OpenOptions};
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use std::os::windows::io::AsRawHandle;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    const DELETE: u32 = 0x0001_0000;
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const SHARE_READ_WRITE: u32 = 1 | 2;
    const SHARE_ALL: u32 = 1 | 2 | 4;

    unsafe extern "system" {
        fn GetFinalPathNameByHandleW(h: *mut c_void, buf: *mut u16, len: u32, flags: u32) -> u32;
        fn SetFileInformationByHandle(h: *mut c_void, class: i32, info: *const c_void, size: u32) -> i32;
    }

    struct Report { lines: Vec<String>, root: String, user: String }

    impl Report {
        fn push(&mut self, s: impl Into<String>) {
            let mut s: String = s.into();
            if !self.root.is_empty() { s = s.replace(&self.root, "<TEMP>"); }
            if !self.user.is_empty() { s = s.replace(&self.user, "<utilisateur>"); }
            println!("{s}");
            self.lines.push(s);
        }
    }

    fn err(e: &std::io::Error) -> String {
        format!("ERREUR {} (code {})", e.kind(), e.raw_os_error().map_or("?".into(), |c| c.to_string()))
    }

    fn lower(p: &Path) -> String { p.to_string_lossy().to_lowercase() }

    fn inside(candidate: &Path, root: &Path) -> bool {
        let (c, r) = (lower(candidate), lower(root));
        let r = r.trim_end_matches('\\');
        c == r || c.starts_with(&format!("{r}\\"))
    }

    fn is_reparse(p: &Path) -> bool {
        fs::symlink_metadata(p).map(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0).unwrap_or(false)
    }

    /// Supprime un lien (jonction ou lien symbolique de dossier) sans toucher à sa cible, ou un dossier réel.
    fn rm_any(p: &Path) {
        match fs::symlink_metadata(p) {
            Ok(m) if m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 => {
                let _ = fs::remove_dir(p).or_else(|_| fs::remove_file(p));
            }
            Ok(m) if m.is_dir() => { let _ = fs::remove_dir_all(p); }
            Ok(_) => { let _ = fs::remove_file(p); }
            Err(_) => {}
        }
    }

    fn mklink_j(link: &Path, target: &Path) -> bool {
        Command::new("cmd").args(["/C", "mklink", "/J"]).arg(link).arg(target)
            .output().map(|o| o.status.success()).unwrap_or(false)
    }

    fn final_path(h: *mut c_void) -> Option<PathBuf> {
        unsafe {
            let n = GetFinalPathNameByHandleW(h, std::ptr::null_mut(), 0, 0);
            if n == 0 { return None; }
            let mut buf = vec![0u16; n as usize + 1];
            let m = GetFinalPathNameByHandleW(h, buf.as_mut_ptr(), buf.len() as u32, 0);
            if m == 0 || m as usize > buf.len() { return None; }
            Some(PathBuf::from(OsString::from_wide(&buf[..m as usize])))
        }
    }

    // ------------------------------------------------------------------------------------ A : chemins (ticket #49)

    fn section_chemins(r: &mut Report, t: &Path) {
        r.push("== A. Chemins (ticket #49) ==");
        // A1 chemin long
        let mut long = t.join("long");
        let mut depth = 0;
        while long.to_string_lossy().len() < 330 { long.push("d".repeat(40)); depth += 1; }
        let ok = fs::create_dir_all(&long).and_then(|_| fs::write(long.join("f.txt"), "x"));
        r.push(format!("A1 chemin de {} caractères ({depth} niveaux), création : {}", long.to_string_lossy().len(), ok.as_ref().map_or_else(err, |_| "OK".into())));
        r.push(format!("A1 lecture des métadonnées : {}", fs::metadata(long.join("f.txt")).map_or_else(|e| err(&e), |m| format!("OK ({} octet)", m.len()))));
        r.push(format!("A1 parcours (read_dir) : {}", fs::read_dir(&long).map_or_else(|e| err(&e), |d| format!("OK ({} entrée)", d.count()))));
        r.push(format!("A1 suppression récursive : {}", fs::remove_dir_all(t.join("long")).map_or_else(|e| err(&e), |_| "OK".into())));
        // A2 canonicalize, préfixe \\?\
        let allowed = t.join("allowed");
        let _ = fs::create_dir_all(allowed.join("Sous"));
        let canon = fs::canonicalize(&allowed);
        r.push(format!("A2 canonicalize(<TEMP>\\allowed) = {:?}", canon.as_ref().map(|p| p.to_string_lossy().replace(&r.root, "<TEMP>")).map_err(|e| err(e))));
        if let Ok(c) = &canon {
            r.push(format!("A2 le résultat commence par \\\\?\\ : {}", c.to_string_lossy().starts_with(r"\\?\")));
            r.push(format!("A2 Path::starts_with(chemin brut, racine canonique) = {} ; (racine canonique, chemin brut) = {}", allowed.starts_with(c), c.starts_with(&allowed)));
            r.push(format!("A2 comparaison par chemins canoniques des deux côtés : {}", inside(&fs::canonicalize(allowed.join("Sous")).unwrap_or_default(), c)));
        }
        // A3 casse
        let swapped = PathBuf::from(allowed.to_string_lossy().to_uppercase());
        r.push(format!("A3 casse : Path::starts_with(MAJUSCULES, minuscules) = {} ; metadata(MAJUSCULES) = {} ; canonicalize(MAJUSCULES) identique à canonicalize(original) = {}",
            swapped.starts_with(&allowed), fs::metadata(&swapped).map_or_else(|e| err(&e), |_| "OK".into()),
            fs::canonicalize(&swapped).ok() == fs::canonicalize(&allowed).ok()));
        // A4 séparateurs et ..
        let dotted = format!("{}/./Sous/../Sous", allowed.to_string_lossy());
        r.push(format!("A4 « / », « . » et « .. » : canonicalize = {} ; metadata = {}",
            fs::canonicalize(&dotted).map_or_else(|e| err(&e), |p| format!("{}", inside(&p, canon.as_deref().unwrap_or(&allowed)))),
            fs::metadata(&dotted).map_or_else(|e| err(&e), |_| "OK".into())));
        // A5 noms réservés et points / espaces finaux
        for name in ["CON", "NUL", "aux.txt", "COM1"] {
            r.push(format!("A5 création du fichier « {name} » : {}", fs::write(allowed.join(name), "x").map_or_else(|e| err(&e), |_| "OK (créé)".into())));
        }
        let verb = format!(r"\\?\{}\fin.", canon.as_deref().map(|p| p.to_string_lossy().trim_start_matches(r"\\?\").to_string()).unwrap_or_default());
        r.push(format!("A5 nom finissant par un point, en chemin verbatim : création {} ; existe sans le point : {}",
            fs::write(&verb, "x").map_or_else(|e| err(&e), |_| "OK".into()), allowed.join("fin").exists()));
        // A6 noms courts 8.3
        let long_dir = allowed.join("Dossier avec un long nom");
        let _ = fs::create_dir_all(&long_dir);
        let short = Command::new("cmd").args(["/C", "for", "%I", "in"]).arg(format!("(\"{}\")", long_dir.display())).arg("do").arg("@echo").arg("%~sI").output().ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
        r.push(format!("A6 nom court 8.3 relevé : {} ; canonicalize du nom court donne le nom long : {}",
            if short.is_empty() { "indisponible".into() } else { short.replace(&r.root, "<TEMP>") },
            !short.is_empty() && fs::canonicalize(&short).ok() == fs::canonicalize(&long_dir).ok()));
        // A7 jonction
        let real = t.join("cible_jonction");
        let _ = fs::create_dir_all(&real);
        let _ = fs::write(real.join("proteg.txt"), "protégé");
        let jonc = t.join("jonction");
        if mklink_j(&jonc, &real) {
            let md = fs::symlink_metadata(&jonc);
            r.push(format!("A7 jonction créée par « mklink /J » (sans droits administrateur) : symlink_metadata : is_symlink={:?} is_dir={:?} attribut REPARSE_POINT={:?}",
                md.as_ref().map(|m| m.file_type().is_symlink()).ok(), md.as_ref().map(|m| m.is_dir()).ok(),
                md.as_ref().map(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0).ok()));
            r.push(format!("A7 metadata (qui suit le lien) is_dir : {:?} ; read_link : {}", fs::metadata(&jonc).map(|m| m.is_dir()).ok(),
                fs::read_link(&jonc).map_or_else(|e| err(&e), |p| p.to_string_lossy().replace(&r.root, "<TEMP>"))));
            r.push(format!("A7 canonicalize(jonction\\proteg.txt) sort de la jonction vers la cible : {}",
                fs::canonicalize(jonc.join("proteg.txt")).map(|p| inside(&p, &fs::canonicalize(&real).unwrap_or_default())).unwrap_or(false)));
            r.push(format!("A7 la jonction est vue comme « hors de allowed » après canonicalize : {}",
                fs::canonicalize(&jonc).map(|p| !inside(&p, canon.as_deref().unwrap_or(&allowed))).unwrap_or(false)));
            let _ = fs::remove_dir(&jonc);
            r.push(format!("A7 remove_dir(jonction) : la cible existe toujours : {}", real.join("proteg.txt").exists()));
        } else {
            r.push("A7 création de jonction : ÉCHEC (mklink /J)");
        }
        // A8 lien symbolique de dossier
        let sl = t.join("lien_dossier");
        match std::os::windows::fs::symlink_dir(&real, &sl) {
            Ok(()) => {
                let md = fs::symlink_metadata(&sl);
                r.push(format!("A8 lien symbolique de dossier : créé ; is_symlink={:?} REPARSE_POINT={:?}", md.as_ref().map(|m| m.file_type().is_symlink()).ok(),
                    md.as_ref().map(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0).ok()));
                rm_any(&sl);
            }
            Err(e) => r.push(format!("A8 lien symbolique de dossier : {} (le mode développeur ou les droits administrateur manquent probablement)", err(&e))),
        }
    }

    // ------------------------------------------------------------------------------------ B : suppression (ticket #69)

    #[derive(Clone, Copy, PartialEq)]
    enum Out { Done, Refused }

    /// Chaque variante appelle `hook` UNE fois, entre sa vérification et son action : c'est là que le banc lance le fil
    /// qui échange le dossier et la jonction, puis attend `delai` microsecondes. La fenêtre est la même pour toutes.
    type Hook<'a> = &'a dyn Fn();

    fn naive(path: &Path, allowed: &Path, hook: Hook) -> Out {
        let Ok(c) = fs::canonicalize(path) else { return Out::Refused };
        if !inside(&c, &fs::canonicalize(allowed).unwrap_or_default()) { return Out::Refused; }
        hook();
        if fs::remove_file(path).is_ok() { Out::Done } else { Out::Refused }
    }

    fn trash_naif(path: &Path, allowed: &Path, hook: Hook) -> Out {
        let Ok(c) = fs::canonicalize(path) else { return Out::Refused };
        if !inside(&c, &fs::canonicalize(allowed).unwrap_or_default()) { return Out::Refused; }
        hook();
        if trash::delete(path).is_ok() { Out::Done } else { Out::Refused }
    }

    /// On ouvre l'objet lui-même (sans suivre un point d'analyse final), on lit son chemin réel PAR LE HANDLE, on le
    /// compare à la racine autorisée, puis on supprime PAR LE MÊME HANDLE : il n'y a plus d'écart entre vérification et action.
    fn par_handle(path: &Path, allowed: &Path, hook: Hook) -> Out {
        let Ok(f) = OpenOptions::new().access_mode(DELETE | FILE_READ_ATTRIBUTES).share_mode(SHARE_ALL)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS).open(path) else { return Out::Refused };
        let h = f.as_raw_handle();
        let Some(real) = final_path(h) else { return Out::Refused };
        if !inside(&real, &fs::canonicalize(allowed).unwrap_or_default()) { return Out::Refused; }
        if f.metadata().map(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0).unwrap_or(true) { return Out::Refused; }
        hook();
        let info: u8 = 1; // FILE_DISPOSITION_INFO { DeleteFile: TRUE }
        let ok = unsafe { SetFileInformationByHandle(h, 4, &info as *const u8 as *const c_void, 1) } != 0;
        drop(f);
        if ok { Out::Done } else { Out::Refused }
    }

    /// Ouvre, sans partage de suppression, la racine autorisée et chaque dossier du chemin (le dernier composant aussi si
    /// `avec_final`) ; refuse si l'un d'eux est un point d'analyse. Tant que ces handles vivent, un autre processus ne peut
    /// ni les renommer ni les supprimer, donc ni les remplacer par une jonction.
    fn verrouiller(path: &Path, allowed: &Path, avec_final: bool) -> Option<Vec<fs::File>> {
        let rel = path.strip_prefix(allowed).ok()?;
        let mut cur = allowed.to_path_buf();
        let mut chain = vec![allowed.to_path_buf()];
        let comps: Vec<_> = rel.components().collect();
        let n = if avec_final { comps.len() } else { comps.len().saturating_sub(1) };
        for c in comps.iter().take(n) { cur.push(c); chain.push(cur.clone()); }
        let mut handles = vec![];
        for d in &chain {
            let h = OpenOptions::new().read(true).share_mode(SHARE_READ_WRITE)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS).open(d).ok()?;
            if h.metadata().map(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0).unwrap_or(true) { return None; }
            handles.push(h);
        }
        Some(handles)
    }

    fn chaine_verrouillee(path: &Path, allowed: &Path, hook: Hook) -> Out {
        let Some(handles) = verrouiller(path, allowed, false) else { return Out::Refused };
        hook();
        let out = if fs::remove_file(path).is_ok() { Out::Done } else { Out::Refused };
        drop(handles);
        out
    }

    /// Corbeille : mêmes dossiers verrouillés, puis `trash::delete(chemin)`.
    fn trash_chaine(path: &Path, allowed: &Path, hook: Hook) -> Out {
        let Some(handles) = verrouiller(path, allowed, false) else { return Out::Refused };
        hook();
        let out = if trash::delete(path).is_ok() { Out::Done } else { Out::Refused };
        drop(handles);
        out
    }

    /// Dossiers (suppression définitive) : naïf = vérifier puis `remove_dir_all(chemin)`.
    fn rd_naif(path: &Path, allowed: &Path, hook: Hook) -> Out {
        let Ok(c) = fs::canonicalize(path) else { return Out::Refused };
        if !inside(&c, &fs::canonicalize(allowed).unwrap_or_default()) { return Out::Refused; }
        hook();
        if fs::remove_dir_all(path).is_ok() { Out::Done } else { Out::Refused }
    }

    /// Dossiers : seuls les parents sont verrouillés (le dernier composant reste échangeable).
    fn rd_chaine_parents(path: &Path, allowed: &Path, hook: Hook) -> Out {
        let Some(handles) = verrouiller(path, allowed, false) else { return Out::Refused };
        hook();
        let out = if fs::remove_dir_all(path).is_ok() { Out::Done } else { Out::Refused };
        drop(handles);
        out
    }

    /// Dossiers : parents ET dossier final verrouillés ; le contenu est supprimé sous le verrou, puis on lâche le handle
    /// du dossier final et on retire le dossier (vide, ou une jonction : `remove_dir` ne détruit jamais la cible d'une jonction).
    fn rd_chaine_complete(path: &Path, allowed: &Path, hook: Hook) -> Out {
        let Some(mut handles) = verrouiller(path, allowed, true) else { return Out::Refused };
        hook();
        let mut ok = true;
        if let Ok(rd) = fs::read_dir(path) {
            for e in rd.flatten() {
                let p = e.path();
                let reparse = is_reparse(&p);
                ok &= if reparse { fs::remove_dir(&p).or_else(|_| fs::remove_file(&p)).is_ok() }
                      else if p.is_dir() { fs::remove_dir_all(&p).is_ok() } else { fs::remove_file(&p).is_ok() };
            }
        } else { ok = false; }
        let dernier = handles.pop();
        drop(dernier);
        ok &= fs::remove_dir(path).is_ok();
        drop(handles);
        if ok { Out::Done } else { Out::Refused }
    }

    struct Course { done: usize, refused: usize, lost: usize, vue: usize, ms: u128 }

    #[derive(Clone, Copy)]
    enum Kind { Fichier, DossierParent, DossierFinal }

    /// `vue` : nombre d'essais où la jonction a été observée en place au moins une fois pendant la fenêtre.
    /// Fichier : `slot\<nom>` ; DossierParent : `slot\cachedir` (on échange `slot`) ; DossierFinal : `cachedir` (on l'échange lui-même).
    fn course(t: &Path, tag: &str, trials: usize, delai_us: u64, nom: &str, kind: Kind, variant: fn(&Path, &Path, Hook) -> Out) -> Course {
        use std::cell::{Cell, RefCell};
        let allowed = t.join(format!("allowed_{tag}"));
        let protected = t.join(format!("protected_{tag}"));
        let _ = fs::create_dir_all(&allowed);
        let _ = fs::create_dir_all(&protected);
        let slot = allowed.join("slot");
        let (a, b, c, target, junction_to, sentinel) = match kind {
            Kind::Fichier => (slot.clone(), allowed.join("slot_alt"), allowed.join("slot_tmp"), slot.join(nom), protected.clone(), protected.join(nom)),
            Kind::DossierParent => (slot.clone(), allowed.join("slot_alt"), allowed.join("slot_tmp"), slot.join("cachedir"), protected.clone(), protected.join("cachedir").join("keep.bin")),
            Kind::DossierFinal => (allowed.join("cachedir"), allowed.join("cachedir_alt"), allowed.join("cachedir_tmp"), allowed.join("cachedir"), protected.join("cachedir"), protected.join("cachedir").join("keep.bin")),
        };
        let (mut done, mut refused, mut lost) = (0, 0, 0);
        let vue = Cell::new(0usize);
        let start = std::time::Instant::now();
        for i in 0..trials {
            for p in [&slot, &a, &b, &c, &allowed.join("slot_alt"), &allowed.join("slot_tmp")] { rm_any(p); }
            let _ = fs::create_dir_all(&protected);
            match kind {
                Kind::Fichier => {
                    let _ = fs::create_dir_all(&slot);
                    let _ = fs::write(&target, format!("cache {i}"));
                    let _ = fs::write(&sentinel, format!("protégé {i}"));
                }
                Kind::DossierParent | Kind::DossierFinal => {
                    let _ = fs::create_dir_all(target.join("sous"));
                    for k in 0..5 { let _ = fs::write(target.join(format!("f{k}.bin")), "c"); let _ = fs::write(target.join("sous").join(format!("g{k}.bin")), "c"); }
                    let _ = fs::create_dir_all(protected.join("cachedir"));
                    let _ = fs::write(&sentinel, "protégé");
                }
            }
            if !mklink_j(&b, &junction_to) { continue; }
            let stop = Arc::new(AtomicBool::new(false));
            let swapper: RefCell<Option<std::thread::JoinHandle<()>>> = RefCell::new(None);
            let hook = || {
                let (s2, a2, b2, c2) = (stop.clone(), a.clone(), b.clone(), c.clone());
                *swapper.borrow_mut() = Some(std::thread::spawn(move || {
                    while !s2.load(Ordering::Relaxed) {
                        if fs::rename(&a2, &c2).is_ok() {
                            let _ = fs::rename(&b2, &a2);
                            let _ = fs::rename(&c2, &b2);
                        }
                    }
                }));
                // Pendant toute la fenêtre, on échantillonne l'état du composant échangé : « vue » compte les essais où l'on
                // a observé au moins une fois la jonction à sa place (le fil perturbateur a donc bien agi dans la fenêtre).
                let end = std::time::Instant::now() + std::time::Duration::from_micros(delai_us);
                let mut vu = false;
                while std::time::Instant::now() < end { if !vu && is_reparse(&a) { vu = true; } }
                if vu { vue.set(vue.get() + 1); }
            };
            let out = variant(&target, &allowed, &hook);
            stop.store(true, Ordering::Relaxed);
            if let Some(h) = swapper.borrow_mut().take() { let _ = h.join(); }
            if out == Out::Done { done += 1 } else { refused += 1 }
            if !sentinel.exists() { lost += 1; }
        }
        Course { done, refused, lost, vue: vue.get(), ms: start.elapsed().as_millis() }
    }

    fn section_suppression(r: &mut Report, t: &Path, trials: usize, delai_us: u64, corbeille: bool) {
        r.push("== B. Suppression sûre (ticket #69) ==");
        let allowed = t.join("allowed_b");
        let protected = t.join("protected_b");
        let _ = fs::create_dir_all(&allowed);
        let _ = fs::create_dir_all(protected.join("cachedir"));
        let _ = fs::write(protected.join("cachedir").join("garde.bin"), "protégé");
        let lien = allowed.join("lien");
        if !mklink_j(&lien, &protected) { r.push("B1 jonction impossible : section ignorée"); return; }
        // B1 : suppression d'un dossier qui CONTIENT une jonction
        let contenant = allowed.join("contenant");
        let _ = fs::create_dir_all(&contenant);
        let _ = mklink_j(&contenant.join("jonction_interne"), &protected);
        let res = fs::remove_dir_all(&contenant);
        r.push(format!("B1 remove_dir_all d'un dossier qui contient une jonction : {} ; la cible de la jonction est intacte : {}",
            res.map_or_else(|e| err(&e), |_| "OK".into()), protected.join("cachedir").join("garde.bin").exists()));
        // B2 : suppression PAR UN CHEMIN qui traverse une jonction
        let via = lien.join("cachedir");
        let res = fs::remove_dir_all(&via);
        r.push(format!("B2 remove_dir_all(chemin qui traverse une jonction) : {} ; le fichier protégé a été supprimé : {}",
            res.map_or_else(|e| err(&e), |_| "OK".into()), !protected.join("cachedir").join("garde.bin").exists()));
        // B3 : détection du point d'analyse composant par composant
        let _ = fs::create_dir_all(protected.join("cachedir"));
        r.push(format!("B3 le composant « lien » est un point d'analyse (attribut REPARSE_POINT) : {} ; un dossier normal ne l'est pas : {}",
            is_reparse(&lien), !is_reparse(&allowed)));
        // B4 : verrouillage de la chaîne : peut-on renommer un dossier tenu ouvert sans partage de suppression ?
        let slot = allowed.join("slot");
        let _ = fs::create_dir_all(&slot);
        let verrou = OpenOptions::new().read(true).share_mode(SHARE_READ_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS).open(&slot);
        let renomme = fs::rename(&slot, allowed.join("slot_renomme"));
        r.push(format!("B4 dossier tenu ouvert sans partage de suppression : handle {} ; renommage : {}", if verrou.is_ok() { "obtenu" } else { "REFUSÉ" },
            renomme.as_ref().map_or_else(err, |_| "AUTORISÉ (le verrou ne protège pas)".into())));
        drop(verrou);
        let _ = fs::rename(allowed.join("slot_renomme"), &slot);
        let libre = fs::rename(&slot, allowed.join("slot_renomme2"));
        r.push(format!("B4 même renommage sans handle ouvert : {}", libre.as_ref().map_or_else(err, |_| "AUTORISÉ (attendu)".into())));
        // B5 : course (sans échange atomique sous Windows : trois renommages successifs, lancés entre vérification et action)
        r.push(format!("B5 course, {trials} essais par variante : entre sa vérification et son action, chaque variante laisse un fil alterner « slot » (dossier réel) et « slot_alt » (jonction vers un dossier protégé) pendant {delai_us} µs. « vue » = essais où la jonction a été observée en place au moins une fois pendant la fenêtre. Fichiers protégés perdus :"));
        let variantes: [(&str, fn(&Path, &Path, Hook) -> Out); 3] = [("naïf (canonicalize puis remove_file)", naive), ("par handle (vérification et suppression par le même handle)", par_handle), ("chaîne verrouillée (dossiers tenus ouverts)", chaine_verrouillee)];
        for (name, f) in variantes {
            let c = course(t, "c", trials, delai_us, "target.bin", Kind::Fichier, f);
            r.push(format!("   - {name} : {} perdus sur {trials} (jonction vue {}, faits {}, refus {}), {} ms", c.lost, c.vue, c.done, c.refused, c.ms));
        }
        // B6 : dossiers (suppression définitive d'un dossier de cache)
        let essais_d = trials.min(200);
        let dossiers: [(&str, fn(&Path, &Path, Hook) -> Out); 3] = [("naïf (remove_dir_all après vérification)", rd_naif), ("parents verrouillés seulement", rd_chaine_parents), ("parents et dossier final verrouillés", rd_chaine_complete)];
        for (titre, kind, tag) in [("B6a on échange un dossier PARENT (slot) du chemin", Kind::DossierParent, "dp"), ("B6b on échange le DERNIER composant (cachedir) lui-même", Kind::DossierFinal, "df")] {
            r.push(format!("{titre}, {essais_d} essais par variante, fenêtre de {delai_us} µs :"));
            for (name, f) in dossiers {
                let c = course(t, tag, essais_d, delai_us, "cachedir", kind, f);
                r.push(format!("   - {name} : {} perdus sur {essais_d} (jonction vue {}, faits {}, refus {}), {} ms", c.lost, c.vue, c.done, c.refused, c.ms));
            }
        }
        if corbeille { section_corbeille(r, t, trials.min(200), delai_us); }
    }

    fn section_corbeille(r: &mut Report, t: &Path, trials: usize, delai_us: u64) {
        r.push("== C. Vraie corbeille de Windows (option --corbeille) ==");
        let allowed = t.join("allowed_t");
        let protected = t.join("protected_t");
        let _ = fs::create_dir_all(&allowed);
        let _ = fs::create_dir_all(&protected);
        let lien = allowed.join("lien");
        let _ = fs::write(protected.join("breathly-essai-protege.bin"), "protégé");
        if mklink_j(&lien, &protected) {
            let res = trash::delete(lien.join("breathly-essai-protege.bin"));
            r.push(format!("C1 trash::delete(chemin qui traverse une jonction) : {} ; le fichier protégé a quitté son dossier : {}",
                res.map_or_else(|e| format!("ERREUR {e}"), |_| "OK".into()), !protected.join("breathly-essai-protege.bin").exists()));
        }
        rm_any(&lien);
        let c = course(t, "t", trials, delai_us, "breathly-essai-cible.bin", Kind::Fichier, trash_naif);
        r.push(format!("C2 course avec trash::delete naïf, {delai_us} µs entre vérification et action : {} fichiers protégés perdus sur {trials} essais (jonction vue {}, suppressions faites {}, refus {}), {} ms",
            c.lost, c.vue, c.done, c.refused, c.ms));
        let c = course(t, "t2", trials, delai_us, "breathly-essai-cible.bin", Kind::Fichier, trash_chaine);
        r.push(format!("C2b course avec dossiers verrouillés puis trash::delete, {delai_us} µs : {} fichiers protégés perdus sur {trials} essais (jonction vue {}, suppressions faites {}, refus {}), {} ms",
            c.lost, c.vue, c.done, c.refused, c.ms));
        // nettoyage de la corbeille : seulement les éléments d'essai
        match trash::os_limited::list() {
            Ok(items) => {
                let a_purger: Vec<_> = items.into_iter()
                    .filter(|i| i.name.to_string_lossy().starts_with("breathly-essai-"))
                    .collect();
                let n = a_purger.len();
                r.push(format!("C3 nettoyage : {n} éléments d'essai retirés de la corbeille : {}", trash::os_limited::purge_all(a_purger).map_or_else(|e| format!("ERREUR {e}"), |_| "OK".into())));
            }
            Err(e) => r.push(format!("C3 liste de la corbeille impossible : {e}")),
        }
    }

    pub fn run() {
        let args: Vec<String> = std::env::args().collect();
        let corbeille = args.iter().any(|a| a == "--corbeille");
        let trials = args.iter().position(|a| a == "--essais").and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(300usize);
        let delai_us = args.iter().position(|a| a == "--delai-us").and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(10_000u64);
        let t = std::env::temp_dir().join(format!("breathly-essai-{}", std::process::id()));
        let _ = fs::remove_dir_all(&t);
        fs::create_dir_all(&t).expect("création du dossier temporaire");
        let mut r = Report { lines: vec![], root: t.to_string_lossy().to_string(), user: std::env::var("USERNAME").unwrap_or_default() };
        r.push(format!("Essai Windows, {}", std::env::consts::ARCH));
        r.push(format!("Windows : {}", Command::new("cmd").args(["/C", "ver"]).output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()));
        r.push(format!("Dossier de travail : <TEMP> ({} essais par variante de course, délai de {} µs entre vérification et action ; option --corbeille : {})", trials, delai_us, corbeille));
        section_chemins(&mut r, &t);
        section_suppression(&mut r, &t, trials, delai_us, corbeille);
        let _ = fs::remove_dir_all(&t);
        r.push(format!("Dossier temporaire supprimé : {}", !t.exists()));
        let _ = fs::write("rapport-essai-windows.txt", r.lines.join("\n") + "\n");
        println!("\nRapport écrit dans rapport-essai-windows.txt : collez-le à Claude.");
    }
}
