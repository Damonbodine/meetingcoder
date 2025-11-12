import React from "react";
import { useSettings } from "../../hooks/useSettings";

export const GitHubRepo: React.FC<{
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}> = ({ descriptionMode = "tooltip", grouped = false }) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const owner = getSetting("github_repo_owner") ?? "";
  const name = getSetting("github_repo_name") ?? "";

  return (
    <div className={`space-y-3 ${grouped ? "py-3" : "py-4"}`}>
      <div>
        <label className="block text-sm font-medium mb-2">
          Repository Owner
        </label>
        <input
          type="text"
          value={owner || ""}
          onChange={(e) =>
            updateSetting(
              "github_repo_owner",
              e.target.value || null
            )
          }
          placeholder="username or organization"
          className="w-full px-3 py-2 border rounded-md text-sm"
          disabled={isUpdating("github_repo_owner")}
        />
        <p className="text-xs text-gray-500 mt-1">
          The GitHub username or organization that owns the repository
        </p>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">
          Repository Name
        </label>
        <input
          type="text"
          value={name || ""}
          onChange={(e) =>
            updateSetting(
              "github_repo_name",
              e.target.value || null
            )
          }
          placeholder="repository-name"
          className="w-full px-3 py-2 border rounded-md text-sm"
          disabled={isUpdating("github_repo_name")}
        />
        <p className="text-xs text-gray-500 mt-1">
          The name of the repository where updates will be pushed
        </p>
      </div>

      {owner && name && (
        <div className="p-2 bg-gray-50 rounded-md text-sm">
          <span className="text-gray-600">Repository:</span>{" "}
          <a
            href={`https://github.com/${owner}/${name}`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-600 hover:underline font-mono"
          >
            {owner}/{name}
          </a>
        </div>
      )}
    </div>
  );
};
