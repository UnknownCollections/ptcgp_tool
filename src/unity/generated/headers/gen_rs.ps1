$bindgenArgs = @(
    "--no-layout-tests"
    "--impl-debug"
    "--with-derive-default"
    "--no-doc-comments"
    "--enable-cxx-namespaces"
    "--disable-header-comment"
    "--ignore-functions"
    "--no-prepend-enum-name"
    "--use-array-pointers-in-arguments"
    "--raw-line", "#![allow(unused_qualifications)]",
    "--raw-line", "#![allow(unsafe_op_in_unsafe_fn)]"
)

Get-ChildItem -Path . -Filter "*.h" | ForEach-Object {
    $headerFileFullPath = $_.FullName
    $headerFileName = $_.Name
    $headerDirectory = $_.DirectoryName
    $baseName = [System.IO.Path]::GetFileNameWithoutExtension($headerFileName) -replace '[^a-zA-Z0-9]', ''

    $outputFile = Join-Path -Path $headerDirectory -ChildPath "../il2cpp_$baseName.rs"

    Write-Host "Processing '$headerFileName' -> '$($outputFile)'"

    $currentBindgenArgs = @(
        $headerFileFullPath,
        "-o", $outputFile
    ) + $bindgenArgs

    $clangArgs = @(
        "--target=aarch64-linux-android"
        "-x", "c++"
    )

    $finalCmdArgs = $currentBindgenArgs + @("--") + $clangArgs

    Write-Host "  Command: & bindgen.exe $($finalCmdArgs -join ' ')"

    try {
        & bindgen.exe @finalCmdArgs

        if ($LASTEXITCODE -ne 0) {
            Write-Error "  bindgen.exe failed for '$headerFileName' with exit code $LASTEXITCODE"
        } else {
            Write-Host "  Successfully generated '$outputFile'"
        }
    } catch {
        Write-Error "  Error executing bindgen.exe for '$headerFileName': $_"
    }

    Write-Host ""
}

Write-Host "Script finished."