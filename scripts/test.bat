@echo off
REM Test runner script for Permata Gateway on Windows
REM Usage: scripts\test.bat [unit|integration|all]

echo === Permata Gateway Test Runner ===

if "%1"=="" goto all
if "%1"=="unit" goto unit
if "%1"=="integration" goto integration
if "%1"=="all" goto all
if "%1"=="help" goto help

:all
echo.
echo Running All Tests...
cargo test -- --nocapture
goto end

:unit
echo.
echo Running Unit Tests...
cargo test --test unit_tests -- --nocapture
goto end

:integration
echo.
echo Running Integration Tests...
cargo test --test integration_tests -- --nocapture
goto end

:help
echo.
echo Usage: scripts\test.bat [OPTION]
echo.
echo Options:
echo   all           Run all tests (default)
echo   unit          Run only unit tests
echo   integration   Run only integration tests
echo   help          Show this help message
goto end

:end
echo.
echo Test execution completed