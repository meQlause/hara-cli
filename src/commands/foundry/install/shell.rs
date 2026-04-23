use std::env;
use std::path::{PathBuf, Path};

/// Returns the path to foundryup.exe.
pub fn foundryup_path() -> PathBuf {
    let home = env::var("USERPROFILE").unwrap_or_default();
    let p = PathBuf::from(&home).join(".foundry").join("bin").join("foundryup.exe");
    if p.exists() {
        p
    } else {
        PathBuf::from("foundryup.exe")
    }
}

/// Returns true if a bash-compatible shell was found.
pub fn has_bash() -> bool {
    let shell = which_shell();
    if shell == "bash" {
        // Double check if 'bash' is in PATH and working
        Command::new("bash")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        Path::new(&shell).exists()
    }
}

/// Returns the path to the best available PowerShell executable (pwsh then powershell).
pub fn which_powershell() -> String {
    if Command::new("pwsh")
        .arg("-Command")
        .arg("exit")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        "pwsh".to_string()
    } else {
        "powershell".to_string()
    }
}

/// Returns the path to bash from Git Bash on Windows.
pub fn which_shell() -> String {
    let candidates = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files\Git\usr\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\usr\bin\bash.exe",
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            return c.to_string();
        }
    }
    "bash".to_string()
}

use std::process::{Command, Stdio};
