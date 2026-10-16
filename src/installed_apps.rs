use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub executable_name: Option<String>,
    pub is_distraction: bool,
    pub category: String, // "Gaming", "Social/Chat", "Media", "Productivity"
}

// Known distraction list for automatic classification
const KNOWN_GAMES_AND_CHAT: &[(&str, &str, &str)] = &[
    ("discord", "discord.exe", "Social/Chat"),
    ("steam", "steam.exe", "Gaming"),
    ("epic games", "epicgameslauncher.exe", "Gaming"),
    ("spotify", "spotify.exe", "Media"),
    ("riot client", "riotclientservices.exe", "Gaming"),
    ("valorant", "valorant.exe", "Gaming"),
    ("league of legends", "leagueclient.exe", "Gaming"),
    ("battle.net", "battle.net.exe", "Gaming"),
    ("telegram", "telegram.exe", "Social/Chat"),
    ("whatsapp", "whatsapp.exe", "Social/Chat"),
    ("genshin impact", "genshinimpact.exe", "Gaming"),
    ("zenless zone zero", "zenlesszonezero.exe", "Gaming"),
    ("roblox", "robloxplayerbeta.exe", "Gaming"),
];

fn read_reg_string(hkey: HKEY, value_name: &str) -> Option<String> {
    let mut val_u16: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
    let mut data_type = 0u32;
    let mut data_size = 0u32;

    unsafe {
        if RegQueryValueExW(
            hkey,
            PCWSTR(val_u16.as_ptr()),
            None,
            Some(&mut data_type),
            None,
            Some(&mut data_size),
        )
        .is_err()
            || data_type != REG_SZ.0
        {
            return None;
        }

        let mut buffer: Vec<u16> = vec![0; (data_size as usize / 2) + 1];
        if RegQueryValueExW(
            hkey,
            PCWSTR(val_u16.as_ptr()),
            None,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        )
        .is_ok()
        {
            let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
            return Some(String::from_utf16_lossy(&buffer[..len]));
        }
    }
    None
}

fn scan_registry_key(root: HKEY, subkey: &str, apps: &mut Vec<InstalledApp>, seen: &mut HashSet<String>) {
    let subkey_u16: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();
    let mut hkey = HKEY::default();

    unsafe {
        if RegOpenKeyExW(root, PCWSTR(subkey_u16.as_ptr()), 0, KEY_READ, &mut hkey).is_err() {
            return;
        }

        let mut index = 0;
        let mut name_buffer = [0u16; 256];

        loop {
            let mut name_len = name_buffer.len() as u32;
            if RegEnumKeyExW(
                hkey,
                index,
                windows::core::PWSTR(name_buffer.as_mut_ptr()),
                &mut name_len,
                None,
                windows::core::PWSTR::null(),
                None,
                None,
            )
            .is_err()
            {
                break;
            }

            let sub_name = String::from_utf16_lossy(&name_buffer[..name_len as usize]);
            let full_sub_path = format!("{}\\{}", subkey, sub_name);
            let full_sub_u16: Vec<u16> = full_sub_path.encode_utf16().chain(Some(0)).collect();

            let mut app_key = HKEY::default();
            if RegOpenKeyExW(root, PCWSTR(full_sub_u16.as_ptr()), 0, KEY_READ, &mut app_key).is_ok() {
                if let Some(display_name) = read_reg_string(app_key, "DisplayName") {
                    let display_icon = read_reg_string(app_key, "DisplayIcon");
                    let name_trimmed = display_name.trim().to_string();

                    if !name_trimmed.is_empty() && !seen.contains(&name_trimmed.to_lowercase()) {
                        seen.insert(name_trimmed.to_lowercase());

                        // Extract executable name from DisplayIcon if available
                        let exe_candidate = display_icon.and_then(|icon_path| {
                            let clean = icon_path.trim_matches('"').split(',').next()?.trim();
                            if clean.to_lowercase().ends_with(".exe") {
                                std::path::Path::new(clean)
                                    .file_name()
                                    .map(|f| f.to_string_lossy().to_string().to_lowercase())
                            } else {
                                None
                            }
                        });

                        // Categorize app against distraction dictionary
                        let lower = name_trimmed.to_lowercase();
                        let mut is_distraction = false;
                        let mut category = "General".to_string();
                        let mut matched_exe = exe_candidate;

                        for &(distract_key, known_exe, cat) in KNOWN_GAMES_AND_CHAT {
                            if lower.contains(distract_key) {
                                is_distraction = true;
                                category = cat.to_string();
                                if matched_exe.is_none() {
                                    matched_exe = Some(known_exe.to_string());
                                }
                                break;
                            }
                        }

                        apps.push(InstalledApp {
                            name: name_trimmed,
                            executable_name: matched_exe,
                            is_distraction,
                            category,
                        });
                    }
                }
                let _ = RegCloseKey(app_key);
            }

            index += 1;
        }

        let _ = RegCloseKey(hkey);
    }
}

pub fn get_installed_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    let mut seen = HashSet::new();

    // 64-bit Machine Registry
    scan_registry_key(
        HKEY_LOCAL_MACHINE,
        "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        &mut apps,
        &mut seen,
    );
    // 32-bit Machine Registry
    scan_registry_key(
        HKEY_LOCAL_MACHINE,
        "Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        &mut apps,
        &mut seen,
    );
    // Current User Registry
    scan_registry_key(
        HKEY_CURRENT_USER,
        "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        &mut apps,
        &mut seen,
    );

    // Sort: distractions first, then alphabetical
    apps.sort_by(|a, b| b.is_distraction.cmp(&a.is_distraction).then(a.name.cmp(&b.name)));
    apps
}