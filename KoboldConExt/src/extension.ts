
// Author: Majeh
/* eslint-disable @typescript-eslint/naming-convention */
import * as vscode from 'vscode';
import * as http from 'http';
import * as https from 'https';
import { URL } from 'url';


let projectContext: string = ""; // Stores project rules/context
let connSett: string | undefined;

// Build the request object with default parameters matching the AI portal
function buildRequestBody(prompt: string): any {
  return {
    n: 1,
    max_context_length: 131072,
    max_length: 4096,
    rep_pen: 1.05,
    temperature: 0.75,
    top_p: 0.92,
    top_k: 100,
    top_a: 0,
    typical: 1,
    tfs: 1,
    rep_pen_range: 360,
    rep_pen_slope: 0.7,
    sampler_order: [6, 0, 1, 3, 4, 2, 5],
    memory: "",
    trim_stop: true,
    genkey: "KCPP3265",
    min_p: 0,
    dynatemp_range: 0,
    dynatemp_exponent: 1,
    smoothing_factor: 0,
    smoothing_curve: 1,
    nsigma: 0,
    banned_tokens: [],
    render_special: false,
    logprobs: false,
    replace_instruct_placeholders: true,
    presence_penalty: 0,
    logit_bias: {},
    adaptive_target: -1,
    adaptive_decay: 0.9,
    stop_sequence: ["{{[INPUT]}}", "{{[OUTPUT]}}"],
    use_default_badwordsids: false,
    bypass_eos: false,
    prompt,
    stream: true
  };
}

// Set project context/rules (stored client-side, appended to all prompts)
export async function setProjectContext() {
  projectContext = await vscode.window.showInputBox({
    placeHolder: "Describe rules and context for your project (e.g., 'Modern React app with TypeScript, use functional components, follow ESLint rules')",
    prompt: "Project Rules and Context ...",
    value: projectContext
  }) || projectContext;

  if (projectContext) {
    vscode.window.showInformationMessage(`Project context set. Will be appended to all prompts.`);
  }
}

export async function addEditContext() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('No active text editor');
    return;
  }

  const lineNumber = editor.selection.start.line;
  const column = editor.selection.start.character;
  
  // Get the entire file content
  const fullContent = editor.document.getText();
  const lines = fullContent.split('\n');

  // Insert cursor marker at the exact position
  let markedContent = '';
  for (let i = 0; i < lines.length; i++) {
    markedContent += lines[i];
    if (i === lineNumber) {
      // Insert marker at cursor position on this line
      markedContent = markedContent.slice(0, markedContent.length - lines[i].length + column) + 
                      '<<<CURSOR_POSITION>>>' + 
                      markedContent.slice(markedContent.length - lines[i].length + column);
    }
    if (i < lines.length - 1) {
      markedContent += '\n';
    }
  }

  let editRequest = await vscode.window.showInputBox({
    placeHolder: "What should be generated/edited at the cursor position?",
    prompt: "Edit instruction ...",
    value: ""
  });

  if (editRequest === undefined) { return; }

  // Create context-aware prompt with complete file and cursor position
  const contextPrompt = `${projectContext ? 'Project Context: ' + projectContext + '\n\n' : ''}Here is the complete file with cursor position marked:\n\`\`\`\n${markedContent}\n\`\`\`\n\nInstructions: Generate code to be inserted at the <<<CURSOR_POSITION>>> marker. ${editRequest}\n\nProvide ONLY the code without explanations or backticks.`;

  // Send to API with streaming
  await sendToApiWithStreaming(contextPrompt, editor, lineNumber);
}


