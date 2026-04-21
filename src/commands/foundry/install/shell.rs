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

/// Returns the path to bash from Git Bash on Windows.
pub fn which_shell() -> String {
    let candidates = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            return c.to_string();
        }
    }
    "bash".to_string()
}
