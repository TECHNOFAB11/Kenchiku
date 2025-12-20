# Model Context Protocol (MCP) Server

Kenchiku includes a built-in MCP server that allows AI agents (like Claude Desktop or IDE assistants) to interact with your scaffolds.
This enables agents to discover available project templates, inspect their requirements, and construct them interactively.

## Capabilities

The Kenchiku MCP server currently supports the following tools:

### `list`

Lists all available scaffolds and patches discovered by Kenchiku.

- **Input**: None
- **Output**: A formatted string listing all scaffolds, their descriptions, and available patches.

### `show`

Shows details and required values for a specific scaffold or patch. Always use this tool before constructing or patching to understand what values are needed.

- **Input**:
    - `name` (string): The name of the scaffold (e.g., `my-scaffold`) or patch (e.g., `my-scaffold:my-patch`) to show.
- **Output**: Detailed information about the scaffold/patch, including a list of required values, their types, descriptions, and default values.

### `construct`

Initiates the construction of a new project using a scaffold.

- **Input**:
    - `scaffold_name` (string): The name of the scaffold to construct.
    - `values` (dictionary, optional): A dictionary of values to pass to the scaffold.
    - `output` (string, optional): The path where the scaffold will be generated. Defaults to the current directory.
- **Output**: A success message, or a request for missing values (see [Interactive Sessions](#interactive-sessions)).

### `patch`

Initiates the execution of a patch on an existing project.

- **Input**:
    - `name` (string): The name of the patch to run, in the format `<scaffold>:<patch>`.
    - `values` (dictionary, optional): A dictionary of values to pass to the patch.
    - `output` (string, optional): The path where the patch will run. Defaults to the current directory.
- **Output**: A success message, or a request for missing values (see [Interactive Sessions](#interactive-sessions)).

### `provide_values`

Supplies values to the current active session (construction or patch) when the server requests missing values.

- **Input**:
    - `values` (dictionary): A dictionary of values to provide.
- **Output**: A success message if the operation completes, or another request for missing values if more are needed.

### `cancel_session`

Cancels the current active session.

- **Input**: None
- **Output**: A confirmation message that the session has been cancelled.

## Interactive Sessions

Kenchiku supports interactive sessions for `construct` and `patch` operations. If you do not provide all required values upfront, the server will pause execution and request the missing values.

1. **Start**: You call `construct` or `patch`.
1. **Pause**: If a required value is missing, the tool returns a message indicating which value is missing, its description, and type.
1. **Provide**: You call `provide_values` with the missing value.
1. **Resume**: The server resumes execution. Steps 2-3 repeat until all values are provided.
1. **Finish**: Once all values are available, the operation completes and returns the final result.

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
