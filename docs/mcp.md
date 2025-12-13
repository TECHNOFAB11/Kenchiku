# Model Context Protocol (MCP) Server

Kenchiku includes a built-in MCP server that allows AI agents (like Claude Desktop or IDE assistants) to interact with
your scaffolds.
This enables agents to discover available project templates and construct them.

!!! warning

    The MCP server is still under development.

## Capabilities

The Kenchiku MCP server currently supports the following tools:

### `list`

Lists all available scaffolds discovered by Kenchiku.

- **Input**: None
- **Output**: A formatted string listing all scaffolds, their descriptions, and available patches.

### `read`

Reads the `scaffold.lua` definition of a specific scaffold.
This is useful for an agent to inspect the logic, required values, and structure of a scaffold before recommending it or
helping you use it.

- **Input**:
    - `scaffold_name` (string): The name of the scaffold to read.
- **Output**: The raw content of the `scaffold.lua` file.

## Usage

To use the MCP server, configure your MCP client to run the `kenchiku mcp` command.

### Example Configuration (Claude Desktop)

Add the following to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "kenchiku": {
      "command": "kenchiku",
      "args": ["mcp"]
    }
  }
}
```

Ensure `kenchiku` is in your `PATH`, or provide the absolute path to the executable.
