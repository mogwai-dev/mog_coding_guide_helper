@echo off
REM Coding Guide Helper - Eclipse LSP Setup Script (Windows)

echo === Coding Guide Helper Eclipse Setup ===
echo.

REM LSPサーバーのパスを検索
set "LSP_SERVER="
if exist "..\..\target\release\coding-guide-helper-lsp.exe" (
    pushd ..\..\target\release
    set "LSP_SERVER=%CD%\coding-guide-helper-lsp.exe"
    popd
) else if exist "..\..\target\debug\coding-guide-helper-lsp.exe" (
    pushd ..\..\target\debug
    set "LSP_SERVER=%CD%\coding-guide-helper-lsp.exe"
    popd
) else (
    echo Error: LSP server not found. Please build it first:
    echo   cargo build --release --package coding-guide-helper-lsp
    exit /b 1
)

echo Found LSP server: %LSP_SERVER%
echo.

REM パスのエスケープ（バックスラッシュを二重に）
set "ESCAPED_PATH=%LSP_SERVER:\=\\%"

REM Eclipseワークスペースディレクトリの確認
set "WORKSPACE=%USERPROFILE%\eclipse-workspace"
if not exist "%WORKSPACE%" (
    echo.
    echo Eclipse workspace not found at: %WORKSPACE%
    echo.
    echo Please configure manually:
    echo   1. Window -^> Preferences -^> Language Servers
    echo   2. Add new server with path: %LSP_SERVER%
    echo   3. Content type: C Source File
    pause
    exit /b 0
)

echo Workspace found: %WORKSPACE%
echo.

REM 設定ファイルディレクトリを作成
set "SETTINGS_DIR=%WORKSPACE%\.metadata\.plugins\org.eclipse.core.runtime\.settings"
if not exist "%SETTINGS_DIR%" (
    mkdir "%SETTINGS_DIR%"
)

REM 設定ファイルを生成
set "PREFS_FILE=%SETTINGS_DIR%\org.eclipse.lsp4e.prefs"
echo Creating preferences file: %PREFS_FILE%

(
echo eclipse.preferences.version=1
echo languageServers=coding-guide-helper
echo coding-guide-helper.command=%ESCAPED_PATH%
echo coding-guide-helper.contentType=org.eclipse.cdt.core.cSource
echo coding-guide-helper.contentType.1=org.eclipse.cdt.core.cHeader
echo coding-guide-helper.initializationOptions={}
echo coding-guide-helper.trace=off
) > "%PREFS_FILE%"

echo.
echo Setup complete!
echo.
echo Next steps:
echo   1. Restart Eclipse
echo   2. Open a C project
echo   3. LSP should activate automatically for .c and .h files
echo.
echo To verify:
echo   Window -^> Preferences -^> Language Servers
echo.
pause
