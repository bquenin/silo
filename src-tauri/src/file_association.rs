use serde::Serialize;
use std::path::Path;

const EXTENSION: &str = ".kwreplay";
const PROG_ID: &str = "Tacitus.KWReplay";

#[derive(Debug, Serialize)]
pub struct AssociationStatus {
    pub supported: bool,
    pub associated: bool,
}

pub fn status(executable: &Path) -> anyhow::Result<AssociationStatus> {
    platform::status(executable)
}

pub fn associate(executable: &Path) -> anyhow::Result<AssociationStatus> {
    platform::associate(executable)
}

fn open_command(executable: &Path) -> String {
    format!("\"{}\" \"%1\"", executable.display())
}

#[cfg(windows)]
mod platform {
    use super::{open_command, AssociationStatus, EXTENSION, PROG_ID};
    use anyhow::Context;
    use std::path::Path;
    use windows_sys::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    const CLASSES: &str = r"Software\Classes";

    pub fn status(executable: &Path) -> anyhow::Result<AssociationStatus> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let classes = match hkcu.open_subkey_with_flags(CLASSES, KEY_READ) {
            Ok(key) => key,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(AssociationStatus {
                    supported: true,
                    associated: false,
                });
            }
            Err(error) => return Err(error).context("Could not read Windows file associations"),
        };

        let extension: Option<String> = classes
            .open_subkey_with_flags(EXTENSION, KEY_READ)
            .ok()
            .and_then(|key| key.get_value("").ok());
        let command: Option<String> = classes
            .open_subkey_with_flags(format!(r"{PROG_ID}\shell\open\command"), KEY_READ)
            .ok()
            .and_then(|key| key.get_value("").ok());
        let user_choice: Option<String> = hkcu
            .open_subkey_with_flags(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.kwreplay\UserChoice",
                KEY_READ,
            )
            .ok()
            .and_then(|key| key.get_value("ProgId").ok());
        let associated = user_choice.as_deref().or(extension.as_deref()) == Some(PROG_ID)
            && command
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case(&open_command(executable)));

        Ok(AssociationStatus {
            supported: true,
            associated,
        })
    }

    pub fn associate(executable: &Path) -> anyhow::Result<AssociationStatus> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (classes, _) = hkcu
            .create_subkey(CLASSES)
            .context("Could not open the Windows file-association registry")?;

        let (kind, _) = classes.create_subkey(PROG_ID)?;
        kind.set_value("", &"Kane's Wrath Replay")?;
        kind.set_value("FriendlyTypeName", &"Kane's Wrath Replay")?;

        let (icon, _) = classes.create_subkey(format!(r"{PROG_ID}\DefaultIcon"))?;
        icon.set_value("", &format!("\"{}\",0", executable.display()))?;

        let (command, _) = classes.create_subkey(format!(r"{PROG_ID}\shell\open\command"))?;
        command.set_value("", &open_command(executable))?;

        let (extension, _) = classes
            .create_subkey(EXTENSION)
            .context("Could not register the .kwreplay extension")?;
        extension.set_value("", &PROG_ID)?;
        let (handlers, _) = extension.create_subkey("OpenWithProgids")?;
        handlers.set_value(PROG_ID, &"")?;

        // Tell Explorer that extension and icon metadata changed.
        unsafe {
            SHChangeNotify(
                SHCNE_ASSOCCHANGED as i32,
                SHCNF_IDLIST,
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        status(executable)
    }
}

#[cfg(not(windows))]
mod platform {
    use super::AssociationStatus;
    use std::path::Path;

    pub fn status(_executable: &Path) -> anyhow::Result<AssociationStatus> {
        Ok(AssociationStatus {
            supported: false,
            associated: false,
        })
    }

    pub fn associate(_executable: &Path) -> anyhow::Result<AssociationStatus> {
        anyhow::bail!("File association is only available on Windows")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_quotes_the_executable_and_replay_path() {
        assert_eq!(
            open_command(Path::new(r"C:\Games and tools\tacitus.exe")),
            r#""C:\Games and tools\tacitus.exe" "%1""#,
        );
    }
}
