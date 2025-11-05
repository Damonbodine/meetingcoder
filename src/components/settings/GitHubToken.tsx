import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { GitHubConnectionTest } from "../../lib/types";

export const GitHubToken: React.FC<{
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}> = ({ descriptionMode = "tooltip", grouped = false }) => {
  const [token, setToken] = useState("");
  const [testResult, setTestResult] = useState<GitHubConnectionTest | null>(null);
  const [isTesting, setIsTesting] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [showToken, setShowToken] = useState(false);

  const handleSaveToken = async () => {
    console.log("handleSaveToken called, token length:", token.length);
    if (!token) {
      console.log("No token provided");
      return;
    }

    setIsSaving(true);
    try {
      console.log("Invoking set_github_token...");
      await invoke("set_github_token", { token });
      console.log("Token saved successfully");
      setToken("");
      setTestResult(null);
      alert("GitHub token saved successfully!");
    } catch (error) {
      console.error("Failed to save token:", error);
      alert(`Failed to save token: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    console.log("handleTestConnection called");
    setIsTesting(true);
    try {
      console.log("Invoking test_github_connection...");
      const result = await invoke<GitHubConnectionTest>("test_github_connection");
      console.log("Test result:", result);
      setTestResult(result);
    } catch (error) {
      console.error("Failed to test connection:", error);
      setTestResult({
        success: false,
        username: null,
        error: String(error),
      });
    } finally {
      setIsTesting(false);
    }
  };

  const handleRemoveToken = async () => {
    if (!confirm("Are you sure you want to remove your GitHub token?")) return;

    try {
      await invoke("remove_github_token");
      setTestResult(null);
      alert("GitHub token removed successfully!");
    } catch (error) {
      console.error("Failed to remove token:", error);
      alert(`Failed to remove token: ${error}`);
    }
  };

  return (
    <div className={`space-y-3 ${grouped ? "py-3" : "py-4"}`}>
      <div>
        <label className="block text-sm font-medium mb-2">
          GitHub Personal Access Token
        </label>
        <div className="flex gap-2">
          <div className="relative flex-1">
            <input
              type={showToken ? "text" : "password"}
              value={token}
              onChange={(e) => setToken(e.target.value)}
              placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
              className="w-full px-3 py-2 border rounded-md font-mono text-sm"
            />
            <button
              type="button"
              onClick={() => setShowToken(!showToken)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-gray-500 hover:text-gray-700"
            >
              {showToken ? "Hide" : "Show"}
            </button>
          </div>
          <button
            onClick={(e) => {
              e.preventDefault();
              console.log("Save button clicked");
              handleSaveToken();
            }}
            disabled={!token || isSaving}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-sm"
          >
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
        <p className="text-xs text-gray-500 mt-1">
          Create a token at{" "}
          <a
            href="https://github.com/settings/tokens/new?scopes=repo"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-600 hover:underline"
          >
            GitHub Settings
          </a>{" "}
          with <code className="bg-gray-100 px-1 rounded">repo</code> scope
        </p>
      </div>

      <div className="flex gap-2">
        <button
          onClick={(e) => {
            e.preventDefault();
            console.log("Test button clicked");
            handleTestConnection();
          }}
          disabled={isTesting}
          className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-sm"
        >
          {isTesting ? "Testing..." : "Test Connection"}
        </button>
        <button
          onClick={handleRemoveToken}
          className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 text-sm"
        >
          Remove Token
        </button>
      </div>

      {testResult && (
        <div
          className={`p-3 rounded-md text-sm ${
            testResult.success
              ? "bg-green-50 text-green-800 border border-green-200"
              : "bg-red-50 text-red-800 border border-red-200"
          }`}
        >
          {testResult.success ? (
            <>
              <strong>Connected successfully!</strong>
              {testResult.username && <> as @{testResult.username}</>}
            </>
          ) : (
            <>
              <strong>Connection failed:</strong> {testResult.error}
            </>
          )}
        </div>
      )}
    </div>
  );
};
