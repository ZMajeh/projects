## Project Overview

This is a Visual Studio Code extension that connects to a KoboldCPP server to provide AI-powered code completion and generation. It allows users to interact with their Kobold AI instance directly within the editor.

**Main Technologies:**

*   **TypeScript:** The primary language used for the extension's source code.
*   **VS Code API:** Used to create commands, manage editor content, and interact with the VS Code UI.
*   **Node.js:** The runtime environment for the extension.
*   **Webpack:** Used to bundle and package the extension for distribution.

**Architecture:**

The extension works by making HTTP requests to a KoboldCPP server. It sends prompts to the server's API and receives a streaming response containing the generated text. This text is then inserted into the active text editor in real-time.

The extension provides several commands for different types of interactions:

*   **Setting project context:** Users can define a set of rules and context for their project, which will be appended to all prompts.
*   **Context-aware code generation:** The extension can send the entire content of the current file, along with the cursor's position, to the AI to generate code that is aware of the surrounding context.
*   **Custom prompts:** Users can enter a custom prompt to ask the AI to generate a specific code snippet.
*   **Sidebar:** The extension can open the KoboldCPP server's web interface in a sidebar for easy access.

## Building and Running

**Building the extension:**

```bash
npm run compile
```

**Running the extension in a development environment:**

1.  Open the project in Visual Studio Code.
2.  Press `F5` to open a new window with the extension loaded.

**Packaging the extension for distribution:**

```bash
npm run package
```

**Running tests:**

```bash
npm test
```

## Development Conventions

*   The code is written in TypeScript and follows the coding style enforced by ESLint.
*   The extension uses a streaming API to provide a real-time user experience.
*   The extension is configured through the `package.json` file, which defines the commands, views, and configuration properties.
*   The extension's dependencies are managed using npm.
*   The code is bundled using Webpack.
