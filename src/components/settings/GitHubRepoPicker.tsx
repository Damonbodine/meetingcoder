import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RepoInfo } from "../../lib/types";
import { useSettings } from "../../hooks/useSettings";

export const GitHubRepoPicker: React.FC<{
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}> = ({ descriptionMode = "tooltip", grouped = false }) => {
  const { getSetting, updateSetting } = useSettings();
  const [repos, setRepos] = useState<RepoInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showPicker, setShowPicker] = useState(false);

  const currentOwner = getSetting("github_repo_owner");
  const currentName = getSetting("github_repo_name");
  const currentRepo = repos.find(
    (r) => r.owner.login === currentOwner && r.name === currentName
  );

  const loadRepos = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const repoList = await invoke<RepoInfo[]>("list_github_repos");
      setRepos(repoList);
      setShowPicker(true);
    } catch (err) {
      console.error("Failed to load repos:", err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleSelectRepo = async (repo: RepoInfo) => {
    await updateSetting("github_repo_owner", repo.owner.login);
    await updateSetting("github_repo_name", repo.name);
    await updateSetting("github_default_branch", repo.default_branch);
    setShowPicker(false);
  };

  return (
    <div className={`space-y-3 ${grouped ? "py-3" : "py-4"}`}>
      <div>
        <label className="block text-sm font-medium mb-2">
          Repository Selection
        </label>

        {currentRepo ? (
          <div className="p-3 bg-gray-50 border border-gray-200 rounded-md">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <div className="font-mono text-sm font-medium">
                  {currentRepo.full_name}
                </div>
                {currentRepo.description && (
                  <div className="text-xs text-gray-600 mt-1">
                    {currentRepo.description}
                  </div>
                )}
                <div className="flex items-center gap-2 mt-2">
                  <span
                    className={`px-2 py-0.5 rounded text-xs ${
                      currentRepo.private
                        ? "bg-yellow-100 text-yellow-800"
                        : "bg-green-100 text-green-800"
                    }`}
                  >
                    {currentRepo.private ? "Private" : "Public"}
                  </span>
                  <span className="text-xs text-gray-500">
                    Branch: {currentRepo.default_branch}
                  </span>
                </div>
              </div>
              <button
                onClick={() => setShowPicker(true)}
                className="ml-4 px-3 py-1 text-sm text-blue-600 hover:bg-blue-50 rounded-md"
              >
                Change
              </button>
            </div>
          </div>
        ) : (
          <button
            onClick={loadRepos}
            disabled={isLoading}
            className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-sm"
          >
            {isLoading ? "Loading Repositories..." : "Browse My Repositories"}
          </button>
        )}

        {error && (
          <div className="mt-2 p-2 bg-red-50 text-red-800 text-sm rounded-md">
            {error}
          </div>
        )}
      </div>

      {/* Repository Picker Modal */}
      {showPicker && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] flex flex-col">
            <div className="p-4 border-b border-gray-200 flex items-center justify-between">
              <h3 className="text-lg font-medium text-gray-900">Select Repository</h3>
              <button
                onClick={() => setShowPicker(false)}
                className="text-gray-500 hover:text-gray-700"
              >
                <svg
                  className="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-4 bg-white">
              {isLoading ? (
                <div className="text-center py-8 text-gray-500">
                  Loading repositories...
                </div>
              ) : repos.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  No repositories found
                </div>
              ) : (
                <div className="space-y-2">
                  {repos.map((repo) => (
                    <button
                      key={repo.id}
                      onClick={() => handleSelectRepo(repo)}
                      className="w-full text-left p-3 border border-gray-200 rounded-md hover:bg-gray-50 hover:border-blue-300 transition-colors bg-white"
                    >
                      <div className="font-mono text-sm font-medium text-gray-900">
                        {repo.full_name}
                      </div>
                      {repo.description && (
                        <div className="text-xs text-gray-600 mt-1">
                          {repo.description}
                        </div>
                      )}
                      <div className="flex items-center gap-2 mt-2">
                        <span
                          className={`px-2 py-0.5 rounded text-xs font-medium ${
                            repo.private
                              ? "bg-yellow-100 text-yellow-800"
                              : "bg-green-100 text-green-800"
                          }`}
                        >
                          {repo.private ? "Private" : "Public"}
                        </span>
                        <span className="text-xs text-gray-600">
                          {repo.default_branch}
                        </span>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>

            <div className="p-4 border-t border-gray-200 bg-white">
              <button
                onClick={() => setShowPicker(false)}
                className="w-full px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300 font-medium"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
