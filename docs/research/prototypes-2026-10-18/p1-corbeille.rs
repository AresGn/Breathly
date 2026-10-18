// PROTOTYPE JETABLE (étape 2.9, P1) : corbeille et suppression sûre. Ne pas réutiliser tel quel.
// Lancer avec XDG_DATA_HOME pointant vers un dossier temporaire pour ne jamais toucher à la vraie corbeille.
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use trash::os_limited;

fn ok(label: &str, cond: bool) {
    println!("{} {}", if cond { "OK  " } else { "ÉCHEC" }, label);
}

fn names() -> Vec<String> {
    let mut v: Vec<String> = os_limited::list().unwrap().iter().map(|i| i.name.to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

fn main() {
    let base = PathBuf::from(std::env::var("P1_WORK").expect("P1_WORK"));
    let trash_home = std::env::var("XDG_DATA_HOME").expect("XDG_DATA_HOME");
    println!("corbeille de test : {trash_home}/Trash ; dossier de travail : {}", base.display());
    ok("la corbeille de test est vide au départ", os_limited::is_empty().unwrap());

    // A. envoyer 3 fichiers, lister, lire les métadonnées
    let files: Vec<PathBuf> = ["a.txt", "b.txt", "c.txt"].iter().map(|n| base.join(n)).collect();
    for f in &files { fs::write(f, vec![b'x'; 1000]).unwrap(); }
    trash::delete_all(&files).unwrap();
    ok("A. 3 fichiers listés dans la corbeille", names() == ["a.txt", "b.txt", "c.txt"]);
    let items = os_limited::list().unwrap();
    let meta = os_limited::metadata(&items[0]).unwrap();
    println!("   métadonnées d'un élément : {:?}", meta.size);
    println!("   chemin d'origine : {:?}", items[0].original_path());

    // B. restaurer une sélection
    let a = items.iter().find(|i| i.name == "a.txt").unwrap().clone();
    os_limited::restore_all(vec![a]).unwrap();
    ok("B. a.txt restauré à son emplacement, b et c restent", base.join("a.txt").exists() && names() == ["b.txt", "c.txt"]);

    // C. restaurer quand l'emplacement d'origine est occupé
    fs::write(base.join("b.txt"), b"nouveau").unwrap();
    let b = os_limited::list().unwrap().into_iter().find(|i| i.name == "b.txt").unwrap();
    let r = os_limited::restore_all(vec![b]);
    println!("   C. restauration sur emplacement occupé : {:?}", r.as_ref().map(|_| "Ok").map_err(|e| format!("{e:?}")));
    ok("C. le fichier occupant n'est pas écrasé", fs::read(base.join("b.txt")).unwrap() == b"nouveau");
    println!("   C. contenu de la corbeille après : {:?}", names());

    // D. purge sélective, avec un élément étranger pré-existant
    let foreign = base.join("etranger.txt");
    fs::write(&foreign, b"autre").unwrap();
    trash::delete(&foreign).unwrap();
    let all = os_limited::list().unwrap();
    let mine: Vec<_> = all.iter().filter(|i| i.name == "c.txt").cloned().collect();
    os_limited::purge_all(mine).unwrap();
    let after = names();
    ok("D. purge sélective : c.txt détruit, l'élément étranger reste", !after.contains(&"c.txt".to_string()) && after.contains(&"etranger.txt".to_string()));

    // F. lien symbolique : on envoie le lien, pas la cible
    let cible_protegee = base.join("secret_hors_regle.txt");
    fs::write(&cible_protegee, b"ne pas toucher").unwrap();
    let link = base.join("lien_vers_secret");
    symlink(&cible_protegee, &link).unwrap();
    trash::delete(&link).unwrap();
    ok("F. supprimer un lien symbolique n'atteint pas sa cible", cible_protegee.exists() && !link.exists());
    let dir_target = base.join("dossier_interdit");
    fs::create_dir(&dir_target).unwrap();
    fs::write(dir_target.join("doc.txt"), b"important").unwrap();
    let dlink = base.join("lien_dossier");
    symlink(&dir_target, &dlink).unwrap();
    trash::delete(&dlink).unwrap();
    ok("F2. supprimer un lien vers un dossier n'atteint pas le dossier", dir_target.join("doc.txt").exists());

    // G. course entre vérification et action : un composant du chemin devient un lien symbolique
    let allowed = base.join("autorise");
    let sub = allowed.join("cache");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("data.bin"), b"cache").unwrap();
    let forbidden = base.join("interdit");
    fs::create_dir_all(&forbidden).unwrap();
    fs::write(forbidden.join("data.bin"), b"PROTEGE").unwrap();
    let path = sub.join("data.bin");
    let canon_before = fs::canonicalize(&path).unwrap();
    ok("G0. avant l'échange, le chemin est bien sous le dossier autorisé", canon_before.starts_with(fs::canonicalize(&allowed).unwrap()));
    // l'attaquant remplace "cache" par un lien vers "interdit" entre la vérification et l'action
    fs::remove_dir_all(&sub).unwrap();
    symlink(&forbidden, &sub).unwrap();
    // mitigation : recanoniser juste avant d'agir
    let canon_now = fs::canonicalize(&path).unwrap();
    let still_inside = canon_now.starts_with(fs::canonicalize(&allowed).unwrap());
    ok("G1. la recanonisation juste avant l'action détecte l'échange", !still_inside);
    // sans la mitigation, l'action suit le lien :
    trash::delete(&path).unwrap();
    ok("G2. SANS mitigation, trash::delete a suivi le lien et envoyé le fichier protégé à la corbeille", !forbidden.join("data.bin").exists());

    // H. suppression définitive d'un dossier contenant un lien : le lien est retiré, pas sa cible
    let victim = base.join("cache_a_vider");
    fs::create_dir(&victim).unwrap();
    symlink(&dir_target, victim.join("lien_dossier")).unwrap();
    fs::remove_dir_all(&victim).unwrap();
    ok("H. fs::remove_dir_all ne suit pas un lien contenu dans le dossier", dir_target.join("doc.txt").exists() && !victim.exists());
    let _ = Path::new("");
}
