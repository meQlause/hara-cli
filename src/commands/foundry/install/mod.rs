pub mod shell;

use std::process::{Command, Stdio};

/// Orchestrates the installation of Foundry.
pub fn run() -> Result<(), String> {
    if cfg!(windows) {
        return run_powershell();
    }

    run_bash()
}

fn run_powershell() -> Result<(), String> {
    tracing::info!("Installing Foundry for HARA development (Native PowerShell)...");

    let pwsh_script = r#"
        $ErrorActionPreference = 'Stop';
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12;
        $repo = 'foundry-rs/foundry';
        $api = 'https://api.github.com/repos/' + $repo + '/releases/latest';
        $release = Invoke-RestMethod -Uri $api -UseBasicParsing;
        $asset = $release.assets | Where-Object { $_.name -like '*win32_amd64.tar.gz' } | Select-Object -First 1;
        if ($null -eq $asset) { throw 'Foundry Windows asset not found' };
        $url = $asset.browser_download_url;
        $dest = Join-Path $env:TEMP 'foundry.tar.gz';
        Write-Host 'Downloading Foundry from GitHub...';
        Invoke-WebRequest -Uri $url -OutFile $dest;
        $installDir = Join-Path $HOME '.foundry\bin';
        if (-not (Test-Path $installDir)) { New-Item -ItemType Directory -Path $installDir -Force | Out-Null };
        Write-Host 'Extracting to .foundry/bin...';
        tar -xzf $dest -C $installDir;
        Remove-Item $dest;

        # Unblock binaries
        Get-ChildItem -Path $installDir -Filter *.exe | Unblock-File;

        # Update PATH for Foundry
        $u = [Environment]::GetEnvironmentVariable('Path', 'User');
        if ($u -notlike "*$installDir*") {
            [Environment]::SetEnvironmentVariable('Path', "$installDir;$u", 'User');
            $env:Path = "$installDir;$env:Path";
            Write-Host 'Success! Added .foundry/bin to your PATH.';
        }

        Write-Host 'Foundry binaries ready.';
    "#;

    let status = Command::new(shell::which_powershell())
        .args(["-Command", pwsh_script])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run PowerShell: {e}"))?;

    if !status.success() {
        return Err("Foundry installation via PowerShell failed.".to_string());
    }

    tracing::info!("Foundry installed successfully!");
    tracing::info!("Next step: cd into your project folder and run: hara foundry init");
    Ok(())
}

fn run_bash() -> Result<(), String> {
    tracing::info!("Installing Foundry for HARA development (via Bash)...");

    let install_cmd = "curl -fsSL https://foundry.paradigm.xyz | bash";
    let shell_bin = shell::which_shell();

    let status = Command::new(&shell_bin)
        .args(["-c", install_cmd])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run installer (requires bash + curl): {e}"))?;

    if !status.success() {
        let fpath = shell::foundryup_path();
        if fpath.exists() && fpath.to_string_lossy() != "foundryup.exe" {
            tracing::info!("Foundry installer returned non-zero, but foundryup.exe was found.");
            tracing::info!("Proceeding with installation...");
        } else {
            return Err(
                "Foundry installer failed.\n  Make sure curl and bash are available."
                    .to_string(),
            );
        }
    }

    tracing::info!("foundryup downloaded.");
    tracing::info!("Running foundryup to install forge/cast/anvil...");

    let foundryup = shell::foundryup_path();
    let unix_path = foundryup.to_string_lossy().replace('\\', "/");

    let status = Command::new(&shell_bin)
        .args(["-c", &unix_path])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {}
        _ => {
            tracing::info!("foundryup returned non-zero or failed to run automatically.");
            tracing::info!("Try restarting your terminal and run: foundryup");
            return Ok(());
        }
    }

    tracing::info!("Foundry installed successfully!");
    tracing::info!("Next step: cd into your project folder and run: hara foundry init");
    Ok(())
}
