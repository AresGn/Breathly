// PROTOTYPE JETABLE : faux « service Breathly » sur socket de domaine Unix, droits 0600.
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
fn main() {
    let path = std::env::args().nth(1).expect("chemin du socket");
    let _ = std::fs::remove_file(&path);
    let l = UnixListener::bind(&path).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    println!("service prêt sur {path} mode {:o}", std::fs::metadata(&path).unwrap().permissions().mode() & 0o777);
    for s in l.incoming().flatten() {
        let mut s = s; let mut b = [0u8; 16]; let _ = s.read(&mut b);
        let _ = s.write_all(b"{\"ram_libre_mo\":1234,\"etat\":\"actif\"}");
    }
}
