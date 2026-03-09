@echo off
cd /d "H:\programing language\axon"

echo === STEP 1: Running unit tests ===
cargo test --lib 2>&1 | findstr "test result"

echo.
echo === STEP 2: Building release binary ===
cargo build --release 2>&1 | findstr "Finished" "error"

echo.
echo === STEP 3: Creating test_if.axon ===
(
echo fn main^(^) {
echo     let x: Int64 = 42;
echo     let mut y: Int64 = 0;
echo     if x ^> 10 {
echo         y = 1;
echo     } else {
echo         y = 2;
echo     }
echo }
) > test_if.axon
echo test_if.axon created

echo.
echo === STEP 4: Compiling test_if.axon ===
.\target\release\axonc.exe build test_if.axon --keep-temps -o test_if.exe 2>&1

if errorlevel 1 (
    echo === STEP 5: Showing error details ===
    type test_if.exe.axon_build\program.ll
) else (
    echo === STEP 5: Running test_if.exe ===
    .\test_if.exe
    echo exit: %ERRORLEVEL%
)

echo.
echo === STEP 6: Creating test_while.axon ===
(
echo fn main^(^) {
echo     let mut i: Int64 = 0;
echo     while i ^< 10 {
echo         i = i + 1;
echo     }
echo }
) > test_while.axon
echo test_while.axon created

echo.
echo === STEP 7: Compiling test_while.axon ===
.\target\release\axonc.exe build test_while.axon --keep-temps -o test_while.exe 2>&1

if errorlevel 1 (
    echo === STEP 8: Showing error details ===
    type test_while.exe.axon_build\program.ll
) else (
    echo === STEP 8: Running test_while.exe ===
    .\test_while.exe
    echo exit: %ERRORLEVEL%
)

echo === ALL STEPS COMPLETED ===
pause
