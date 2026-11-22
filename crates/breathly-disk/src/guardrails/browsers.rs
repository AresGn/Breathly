//! Emplacements des profils de navigateur (S5), relatifs au dossier personnel.
//!
//! Sources : Firefox, « Profiles - Where Firefox stores your bookmarks, passwords and other user data »
//! (`support.mozilla.org/kb/profiles-where-firefox-stores-user-data`) ; Chrome et Chromium, « User Data Directory »
//! (`chromium.googlesource.com/chromium/src/+/main/docs/user_data_dir.md`). Les autres entrées (Edge, Brave, Vivaldi,
//! Opera, paquets Snap et Flatpak) suivent la même convention de Chromium et de Firefox ; elles n'ont pas été
//! relues à la source dans le ticket #72 (à confirmer à la revue). Les deux systèmes sont toujours contrôlés : un
//! profil Windows sur un disque monté sous Linux reste protégé. Sous Windows, `%APPDATA%` et `%LOCALAPPDATA%` sont
//! lus à leur valeur par défaut (sous le dossier personnel) ; une redirection se couvre par la règle elle-même.

pub(super) const PROFILE_LOCATIONS: [&str; 17] = [
    // Linux
    ".mozilla/firefox",
    "snap/firefox/common/.mozilla/firefox",
    ".var/app/org.mozilla.firefox/.mozilla/firefox",
    ".config/google-chrome",
    ".config/chromium",
    "snap/chromium/common/chromium",
    ".config/microsoft-edge",
    ".config/BraveSoftware",
    ".config/vivaldi",
    ".config/opera",
    // Windows
    "AppData/Roaming/Mozilla/Firefox",
    "AppData/Local/Google/Chrome/User Data",
    "AppData/Local/Chromium/User Data",
    "AppData/Local/Microsoft/Edge/User Data",
    "AppData/Local/BraveSoftware/Brave-Browser/User Data",
    "AppData/Local/Vivaldi/User Data",
    "AppData/Roaming/Opera Software",
];
