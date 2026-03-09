param(
    [string]$file,
    [string]$output
)

$timeout = 30  # 30 seconds timeout

$process = Start-Process -FilePath ".\target\release\axonc.exe" `
    -ArgumentList @("build", $file, "--keep-temps", "-o", $output) `
    -NoNewWindow `
    -RedirectStandardOutput "temp_stdout.txt" `
    -RedirectStandardError "temp_stderr.txt" `
    -PassThru

$exited = $process | Wait-Process -Timeout $timeout -ErrorAction SilentlyContinue

if (-not $exited) {
    Write-Host "Process timeout after $timeout seconds, killing..."
    Stop-Process -Id $process.Id -Force
    Write-Host "COMPILATION TIMEOUT"
} else {
    $exitCode = $process.ExitCode
    Write-Host "Exit code: $exitCode"
    
    Write-Host "`n=== STDOUT ==="
    Get-Content temp_stdout.txt
    
    Write-Host "`n=== STDERR ==="
    Get-Content temp_stderr.txt
    
    # Check for generated IR if compilation failed
    if ($exitCode -ne 0) {
        $buildDir = "$($file).axon_build"
        $irFile = Join-Path $buildDir "program.ll"
        if (Test-Path $irFile) {
            Write-Host "`n=== Generated IR (program.ll) ==="
            Get-Content $irFile
        }
    }
}

Remove-Item temp_stdout.txt, temp_stderr.txt -ErrorAction SilentlyContinue
