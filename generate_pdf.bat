@echo off
echo [*] Generating Hydra OS Documentation PDF...
python "%~dp0scripts\generate_docs_pdf.py" %*
if errorlevel 1 (
    echo [!] PDF generation failed.
    exit /b 1
)
