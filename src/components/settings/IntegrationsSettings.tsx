import React, { useState } from "react";
import { SettingsGroup } from "../ui/SettingsGroup";
import { GitHubEnabled } from "./GitHubEnabled";
import { GitHubOAuth } from "./GitHubOAuth";
import { GitHubToken } from "./GitHubToken";
import { GitHubRepoPicker } from "./GitHubRepoPicker";
import { GitHubRepo } from "./GitHubRepo";
import { GitHubBranchSettings } from "./GitHubBranchSettings";
import { useSettings } from "../../hooks/useSettings";

export const IntegrationsSettings: React.FC = () => {
  const { getSetting } = useSettings();
  const githubEnabled = getSetting("github_enabled") ?? false;
  const advancedEnabled = getSetting("advanced_features_enabled") ?? false;
  const offlineMode = getSetting("offline_mode_enabled") ?? false;
  const [showManualToken, setShowManualToken] = useState(false);
  const [showManualRepo, setShowManualRepo] = useState(false);

  if (!advancedEnabled) {
    return (
      <div className="max-w-3xl w-full mx-auto">
        <div className="rounded-md border border-border p-4 text-sm text-muted-foreground">
          Integrations stay hidden in Recorder mode. Enable Advanced Automations under General → Modes to configure GitHub access, Claude, or IDE launchers.
        </div>
      </div>
    );
  }

  if (offlineMode) {
    return (
      <div className="max-w-3xl w-full mx-auto">
        <div className="rounded-md border border-yellow-500/40 bg-yellow-500/10 p-4 text-sm text-yellow-200">
          Offline mode is enabled. Disable it in General → Modes to authenticate with GitHub or run integrations that require a network connection.
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title="GitHub Integration">
        <GitHubEnabled descriptionMode="tooltip" grouped={true} />
        {githubEnabled && (
          <>
            <div className="border-t pt-4">
              <h4 className="text-sm font-medium text-gray-700 mb-3">Step 1: Authenticate</h4>
              <GitHubOAuth />

              {!showManualToken ? (
                <div className="mt-4 text-center">
                  <button
                    onClick={() => setShowManualToken(true)}
                    className="text-sm text-gray-500 hover:text-gray-700 underline"
                  >
                    Or use a Personal Access Token instead
                  </button>
                </div>
              ) : (
                <div className="mt-4 pt-4 border-t">
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-sm font-medium text-gray-700">Manual Token Entry</h4>
                    <button
                      onClick={() => setShowManualToken(false)}
                      className="text-sm text-gray-500 hover:text-gray-700"
                    >
                      Hide
                    </button>
                  </div>
                  <GitHubToken descriptionMode="tooltip" grouped={true} />
                </div>
              )}
            </div>

            <div className="border-t pt-4 mt-4">
              <h4 className="text-sm font-medium text-gray-700 mb-3">Step 2: Select Repository</h4>
              <GitHubRepoPicker descriptionMode="tooltip" grouped={true} />

              {!showManualRepo ? (
                <div className="mt-2 text-center">
                  <button
                    onClick={() => setShowManualRepo(true)}
                    className="text-sm text-gray-500 hover:text-gray-700 underline"
                  >
                    Or enter repository details manually
                  </button>
                </div>
              ) : (
                <div className="mt-4 pt-4 border-t">
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-sm font-medium text-gray-700">Manual Repository Entry</h4>
                    <button
                      onClick={() => setShowManualRepo(false)}
                      className="text-sm text-gray-500 hover:text-gray-700"
                    >
                      Hide
                    </button>
                  </div>
                  <GitHubRepo descriptionMode="tooltip" grouped={true} />
                </div>
              )}
            </div>

            <div className="border-t pt-4 mt-4">
              <h4 className="text-sm font-medium text-gray-700 mb-3">Step 3: Configure Branch Settings</h4>
              <GitHubBranchSettings descriptionMode="tooltip" grouped={true} />
            </div>
          </>
        )}
      </SettingsGroup>
    </div>
  );
};
