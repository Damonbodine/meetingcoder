import * as vscode from 'vscode';
import WebSocket from 'ws';

let socket: WebSocket | null = null;
let reconnectInterval: NodeJS.Timeout | null = null;

export function activate(context: vscode.ExtensionContext) {
    console.log('MeetingCoder extension is active');

    // Attempt connection immediately
    connectToMeetingCoder();

    // Register command to manually connect
    let disposable = vscode.commands.registerCommand('meetingcoder.connect', () => {
        connectToMeetingCoder();
    });

    context.subscriptions.push(disposable);

    // Track active text editor changes
    vscode.window.onDidChangeActiveTextEditor(editor => {
        if (editor && socket && socket.readyState === WebSocket.OPEN) {
            sendContext(editor);
        }
    });

    // Track document changes (debounced)
    vscode.workspace.onDidChangeTextDocument(event => {
        if (vscode.window.activeTextEditor &&
            event.document === vscode.window.activeTextEditor.document &&
            socket && socket.readyState === WebSocket.OPEN) {
            // In a real app, debounce this
            sendContext(vscode.window.activeTextEditor);
        }
    });
}

function connectToMeetingCoder() {
    if (socket) {
        socket.close();
    }

    // Connect to local WebSocket server hosted by Tauri app
    socket = new WebSocket('ws://127.0.0.1:3000/vscode');

    socket.on('open', () => {
        console.log('Connected to MeetingCoder');
        vscode.window.setStatusBarMessage('MeetingCoder: Connected', 3000);
        if (vscode.window.activeTextEditor) {
            sendContext(vscode.window.activeTextEditor);
        }
    });

    socket.on('close', () => {
        console.log('Disconnected from MeetingCoder');
        // Try to reconnect every 5 seconds
        if (!reconnectInterval) {
            reconnectInterval = setInterval(connectToMeetingCoder, 5000);
        }
    });

    socket.on('error', (err) => {
        console.error('MeetingCoder connection error:', err);
    });

    socket.on('message', (data) => {
        handleMessage(data.toString());
    });
}

function sendContext(editor: vscode.TextEditor) {
    const msg = {
        type: 'context_update',
        path: editor.document.uri.fsPath,
        content: editor.document.getText(), // Be careful with large files
        cursor: editor.selection.active,
        language: editor.document.languageId
    };
    socket?.send(JSON.stringify(msg));
}

function handleMessage(message: string) {
    try {
        const msg = JSON.parse(message);
        if (msg.type === 'open_file') {
            vscode.workspace.openTextDocument(msg.path).then(doc => {
                vscode.window.showTextDocument(doc);
            });
        }
        // Handle other agent actions (insert code, highlight, etc.)
    } catch (e) {
        console.error('Failed to parse message:', e);
    }
}

export function deactivate() {
    if (socket) {
        socket.close();
    }
    if (reconnectInterval) {
        clearInterval(reconnectInterval);
    }
}
