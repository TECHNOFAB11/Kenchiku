# Kenchiku - Project Scaffolding

Project scaffolding tool written in Rust, which uses Lua to define "Scaffolds" and
"Patches". Scaffolds construct new projects, mostly from zero, while Patches allow
modifying existing projects, like patching files to add features.

> **Fun Fact**: "Kenchiku" (建築) is Japanese for "architecture" or "construction".

## Features

- Flexible, thanks to Lua
- Integrated MCP server for LLMs to use Kenchiku (for easy project bootstrapping)
- Secure-ish (some sandboxing/escape preventions and confirmation prompts, but ultimately you should read
    the scaffolds and don't execute random stuff from the internet, like always)
