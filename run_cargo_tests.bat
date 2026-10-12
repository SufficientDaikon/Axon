@echo off
cd /d "H:\programing language\axon"

echo.
echo ========== Test Run 1: cargo test --lib ==========
echo.
call cargo test --lib 2>&1
echo.
echo.

echo ========== Test Run 2: cargo test --test ai_framework_tests ==========
echo.
call cargo test --test ai_framework_tests 2>&1
echo.
echo.

echo ========== Test Run 3: cargo test --test stdlib_tests ==========
echo.
call cargo test --test stdlib_tests 2>&1
echo.
echo.

echo ========== All tests completed ==========
pause
