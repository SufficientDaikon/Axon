@echo off
cd /d "H:\programing language\axon"

REM Test 1: Compile test_fib.axon
echo ======================================
echo TEST 1: test_fib.axon
echo ======================================
echo.
echo Compiling...
timeout /t 1 /nobreak >nul
.\target\release\axonc.exe build test_fib.axon --keep-temps -o test_fib.exe 2>&1
echo.
echo Exit code: %ERRORLEVEL%
echo.

if exist test_fib.exe (
    echo Executable created, running...
    test_fib.exe
) else (
    echo Compilation failed or timeout
    if exist test_fib.exe.axon_build\program.ll (
        echo.
        echo === Generated IR ===
        type test_fib.exe.axon_build\program.ll
    )
)

echo.
echo.
REM Test 2: Compile test_if.axon
echo ======================================
echo TEST 2: test_if.axon
echo ======================================
echo.
echo Compiling...
timeout /t 1 /nobreak >nul
.\target\release\axonc.exe build test_if.axon --keep-temps -o test_if.exe 2>&1
echo.
echo Exit code: %ERRORLEVEL%
echo.

if exist test_if.exe (
    echo Executable created, running...
    test_if.exe
) else (
    echo Compilation failed or timeout
    if exist test_if.exe.axon_build\program.ll (
        echo.
        echo === Generated IR ===
        type test_if.exe.axon_build\program.ll
    )
)

echo.
echo.
REM Test 3: Compile test_while.axon
echo ======================================
echo TEST 3: test_while.axon
echo ======================================
echo.
echo Compiling...
timeout /t 1 /nobreak >nul
.\target\release\axonc.exe build test_while.axon --keep-temps -o test_while.exe 2>&1
echo.
echo Exit code: %ERRORLEVEL%
echo.

if exist test_while.exe (
    echo Executable created, running...
    test_while.exe
) else (
    echo Compilation failed or timeout
    if exist test_while.exe.axon_build\program.ll (
        echo.
        echo === Generated IR ===
        type test_while.exe.axon_build\program.ll
    )
)

pause
