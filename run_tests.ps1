cd 'H:\programing language\axon'
Write-Host "=== STEP 1: Running unit tests ==="
cargo test --lib 2>&1 | Select-String "test result"

Write-Host "`n=== STEP 2: Building release binary ==="
cargo build --release 2>&1 | Select-String "Finished|error"

Write-Host "`n=== STEP 3: Creating test_if.axon ==="
@"
fn main() {
    let x: Int64 = 42;
    let mut y: Int64 = 0;
    if x > 10 {
        y = 1;
    } else {
        y = 2;
    }
}
"@ | Set-Content -Path test_if.axon -Encoding UTF8
Write-Host "test_if.axon created"

Write-Host "`n=== STEP 4: Compiling test_if.axon ==="
$step4_output = .\target\release\axonc.exe build test_if.axon --keep-temps -o test_if.exe 2>&1
$step4_output
$step4_success = $LASTEXITCODE

if ($step4_success -eq 0) {
    Write-Host "`n=== STEP 5: Running test_if.exe ==="
    .\test_if.exe
    Write-Host "exit: $LASTEXITCODE"
} else {
    Write-Host "`n=== STEP 5: Showing error details ==="
    Get-Content test_if.exe.axon_build\program.ll
}

Write-Host "`n=== STEP 6: Creating test_while.axon ==="
@"
fn main() {
    let mut i: Int64 = 0;
    while i < 10 {
        i = i + 1;
    }
}
"@ | Set-Content -Path test_while.axon -Encoding UTF8
Write-Host "test_while.axon created"

Write-Host "`n=== STEP 7: Compiling test_while.axon ==="
$step7_output = .\target\release\axonc.exe build test_while.axon --keep-temps -o test_while.exe 2>&1
$step7_output
$step7_success = $LASTEXITCODE

if ($step7_success -eq 0) {
    Write-Host "`n=== STEP 8: Running test_while.exe ==="
    .\test_while.exe
    Write-Host "exit: $LASTEXITCODE"
} else {
    Write-Host "`n=== STEP 8: Showing error details ==="
    Get-Content test_while.exe.axon_build\program.ll
}

Write-Host "`n=== ALL STEPS COMPLETED ==="
