import React from "react";
import { useSettings } from "../../hooks/useSettings";

export const GitHubBranchSettings: React.FC<{
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}> = ({ descriptionMode = "tooltip", grouped = false }) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const defaultBranch = getSetting("github_default_branch") ?? "main";
  const branchPattern = getSetting("github_branch_pattern") ?? "meeting/{meeting_id}";

  return (
    <div className={`space-y-3 ${grouped ? "py-3" : "py-4"}`}>
      <div>
        <label className="block text-sm font-medium mb-2">
          Default Branch
        </label>
        <input
          type="text"
          value={defaultBranch}
          onChange={(e) =>
            updateSetting("github_default_branch", e.target.value || "main")
          }
          placeholder="main"
          className="w-full px-3 py-2 border rounded-md text-sm"
          disabled={isUpdating("github_default_branch")}
        />
        <p className="text-xs text-gray-500 mt-1">
          The base branch for pull requests (usually "main" or "master")
        </p>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">
          Branch Naming Pattern
        </label>
        <input
          type="text"
          value={branchPattern}
          onChange={(e) =>
            updateSetting(
              "github_branch_pattern",
              e.target.value || "meeting/{meeting_id}"
            )
          }
          placeholder="meeting/{meeting_id}"
          className="w-full px-3 py-2 border rounded-md font-mono text-sm"
          disabled={isUpdating("github_branch_pattern")}
        />
        <p className="text-xs text-gray-500 mt-1">
          Pattern for branch names. Available variables:{" "}
          <code className="bg-gray-100 px-1 rounded">{"{meeting_id}"}</code>,{" "}
          <code className="bg-gray-100 px-1 rounded">{"{meeting_name}"}</code>
        </p>
      </div>
    </div>
  );
};
