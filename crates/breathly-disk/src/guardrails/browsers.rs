//! Emplacements des profils de navigateur (S5), relatifs au dossier personnel.
//!
//! Sources : Firefox, « Profiles - Where Firefox stores your bookmarks, passwords and other user data »
//! (`support.mozilla.org/kb/profiles-where-firefox-stores-user-data`) ; Chrome et Chromium, « User Data Directory »
//! (`chromium.googlesource.com/chromium/src/+/main/docs/user_data_dir.md`). Les autres entrées (Edge, Brave, Vivaldi,
//! Opera, paquets Snap et Flatpak, dossier XDG de Firefox, paquet Microsoft Store de Firefox) suivent la même
//! convention ; **elles n'ont pas été relues à la source** (à confirmer à la revue). Les deux systèmes sont toujours
//! contrôlés : un profil Windows sur un disque monté sous Linux reste protégé. Sous Windows, `%APPDATA%` et
//! `%LOCALAPPDATA%` sont lus à leur valeur par défaut (sous le dossier personnel).

pub(super) const PROFILE_LOCATIONS: [&str; 25] = [
    // Linux
    ".mozilla/firefox",
    ".config/mozilla/firefox",
    "snap/firefox/common/.mozilla/firefox",
    ".var/app/org.mozilla.firefox/.mozilla/firefox",
    ".var/app/org.mozilla.firefox/config/mozilla/firefox",
    ".config/google-chrome",
    ".config/chromium",
    "snap/chromium/common/chromium",
    ".var/app/com.google.Chrome/config/google-chrome",
    ".var/app/org.chromium.Chromium/config/chromium",
    ".config/microsoft-edge",
    ".var/app/com.microsoft.Edge/config/microsoft-edge",
    ".config/BraveSoftware",
    ".var/app/com.brave.Browser/config/BraveSoftware",
    ".config/vivaldi",
    ".var/app/com.vivaldi.Vivaldi/config/vivaldi",
    ".config/opera",
    ".var/app/com.opera.Opera/config/opera",
    // Windows
    "AppData/Roaming/Mozilla/Firefox",
    "AppData/Local/Google/Chrome/User Data",
    "AppData/Local/Chromium/User Data",
    "AppData/Local/Microsoft/Edge/User Data",
    "AppData/Local/BraveSoftware/Brave-Browser/User Data",
    "AppData/Local/Vivaldi/User Data",
    "AppData/Roaming/Opera Software",
];

/// Dossier des paquets Microsoft Store et préfixe du dossier de Firefox (nom suivi d'un identifiant d'éditeur) :
/// ses profils sont dans `LocalCache/Roaming/Mozilla/Firefox` sous ce dossier.
pub(super) const PACKAGES_FOLDER: &str = "AppData/Local/Packages";
pub(super) const FIREFOX_PACKAGE_PREFIX: &str = "mozilla.firefox_";
pub(super) const FIREFOX_PACKAGE_PROFILE: &str = "LocalCache/Roaming/Mozilla/Firefox";
