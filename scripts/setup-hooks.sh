#!/bin/bash
# Points git to use scripts/ as the hooks directory
set -e
git config core.hooksPath scripts
echo "Git hooks installed (core.hooksPath = scripts)."
