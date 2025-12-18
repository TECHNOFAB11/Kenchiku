# Kenchiku

[![pipeline status](https://gitlab.com/TECHNOFAB/kenchiku/badges/main/pipeline.svg)](https://gitlab.com/TECHNOFAB/kenchiku/-/commits/main)
![License: MIT](https://img.shields.io/gitlab/license/technofab/kenchiku)
[![Latest Release](https://gitlab.com/TECHNOFAB/kenchiku/-/badges/release.svg)](https://gitlab.com/TECHNOFAB/kenchiku/-/releases)
[![Support me](https://img.shields.io/badge/Support-me-yellow)](https://tec.tf/#support)
[![Docs](https://img.shields.io/badge/Read-Docs-yellow)](https://kenchiku.projects.tf)

Kenchiku is a powerful, extensible scaffolding tool designed to streamline project creation and modification.
It leverages Lua for flexible scaffold definitions and includes a MCP server for seamless integration with AI agents.

## Features

- **Lua-Powered Scaffolds**: Define complex scaffolding logic, file generation, and patching using the full power of Lua.
- **CLI Interface**: Easy-to-use command-line interface for managing and using scaffolds.
- **MCP Server**: Built-in MCP server allows AI assistants (like Claude or IDE agents) to discover, read, and use scaffolds directly.
- **Patching System**: Not just for new projects, Kenchiku can also apply patches to existing codebases, allowing for granular updates and feature additions.
- **Interactive Prompts**: Scaffolds can define required values, which Kenchiku will interactively prompt the user for (with support for text, booleans, and enums). Also supports pre-defined values with `--set`.

## Quickstart

1. Install using Nix or Cargo:

    ```sh
    nix profile install gitlab:TECHNOFAB/kenchiku
    # or
    cargo install --git https://gitlab.com/TECHNOFAB/kenchiku
    ```

1. Choose any directory for your scaffolds and set the env var `KENCHIKU_PATH` to it

1. Create a scaffold inside this directory by creating a subdirectory with the scaffold name
    and add a `scaffold.lua` file, see the [docs](https://kenchiku.projects.tf) for more details here.

1. Run `kenchiku construct <name of scaffold>`

For more information, check the [docs](https://kenchiku.projects.tf).
