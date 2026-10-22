#!/usr/bin/env python3
"""
Convenience entry point for scripts/generate_docs_pdf.py
"""
import sys
from pathlib import Path

# Add scripts directory to path and execute main
scripts_dir = Path(__file__).resolve().parent / "scripts"
sys.path.insert(0, str(scripts_dir))

from generate_docs_pdf import main

if __name__ == "__main__":
    sys.exit(main())