// Send prompt to API and stream results in real-time
async function sendToApiWithStreaming(prompt: string, editor: vscode.TextEditor, insertLineNumber: number) {
  try {
    // FIX: Target KoboldCPP's dedicated streaming endpoint
    const url = new URL(connSett + '/api/extra/generate/stream'); 
    const body = JSON.stringify(buildRequestBody(prompt));
    const lib = url.protocol === 'https:' ? https : http;
    const headers: any = {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(body)
    };
    
    const out = vscode.window.createOutputChannel('Kobold Connect');
    out.show(true);
    
    // Log user input
    out.appendLine('INPUT: ' + prompt);
    out.appendLine('---');

    let currentLine = insertLineNumber;
    let currentCol = editor.document.lineAt(currentLine).text.length;

    await new Promise<void>((resolve, reject) => {
      const req = lib.request(
        {
          hostname: url.hostname,
          port: url.port || (url.protocol === 'https:' ? 443 : 80),
          path: url.pathname + url.search,
          method: 'POST',
          headers
        },
        (res: any) => {
          if (res.statusCode && res.statusCode >= 400) {
            reject(new Error(`API error: ${res.statusCode} ${res.statusMessage || ''}`));
            return;
          }

          let buffer = '';
          let textQueue = '';
          let isEditing = false;
          let hasReceivedStream = false; // Track if we got actual stream chunks

          // Process the text queue sequentially to avoid overlapping VS Code edits
          const processQueue = async () => {
            if (isEditing || textQueue.length === 0) return;
            isEditing = true;

            let textToInsert = textQueue;
            textQueue = '';

            // Basic filter to strip out markdown backticks / language tags on the fly
            textToInsert = textToInsert.replace(/```[a-zA-Z0-9+#-]*\n?/g, '');

            if (textToInsert) {
              await editor.edit(editBuilder => {
                editBuilder.insert(new vscode.Position(currentLine, currentCol), textToInsert);
              });

              // Update cursor position tracking for the next insertion
              const lines = textToInsert.split('\n');
              if (lines.length > 1) {
                currentLine += lines.length - 1;
                currentCol = lines[lines.length - 1].length;
              } else {
                currentCol += textToInsert.length;
              }
            }

            isEditing = false;
            
            // If new tokens arrived while we were updating the editor, loop back
            if (textQueue.length > 0) {
              processQueue();
            }
          };

          // Accumulate and parse response data in real time
          res.on('data', (chunk: Buffer) => {
            buffer += chunk.toString();

            // Extract Server-Sent Event (SSE) blocks separated by double newlines
            let index;
            while ((index = buffer.indexOf('\n\n')) !== -1) {
              const eventBlock = buffer.slice(0, index);
              buffer = buffer.slice(index + 2);

              // Parse out the "data: {...}" JSON from the event block
              const dataMatch = eventBlock.match(/data:\s*(.*)/);
              if (dataMatch && dataMatch[1]) {
                hasReceivedStream = true; // Confirmed we are getting stream chunks
                const dataStr = dataMatch[1].trim();
                
                if (dataStr === '[DONE]') continue;

                try {
                  const data = JSON.parse(dataStr);
                  let token = '';

                  // Parse KoboldCPP stream format
                  if (data.token !== undefined) {
                    token = data.token;
                  } 
                  // Fallback support for OpenAI compatible formatting
                  else if (data.choices && data.choices[0]) {
                    if (data.choices[0].text !== undefined) {
                      token = data.choices[0].text;
                    } else if (data.choices[0].delta && data.choices[0].delta.content) {
                      token = data.choices[0].delta.content;
                    }
                  }

                  if (token) {
                    textQueue += token;
                    processQueue(); // Trigger the editor update live
                  }
                } catch (parseErr) {
                  // Incomplete chunks will fail to parse; safely ignore them until more data arrives
                }
              }
            }
          });

          // Process complete response
          res.on('end', async () => {
            // FALLBACK: If the API ignored the stream request and returned a single JSON block
            if (!hasReceivedStream && buffer.trim().startsWith('{')) {
              try {
                const data = JSON.parse(buffer);
                if (data.results && data.results[0] && data.results[0].text) {
                  textQueue += data.results[0].text;
                  await processQueue();
                }
              } catch (e) {
                out.appendLine('[ERROR] Failed to parse non-streamed fallback data.');
              }
            }

            // Allow a small delay for the editor to finish applying the final queued edits
            const finishInterval = setInterval(() => {
              if (!isEditing && textQueue.length === 0) {
                clearInterval(finishInterval);
                out.appendLine('\n[INFO] Generation complete.');
                resolve();
              }
            }, 50);
          });

          res.on('error', (err: Error) => {
            reject(err);
          });
        }
      );

      req.on('error', (err: Error) => {
        reject(err);
      });
      req.write(body);
      req.end();
    });
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to generate code: ${error}`);
  }
}
/*
// Send prompt to API and stream results in real-time
async function sendToApiWithStreaming(prompt: string, editor: vscode.TextEditor, insertLineNumber: number) {
  try {
    const url = new URL(connSett + '/api/v1/generate');
    const body = JSON.stringify(buildRequestBody(prompt));
    const lib = url.protocol === 'https:' ? https : http;
    const headers: any = {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(body)
    };

    const out = vscode.window.createOutputChannel('Kobold Connect');
    out.show(true);
    
    // Log user input
    out.appendLine('INPUT: ' + prompt);
    out.appendLine('---');

    // Position to insert at
    let currentLine = insertLineNumber;
    let currentCol = editor.document.lineAt(currentLine).text.length;

    await new Promise<void>((resolve, reject) => {
      const req = lib.request(
        {
          hostname: url.hostname,
          port: url.port || (url.protocol === 'https:' ? 443 : 80),
          path: url.pathname + url.search,
          method: 'POST',
          headers
        },
        (res: any) => {
          if (res.statusCode && res.statusCode >= 400) {
            reject(new Error(`API error: ${res.statusCode} ${res.statusMessage || ''}`));
            return;
          }

          let buffer = '';

          // Accumulate response data
          res.on('data', (chunk: Buffer) => {
            buffer += chunk.toString();
            out.appendLine(chunk.toString());
          });

          // Process complete response
          res.on('end', async () => {
            try {
              const data = JSON.parse(buffer);
              
              if (data.results && data.results[0] && data.results[0].text) {
                let fullText = data.results[0].text;

                // Strip code fence markers if present
                /*const cleanText = (() => {
                  const firstFence = fullText.indexOf('```');
                  if (firstFence !== -1) {
                    const secondFence = fullText.indexOf('```', firstFence + 3);
                    if (secondFence !== -1) {
                      return fullText.slice(firstFence + 3, secondFence).trim();
                    }
                  }
                  return fullText;
                })();
                const cleanText = (() => {
                const fences = [];
                let pos = 0;

                while (true) {
                  const start = fullText.indexOf('```', pos);
                  if (start === -1) {break;}
                  const end = fullText.indexOf('```', start + 3);
                  if (end === -1) {break;}

                  fences.push({ start, end });
                  pos = end + 3;
                }

                if (fences.length === 0) {return fullText;}

                // First block: keep as-is
                let output = fullText.slice(fences[0].start + 3, fences[0].end).trim();

                // Other blocks: comment them out
                for (let i = 1; i < fences.length; i++) {
                  const block = fullText.slice(fences[i].start + 3, fences[i].end).trim();
                  output += `\n\n/* Other block:\n${block}\n*`;
                }

                return output;
              })();
              const cleanText = (() => {
                const fences: { start: number; end: number }[] = [];
                let pos = 0;

                while (true) {
                  const start = fullText.indexOf('```', pos);
                  if (start === -1) {
                    break;
                  }
                  const end = fullText.indexOf('```', start + 3);
                  if (end === -1) {
                    break;
                  }

                  fences.push({ start, end });
                  pos = end + 3;
                }

                if (fences.length === 0) {
                  return fullText;
                }

                // First block: keep as-is
                let output = fullText.slice(fences[0].start + 3, fences[0].end).trim();

                // Other blocks: comment them out
                for (let i = 1; i < fences.length; i++) {
                  const block = fullText.slice(fences[i].start + 3, fences[i].end).trim();
                  output += `\n\n/* Other block:\n${block}\n`;
                }

                // Also preserve text outside fences (commented)
                let lastEnd = fences[fences.length - 1].end + 3;
                if (lastEnd < fullText.length) {
                  const trailing = fullText.slice(lastEnd).trim();
                  if (trailing) {
                    output += `\n\n/* Explanation:\n${trailing}\n`;
                  }
                }

                return output;
              })();*//*
                const cleanText = (() => {
                  const firstFence = fullText.indexOf("```");
                  if (firstFence === -1) {
                    return fullText;
                  }

                  const secondFence = fullText.indexOf("```", firstFence + 3);
                  if (secondFence === -1) {
                    return fullText;
                  }

                  // Extract the first fenced block
                  //let output = fullText.slice(firstFence + 3, secondFence).trim();
                  // Extract the first fenced block
                    let block = fullText.slice(firstFence + 3, secondFence).trim();

                    // Remove language hint if present (e.g., "cpp", "bash", "ts")
                    const firstLineBreak = block.indexOf("\n");
                    if (firstLineBreak !== -1) {
                      const firstLine = block.slice(0, firstLineBreak).trim();
                      if (/^[a-zA-Z0-9+#-]+$/.test(firstLine)) {
                        block = block.slice(firstLineBreak + 1).trim();
                      }
                    }

                    let output = block;

                  // Everything after the first block goes into one comment
                  const trailing = fullText.slice(secondFence + 3).trim();
                  if (trailing) {
                    output += `\n\n/*\n${trailing}\n*//*`;
                  }

                  return output;
                })();

                // Split by lines
                const lines = cleanText.split('\n');

                // Stream line by line
                for (let i = 0; i < lines.length; i++) {
                  const line = lines[i];
                  
                  // Insert line
                  await editor.edit(editBuilder => {
                    editBuilder.insert(new vscode.Position(currentLine, currentCol), line);
                  });
                  currentCol += line.length;
                  
                  // Add newline if not the last line
                  if (i < lines.length - 1) {
                    await editor.edit(editBuilder => {
                      editBuilder.insert(new vscode.Position(currentLine, currentCol), '\n');
                    });
                    currentLine += 1;
                    currentCol = 0;
                  }
                  
                  // Delay between lines
                  await new Promise(r => setTimeout(r, 50));
                }

                // Log output statistics
                out.appendLine('OUTPUT LENGTH: ' + cleanText.length + ' characters');
                if (data.tokens_generated) {
                  out.appendLine('OUTPUT TOKENS: ' + data.tokens_generated);
                }
                if (data.prompt_tokens) {
                  out.appendLine('INPUT TOKENS: ' + data.prompt_tokens);
                }
              } else {
                out.appendLine('[ERROR] No text in response');
              }
            } catch (parseErr: any) {
              out.appendLine('[ERROR] Failed to parse response: ' + parseErr.message);
            }
            resolve();
          });

          res.on('error', (err: Error) => {
            reject(err);
          });
        }
      );

      req.on('error', (err: Error) => {
        reject(err);
      });
      req.write(body);
      req.end();
    });
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to generate code: ${error}`);
  }
}
*/
// Legacy function kept for backwards compatibility
export async function grabTextAndSendToApi() {
  await setProjectContext();
}
export async function customPromptSnippetfn() {
  // Get the active text editor.
  const editor = vscode.window.activeTextEditor;
  // If there is no active text editor, return.
  if (!editor) {
    vscode.window.showErrorMessage('No active text editor');
    return;
  }

  // Get the current line number.
  const lineNumber = editor.selection.start.line;

  // Get user's prompt/question
  let snippetPrompt = await vscode.window.showInputBox({
    placeHolder: "Enter your question or code generation request",
    prompt: "Ask the AI ...",
    value: ""
  });

  // User cancelled the input
  if (snippetPrompt === undefined) {
    return;
  }

  // Append project context to the prompt if it exists
  const fullPrompt = projectContext 
    ? `Project Context: ${projectContext}\n\n${snippetPrompt}. \nProvide ONLY the code without explanations`
    : `${snippetPrompt}. \nProvide ONLY the code without explanations`;

  // Send with streaming
  await sendToApiWithStreaming(fullPrompt, editor, lineNumber);
}

export function activate(context: vscode.ExtensionContext) {
  // Initialize server URL from configuration (default: http://localhost:5001)
  const config = vscode.workspace.getConfiguration('koboldcpp');
  connSett = config.get<string>('serverUrl') || 'http://localhost:5001';

  // Listen for configuration changes
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(event => {
      if (event.affectsConfiguration('koboldcpp.serverUrl')) {
        const updatedConfig = vscode.workspace.getConfiguration('koboldcpp');
        connSett = updatedConfig.get<string>('serverUrl') || 'http://localhost:5001';
        vscode.window.showInformationMessage(`KoboldCPP server URL updated to: ${connSett}`);
      }
    })
  );

  // Register command for setting project context (KoboldInfo)
  const setContextCommand = 'koboldcpp.grabTextAndSendToApi';
  const setContextHandler = async () => {
    await setProjectContext();
  };
  context.subscriptions.push(vscode.commands.registerCommand(setContextCommand, setContextHandler));

  // Register command for editing code with context (KoboldEdit)
  const addContextCommand = 'koboldcpp.AddContext';
  const addContextHandler = async () => {
    await addEditContext();
  };
  context.subscriptions.push(vscode.commands.registerCommand(addContextCommand, addContextHandler));

  // Register command for asking AI a question (KoboldAsk)
  const customPromptCommand = 'koboldcpp.CustomPrompt';
  const customPromptHandler = async () => {
    await customPromptSnippetfn();
  };
  context.subscriptions.push(vscode.commands.registerCommand(customPromptCommand, customPromptHandler));
  // Register command for opening KoboldCPP sidebar
const openSidebarCommand = 'koboldcpp.OpenSidebar';
const openSidebarHandler = async () => {
  await openKoboldSidebar(context);
};
context.subscriptions.push(
  vscode.commands.registerCommand(openSidebarCommand, openSidebarHandler)
);
}

export function deactivate() {}
export async function openKoboldSidebar(context: vscode.ExtensionContext) {
  // Get server URL from configuration (default: http://localhost:5001)
  const config = vscode.workspace.getConfiguration('koboldcpp');
  const serverUrl = config.get<string>('serverUrl') || 'http://localhost:5001';

  // Create a new Webview panel
  const panel = vscode.window.createWebviewPanel(
    'koboldcppSidebar',          // internal identifier
    'KoboldCPP',                 // title shown in the sidebar
    vscode.ViewColumn.Beside,    // open beside the editor
    {
      enableScripts: true,       // allow JS inside the webview
    }
  );

  // Load the KoboldCPP server URL inside the webview
  panel.webview.html = `
    <!DOCTYPE html>
    <html lang="en">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>KoboldCPP</title>
      <style>
        body, html {
          margin: 0;
          padding: 0;
          height: 100%;
          overflow: hidden;
        }
        iframe {
          border: none;
          width: 100%;
          height: 100%;
        }
      </style>
    </head>
    <body>
      <iframe src="${serverUrl}"></iframe>
    </body>
    </html>
  `;
}