# Set SKIP_BUILD_NUMBER environment variable
$env:SKIP_BUILD_NUMBER = "1"

# Read all lines from Cargo.toml
$content = Get-Content -Path "Cargo.toml"

$inFeaturesSection = $false
$features = @()

# Parse the file to extract features from the [features] section.
foreach ($line in $content) {
    if ($line -match '^\[features\]') {
        $inFeaturesSection = $true
        continue
    }
    # If a new section starts, stop reading features.
    if ($inFeaturesSection -and $line -match '^\[') {
        break
    }
    if ($inFeaturesSection -and $line -match '^\s*[^#\s]') {
        # Get the feature name (the text before '=')
        $featureName = $line.Split('=')[0].Trim()
        if ($featureName) {
            $features += $featureName
        }
    }
}

if ($features.Count -eq 0) {
    Write-Error "No features found in Cargo.toml."
    exit 1
}

# Extract crate name from Cargo.toml (assumes first occurrence of 'name =')
$crateNameLine = $content | Where-Object { $_ -match '^\s*name\s*=' } | Select-Object -First 1
if (-not $crateNameLine) {
    Write-Error "Crate name not found in Cargo.toml."
    exit 1
}
$crateName = $crateNameLine -replace '.*=\s*"(.*)".*', '$1'

Write-Host "Detected crate: $crateName"
Write-Host "Features found: $($features -join ', ')"

# Loop over each feature to build the project.
foreach ($feature in $features) {
    # If the feature is "default", append "latest" to the feature list and binary suffix.
    if ($feature -eq "default") {
        continue
    } else {
        $buildFeatures = $feature
        $binarySuffix = $feature
    }

    Write-Host "Building with feature: $buildFeatures"

    # Build the project with only this feature (disable defaults).
    cargo build --release --no-default-features --features $buildFeatures

    # For Windows, the binary will have a .exe extension.
    $sourceBinary = "target\release\$crateName.exe"
    $destBinary = "target\release\$crateName" + "_$binarySuffix.exe"

    if (Test-Path $sourceBinary) {
        Move-Item -Path $sourceBinary -Destination $destBinary -Force
        Write-Host "Built binary saved as: $destBinary"
    } else {
        Write-Error "Expected binary not found: $sourceBinary"
    }
}
