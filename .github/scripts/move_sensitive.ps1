<#
Moves listed sensitive files out of the repository root into a timestamped backup directory
and sets restrictive ACLs. This is intended to be run locally by a developer (not in CI).

Usage (PowerShell):
<#
Moves listed sensitive files out of the repository root into a timestamped backup directory
and sets restrictive ACLs. This is intended to be run locally by a developer (not in CI).

Usage (PowerShell):
  ./.github/scripts/move_sensitive.ps1

#>
param()

# Resolve repository root (script directory)
$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition

# Backup root under the user's profile
$backupRoot = Join-Path $env:USERPROFILE "ark_repo_sensitive_backup"
if (-not (Test-Path $backupRoot)) {
    New-Item -ItemType Directory -Path $backupRoot | Out-Null
}

$ts = Get-Date -Format "yyyyMMdd_HHmmss"
$dest = Join-Path $backupRoot "backup_$ts"
New-Item -ItemType Directory -Path $dest | Out-Null

# Files to move (relative to repo root)
$files = @("keystore.json", "priv.hex", "pwd.txt")

$moved = @()
foreach ($f in $files) {
    $path = Join-Path $repoRoot $f
    if (Test-Path $path) {
        $target = Join-Path $dest $f
        Write-Host "Moving $path -> $target"
        try {
            Move-Item -Path $path -Destination $target -Force
        } catch {
            Write-Warning ("Failed to move {0}: {1}" -f $path, $_)
            continue
        }

        # set ACL: allow only current user full control
        try {
            $acl = Get-Acl $target
            $acl.SetAccessRuleProtection($true, $false)
            $rule = New-Object System.Security.AccessControl.FileSystemAccessRule($env:USERNAME, "FullControl", "Allow")
            $acl.SetAccessRule($rule)
            Set-Acl -Path $target -AclObject $acl
        } catch {
            Write-Warning ("Could not set ACL on {0}: {1}" -f $target, $_)
        }

        $moved += $target
    } else {
        Write-Host "$f not present in repo root"
    }
}

if ($moved.Count -eq 0) {
    Write-Host "No sensitive files were found in the repository root."
} else {
    Write-Host "Moved files:"; $moved | ForEach-Object { Write-Host " - $_" }
    Write-Host "Backup directory: $dest"
}
                $target = Join-Path $dest $f
