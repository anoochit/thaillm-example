# Model Context Protocol (MCP) Integration

This bot now supports loading external tools via MCP. To use it:

1. Create an `mcp.json` file in the root directory.
2. Define your MCP servers in the following format:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/tmp"
      ]
    },
    "everything": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-everything"
      ]
    }
  }
}
```

1. Restart the bot. It will automatically detect the file, start the servers, and register the tools with the agent.

## Example File

An example configuration is provided in `mcp.json.example`.
