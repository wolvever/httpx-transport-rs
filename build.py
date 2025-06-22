#!/usr/bin/env python3
"""
Simple build script for rust-httpx-transport development.
"""

import subprocess
import sys
import os
from pathlib import Path

def run_command(cmd, cwd=None):
    """Run a command and return True if successful."""
    print(f"Running: {' '.join(cmd)}")
    try:
        result = subprocess.run(cmd, cwd=cwd, check=True, capture_output=True, text=True)
        if result.stdout:
            print(result.stdout)
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {e}")
        if e.stdout:
            print("STDOUT:", e.stdout)
        if e.stderr:
            print("STDERR:", e.stderr)
        return False

def check_dependencies():
    """Check if required tools are installed."""
    print("Checking dependencies...")
    
    # Check for Python
    if not run_command([sys.executable, "--version"]):
        print("Python is required but not found")
        return False
    
    # Check for uv
    if not run_command(["uv", "--version"]):
        print("uv is required but not found")
        print("Install uv from: https://docs.astral.sh/uv/getting-started/installation/")
        return False
    
    # Check for Rust
    if not run_command(["rustc", "--version"]):
        print("Rust is required but not found")
        print("Install Rust from: https://rustup.rs/")
        return False
    
    # Check for maturin
    if not run_command(["uv", "pip", "show", "maturin"]):
        print("Maturin is required but not found")
        print("Installing maturin...")
        if not run_command(["uv", "pip", "install", "maturin"]):
            print("Failed to install maturin")
            return False
    
    print("✅ All dependencies are available")
    return True

def build_debug():
    """Build the project in debug mode."""
    print("\n🔨 Building in debug mode...")
    return run_command(["maturin", "develop"])

def build_release():
    """Build the project in release mode."""
    print("\n🚀 Building in release mode...")
    return run_command(["maturin", "develop", "--release"])

def run_tests():
    """Run the test suite."""
    print("\n🧪 Running tests...")
    return run_command(["uv", "run", "pytest", "tests/", "-v"])

def check_code():
    """Run code quality checks."""
    print("\n🔍 Running code checks...")
    
    # Check Rust code
    print("Checking Rust code...")
    if not run_command(["cargo", "check"]):
        return False
    
    if not run_command(["cargo", "clippy", "--", "-D", "warnings"]):
        print("Warning: Clippy found issues")
    
    # Check Python code if available
    if run_command(["uv", "run", "--quiet", "ruff", "--version"]):
        print("Checking Python code with ruff...")
        if not run_command(["uv", "run", "ruff", "check", "python/", "tests/", "examples/"]):
            print("Warning: Ruff found issues")
    else:
        print("Ruff not available, skipping Python checks")
    
    return True

def clean():
    """Clean build artifacts."""
    print("\n🧹 Cleaning build artifacts...")
    
    # Clean Rust artifacts
    run_command(["cargo", "clean"])
    
    # Clean Python artifacts
    import shutil
    for pattern in ["build", "dist", "*.egg-info"]:
        for path in Path(".").glob(pattern):
            if path.is_dir():
                shutil.rmtree(path)
                print(f"Removed {path}")

def main():
    """Main build script entry point."""
    if len(sys.argv) < 2:
        print("Usage: python build.py <command>")
        print("Commands:")
        print("  check      - Check dependencies and code")
        print("  build      - Build in debug mode") 
        print("  release    - Build in release mode")
        print("  test       - Run tests")
        print("  clean      - Clean build artifacts")
        print("  all        - Check, build, and test")
        return
    
    command = sys.argv[1]
    
    if command == "check":
        if not check_dependencies():
            sys.exit(1)
        if not check_code():
            sys.exit(1)
    
    elif command == "build":
        if not check_dependencies():
            sys.exit(1)
        if not build_debug():
            sys.exit(1)
    
    elif command == "release":
        if not check_dependencies():
            sys.exit(1)
        if not build_release():
            sys.exit(1)
    
    elif command == "test":
        if not run_tests():
            sys.exit(1)
    
    elif command == "clean":
        clean()
    
    elif command == "all":
        if not check_dependencies():
            sys.exit(1)
        if not check_code():
            sys.exit(1)
        if not build_debug():
            sys.exit(1)
        if not run_tests():
            print("⚠️ Tests failed, but build succeeded")
    
    else:
        print(f"Unknown command: {command}")
        sys.exit(1)
    
    print("\n✅ Done!")

if __name__ == "__main__":
    main() 