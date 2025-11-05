import React from "react";
import { SettingsGroup } from "../ui/SettingsGroup";
import { GitHubEnabled } from "./GitHubEnabled";
import { GitHubToken } from "./GitHubToken";
import { GitHubRepoPicker } from "./GitHubRepoPicker";
import { GitHubRepo } from "./GitHubRepo";
import { GitHubBranchSettings } from "./GitHubBranchSettings";
import { useSettings } from "../../hooks/useSettings";

export const IntegrationsSettings: React.FC = () => {
  const { getSetting } = useSettings();
  const githubEnabled = getSetting("github_enabled") ?? false;

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title="GitHub">
        <GitHubEnabled descriptionMode="tooltip" grouped={true} />
        {githubEnabled && (
          <>
            <GitHubToken descriptionMode="tooltip" grouped={true} />
            <GitHubRepoPicker descriptionMode="tooltip" grouped={true} />
            <GitHubRepo descriptionMode="tooltip" grouped={true} />
            <GitHubBranchSettings descriptionMode="tooltip" grouped={true} />
          </>
        )}
      </SettingsGroup>
    </div>
  );
};

