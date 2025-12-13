{inputs, ...}: let
  inherit (inputs) doclib;
in
  (doclib.mkDocs {
    docs."default" = {
      base = "${inputs.self}";
      path = "${inputs.self}/docs";
      material = {
        enable = true;
        colors = {
          primary = "black";
          accent = "yellow";
        };
        umami = {
          enable = true;
          src = "https://analytics.tf/umami";
          siteId = "09615a72-430f-4843-aee6-12ac6d5d60f3";
          domains = ["kenchiku.projects.tf"];
        };
      };
      config = {
        site_name = "Kenchiku";
        site_url = "https://kenchiku.projects.tf";
        repo_name = "TECHNOFAB/kenchiku";
        repo_url = "https://gitlab.com/TECHNOFAB/kenchiku";
        extra_css = ["style.css"];
        theme = {
          logo = "images/logo.svg";
          icon.repo = "simple/gitlab";
          favicon = "images/logo.svg";
        };
        nav = [
          {"Introduction" = "index.md";}
          {"Usage" = "usage.md";}
          {"CLI" = "cli.md";}
          {"Scaffolds" = "scaffolds.md";}
          {"Lua APIs" = "apis.md";}
          {"Examples" = "examples.md";}
          {"MCP" = "mcp.md";}
        ];
        markdown_extensions = [
          {
            "pymdownx.highlight".pygments_lang_class = true;
          }
          "pymdownx.inlinehilite"
          "pymdownx.snippets"
          "pymdownx.superfences"
          "pymdownx.escapeall"
          "fenced_code"
          "admonition"
        ];
      };
    };
  }).packages
