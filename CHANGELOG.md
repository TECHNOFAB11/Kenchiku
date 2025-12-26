# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [v0.2.0](https://gitlab.com/TECHNOFAB/kenchiku/compare/2abe09ec0f5978e96a5ea69943453dc071f7dcf5..v0.2.0) - 2025-12-26
#### Features
- (**lua**) add fs.copy function - ([d5e24f2](https://gitlab.com/TECHNOFAB/kenchiku/commit/d5e24f229880053cb884df30f1b7f88875dbb8c6)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- load default values from env vars (with KENCHIKU_VAL_ prefix) - ([0fa5c30](https://gitlab.com/TECHNOFAB/kenchiku/commit/0fa5c309dd6637fbc247d8b9e95a72fd2794e272)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add --json param to list and show commands - ([0f8a841](https://gitlab.com/TECHNOFAB/kenchiku/commit/0f8a8414e7af8b78018efa49f1e06a25399b916a)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
#### Bug Fixes
- (**lua**) normalize paths instead of validating them, always succeeding - ([4b12345](https://gitlab.com/TECHNOFAB/kenchiku/commit/4b12345732bbd2069fe4d6d7192463e63154d538)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
#### Refactoring
- DRY mcp server a bit - ([30d5b03](https://gitlab.com/TECHNOFAB/kenchiku/commit/30d5b033df1352e6b4381c9b65acb50ae4b45a59)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
#### Miscellaneous Chores
- (**cocogitto**) fix author linking in changelog - ([2abe09e](https://gitlab.com/TECHNOFAB/kenchiku/commit/2abe09ec0f5978e96a5ea69943453dc071f7dcf5)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**deps**) update rust crate tempfile to v3.24.0 - ([f065be7](https://gitlab.com/TECHNOFAB/kenchiku/commit/f065be741bcd15fed79cbe8750646f926b31ddda)) - Renovate Bot
- (**deps**) update rust crate serde_json to v1.0.147 - ([4759663](https://gitlab.com/TECHNOFAB/kenchiku/commit/4759663924ecd526e95580d7bf0d4425c3259f6a)) - Renovate Bot
- (**deps**) update rust crate serde_json to v1.0.146 - ([43416a3](https://gitlab.com/TECHNOFAB/kenchiku/commit/43416a31c928f5757b2249bf19f44447d21a9ead)) - Renovate Bot
- improve shell completions a bit - ([570bc91](https://gitlab.com/TECHNOFAB/kenchiku/commit/570bc91d5463320d1af77c1f8416f8dd088ef1bf)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add shell completions - ([6766560](https://gitlab.com/TECHNOFAB/kenchiku/commit/676656050fed363e11739f5aa3403530a6a3aea9)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)

- - -

## [v0.1.0](https://gitlab.com/TECHNOFAB/kenchiku/compare/d0200014df74f03038b0e2f3be3b13608f8b1db2..v0.1.0) - 2025-12-20
#### Features
- (**cli**) add subcommands like construct and patch - ([44d2f18](https://gitlab.com/TECHNOFAB/kenchiku/commit/44d2f18425f79a25020ab727902565abe6f5bbf5)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- ![BREAKING](https://img.shields.io/badge/BREAKING-red) (**lua**) return table from exec.run containing stdout, stderr etc. - ([596f91b](https://gitlab.com/TECHNOFAB/kenchiku/commit/596f91b9c72114b3d88dc1298d299664f4871d8a)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**lua**) add template and template_file functions using minijinja - ([e0d6696](https://gitlab.com/TECHNOFAB/kenchiku/commit/e0d66968b82deeb14aca385cd85dbceb6c8aff28)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**lua**) add json module - ([186c9be](https://gitlab.com/TECHNOFAB/kenchiku/commit/186c9be1059d45fd926d35bf16f6ea72cb74f673)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**lua**) add tmpl module - ([0fa57a5](https://gitlab.com/TECHNOFAB/kenchiku/commit/0fa57a58d81886093642d7ef1ce9a378ec39744c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**lua**) allow fs.read to choose between workdir and scaffold dir - ([d8f086d](https://gitlab.com/TECHNOFAB/kenchiku/commit/d8f086daa54786f041374e7f78b0fd6cd40e642d)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**lua**) extend fs module - ([9c3bb44](https://gitlab.com/TECHNOFAB/kenchiku/commit/9c3bb4497f28e4b458cb1664986313b3e08ee692)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**mcp**) implement patch tool aswell - ([14b4383](https://gitlab.com/TECHNOFAB/kenchiku/commit/14b438392bd9a66664ff66ea62497dd67be99b68)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**mcp**) implement mcp construction with session handling - ([ac1d036](https://gitlab.com/TECHNOFAB/kenchiku/commit/ac1d03611ab155d19f75f6f42f0530ea2ec3c7c4)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**mcp**) improve list tool, add show tool, add descriptions to fields - ([325b621](https://gitlab.com/TECHNOFAB/kenchiku/commit/325b621c216c577a6e575ec53bed61634759814e)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**mcp**) add read tool and add tests for read+list tools - ([0e278dd](https://gitlab.com/TECHNOFAB/kenchiku/commit/0e278ddc1371929434234e5d31d9bb1087992ac9)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**scaffold/lua**) implement require functiom - ([d118f26](https://gitlab.com/TECHNOFAB/kenchiku/commit/d118f268f678423f2c0146bdc8685df8c7f69027)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**template**) add extra functions and filters for minijinja templates - ([a9601b8](https://gitlab.com/TECHNOFAB/kenchiku/commit/a9601b8b2e67dcc4ae04dd6a712e9c9ee1b0cd8f)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- improve default value handling - ([458dbb3](https://gitlab.com/TECHNOFAB/kenchiku/commit/458dbb34330f6c27cfb8f45a404977810fa7e25c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- implement basic value handling - ([7b88724](https://gitlab.com/TECHNOFAB/kenchiku/commit/7b88724249385b6fe8fc736dda0a2e0431391ce5)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add initial support for values - ([5160e0a](https://gitlab.com/TECHNOFAB/kenchiku/commit/5160e0a818f3158b732b342aed5085bfdf3dc40d)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add initial mcp server - ([94e427c](https://gitlab.com/TECHNOFAB/kenchiku/commit/94e427cbe030632a1de9b5cbed4ea06b3715ed38)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- implement copying over the files from the tmpdir in construct - ([9e559fc](https://gitlab.com/TECHNOFAB/kenchiku/commit/9e559fcbc7578ddab517ed22cd777938caec6fc7)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- pass around context with info like working dir etc. - ([d805b45](https://gitlab.com/TECHNOFAB/kenchiku/commit/d805b459d155e75c9068c5900e054614b6734f27)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- only register lua functions when running construct or patch - ([9d19d40](https://gitlab.com/TECHNOFAB/kenchiku/commit/9d19d406cdcc7be01280844a52d7b749d9db36d9)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add scaffold discovery to list all found scaffolds - ([49b1dbc](https://gitlab.com/TECHNOFAB/kenchiku/commit/49b1dbc45396b91bf6f1cdf68944240b8a48ae0b)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add scaffold discovery using KENCHIKU_PATH - ([e470240](https://gitlab.com/TECHNOFAB/kenchiku/commit/e470240f28e54f0f6e808087f76ab34223bae468)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- define and load full scaffold schema - ([557b3b0](https://gitlab.com/TECHNOFAB/kenchiku/commit/557b3b09cb77db53f7337cd4b9834ae53c4ba42d)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add lua crate and some globals - ([61a63b4](https://gitlab.com/TECHNOFAB/kenchiku/commit/61a63b40c73e57e4fff52e2e6d29b82c896db32d)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add super basic scaffold loading - ([8deb733](https://gitlab.com/TECHNOFAB/kenchiku/commit/8deb733d5551d99d8e7edd1243c55049825b907d)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
#### Bug Fixes
- (**ci,devshell**) set LD_LIBRARY_PATH - ([1a64e78](https://gitlab.com/TECHNOFAB/kenchiku/commit/1a64e783860a8bbb1c78c682cb8ed07761381b07)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**deps**) update rust crate rmcp to 0.12.0 - ([ced893f](https://gitlab.com/TECHNOFAB/kenchiku/commit/ced893f4cd8c527da3996b37def632932a05f459)) - Renovate Bot
- (**deps**) update rust crate rmcp to 0.11.0 - ([d139cf0](https://gitlab.com/TECHNOFAB/kenchiku/commit/d139cf0f6d2c78d67dc7a934c21227156c97ea91)) - Renovate Bot
- (**package**) set LD_LIBRARY_PATH - ([888e98a](https://gitlab.com/TECHNOFAB/kenchiku/commit/888e98a5379b7968148f808acbce77b86aafaf0c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**schema**) update exec.run return type - ([9e497c5](https://gitlab.com/TECHNOFAB/kenchiku/commit/9e497c56f6b4062b2d71008b59685774072b8e34)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**tests**) run mcp tests sequentially, otherwise they fail with Nix - ([e3f707b](https://gitlab.com/TECHNOFAB/kenchiku/commit/e3f707bd2448f239fd7a4f0eda00c4da62156db8)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- enable lua sandbox, correct file name and exec.run cwd - ([14cdc7c](https://gitlab.com/TECHNOFAB/kenchiku/commit/14cdc7c1620b2a4dbcd4e60f114f78454c6ca962)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
#### Documentation
- extend mcp docs, add new tools etc. - ([be3327b](https://gitlab.com/TECHNOFAB/kenchiku/commit/be3327b4aeddf6614d3cd67ac561cd8c3b6c10db)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- document template extras and confirmation level - ([04ebe71](https://gitlab.com/TECHNOFAB/kenchiku/commit/04ebe7157e715974c78c9f9d5069cb1394bc34ff)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- minor improvements - ([2f0b4e1](https://gitlab.com/TECHNOFAB/kenchiku/commit/2f0b4e1da9d2be5286347fe528440ae07059745e)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- document luals setup with soonix - ([b0aa88f](https://gitlab.com/TECHNOFAB/kenchiku/commit/b0aa88ffa4330fd17b8ca8e20304b86a009818ab)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- write basic README, add LICENSE - ([d3618d8](https://gitlab.com/TECHNOFAB/kenchiku/commit/d3618d8bdc157d72bb1409c01d06c8b9b106c29c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- improve and extend docs - ([a76e4c2](https://gitlab.com/TECHNOFAB/kenchiku/commit/a76e4c2935ee1956ca335eb1f771fab36cfc6a8c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- write initial docs and a bit of vision - ([0bef293](https://gitlab.com/TECHNOFAB/kenchiku/commit/0bef293c7f3635fe194d9956be5b11ce066ea26c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add initial docs setup - ([2ff46ab](https://gitlab.com/TECHNOFAB/kenchiku/commit/2ff46ab75f96d8c38e7cccb347cf43abd68070fc)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
#### Tests
- (**lua**) add tests for exec module - ([9c2b51e](https://gitlab.com/TECHNOFAB/kenchiku/commit/9c2b51eeb8b3379e4358bb115d46ea8b3e09e73a)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**lua/values**) add tests for LuaValues, add trait for eyre<->lua errors - ([261f6c4](https://gitlab.com/TECHNOFAB/kenchiku/commit/261f6c4e73097e9530a547745ff0862b3f542419)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**values**) remove tests for removed default value handling - ([91bd634](https://gitlab.com/TECHNOFAB/kenchiku/commit/91bd6346506acb7a05eff88e6567e18257167ea9)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- fix discovery tests - ([1c1e214](https://gitlab.com/TECHNOFAB/kenchiku/commit/1c1e214997321d48bb7aaa01ee9843e90975f0ac)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
#### Continuous Integration
- add cache to test job - ([49c2d0c](https://gitlab.com/TECHNOFAB/kenchiku/commit/49c2d0c9926fdd43d0960640f232162230e5b079)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- update nix-gitlab-ci to 3.1.1 - ([cdaa173](https://gitlab.com/TECHNOFAB/kenchiku/commit/cdaa1731e456bdf0b15b03057f341f5d93ce3526)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
#### Miscellaneous Chores
- (**cli**) add "force" parameter which allows construct to overwrite files - ([2d9306b](https://gitlab.com/TECHNOFAB/kenchiku/commit/2d9306b997e1626d2ff49781e277612ec711dbba)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**cocogitto**) configure bump hooks - ([ccd5dd2](https://gitlab.com/TECHNOFAB/kenchiku/commit/ccd5dd2ebf5429e5441a069f438689100985727f)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**deps**) update rust crate tracing to v0.1.44 - ([cb608df](https://gitlab.com/TECHNOFAB/kenchiku/commit/cb608df6f0f865acc481a71f844c18f9f78dba1c)) - Renovate Bot
- (**lua**) handle lua print function ourselves for later mcp integration - ([eadcc5b](https://gitlab.com/TECHNOFAB/kenchiku/commit/eadcc5be82984650204f708d2132a7f90b5483c0)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- (**version**) v0.1.0 - ([67e3046](https://gitlab.com/TECHNOFAB/kenchiku/commit/67e30468ebfe2bb8572f264d3f59c44936520249)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- update flake and add cocogitto config - ([0f452cb](https://gitlab.com/TECHNOFAB/kenchiku/commit/0f452cb3c9ed21af9f0b2479c86527686ea41cf0)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add devtools and update nix-gitlab-ci - ([0f81d17](https://gitlab.com/TECHNOFAB/kenchiku/commit/0f81d17ab6ebf18cd6d3054f730808dd80df773c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add schema.lua to nix package, fix typo - ([47c207d](https://gitlab.com/TECHNOFAB/kenchiku/commit/47c207d491d4f0dc568cc04196db5e525f14bd1b)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add renovate, move soonix into own file - ([92c1547](https://gitlab.com/TECHNOFAB/kenchiku/commit/92c1547f19f27bb2a261e6b34903b33c3836ae0c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- format files - ([cd2b713](https://gitlab.com/TECHNOFAB/kenchiku/commit/cd2b713143cddd7be884039a9a538509eec4f061)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add editorconfig for lua - ([7af3401](https://gitlab.com/TECHNOFAB/kenchiku/commit/7af3401ff8d31b5576e66f420dd4b627ee011967)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- move construct call into its own function - ([fe97b30](https://gitlab.com/TECHNOFAB/kenchiku/commit/fe97b305112c8d9d8822b571a1e223c5da30463c)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- add CI - ([21824f0](https://gitlab.com/TECHNOFAB/kenchiku/commit/21824f0cb909f2c9c5125e53b9276054c62e46e9)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)
- initial commit - ([d020001](https://gitlab.com/TECHNOFAB/kenchiku/commit/d0200014df74f03038b0e2f3be3b13608f8b1db2)) - [@TECHNOFAB](https://gitlab.com/TECHNOFAB)

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).