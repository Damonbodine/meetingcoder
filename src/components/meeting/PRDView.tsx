import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { PRDVersion, PRDChangelog } from "../../lib/types";
import { toast } from "sonner";
import { FileText, Download, GitCompare, RefreshCw } from "lucide-react";
import { Button } from "../ui/Button";

interface PRDViewProps {
  meetingId: string;
}

export const PRDView: React.FC<PRDViewProps> = ({ meetingId }) => {
  const [versions, setVersions] = useState<PRDVersion[]>([]);
  const [selectedVersion, setSelectedVersion] = useState<number | null>(null);
  const [prdContent, setPrdContent] = useState<string>("");
  const [changelog, setChangelog] = useState<PRDChangelog | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [showDiff, setShowDiff] = useState(false);
  const [compareFromVersion, setCompareFromVersion] = useState<number | null>(null);
  const [compareToVersion, setCompareToVersion] = useState<number | null>(null);

  // Load PRD versions
  const loadVersions = async () => {
    try {
      const versionsList = await invoke<PRDVersion[]>("get_prd_versions", {
        meetingId,
      });
      setVersions(versionsList);

      // Auto-select latest version if none selected
      if (versionsList.length > 0 && selectedVersion === null) {
        setSelectedVersion(versionsList[versionsList.length - 1].version);
      }
    } catch (error) {
      console.error("Failed to load PRD versions:", error);
    }
  };

  // Load PRD content for selected version
  const loadContent = async (version: number) => {
    setIsLoading(true);
    try {
      const content = await invoke<string>("get_prd_content", {
        meetingId,
        version,
      });
      setPrdContent(content);
    } catch (error) {
      console.error("Failed to load PRD content:", error);
      toast.error("Failed to load PRD content", {
        description: String(error),
      });
    } finally {
      setIsLoading(false);
    }
  };

  // Load changelog
  const loadChangelog = async () => {
    try {
      const log = await invoke<PRDChangelog>("get_prd_changelog", {
        meetingId,
      });
      setChangelog(log);
    } catch (error) {
      console.error("Failed to load changelog:", error);
    }
  };

  // Generate PRD manually
  const handleGeneratePRD = async () => {
    setIsGenerating(true);
    try {
      const newVersion = await invoke<PRDVersion>("generate_prd_now", {
        meetingId,
      });
      toast.success("PRD generated", {
        description: `Version ${newVersion.version} created`,
      });
      await loadVersions();
      setSelectedVersion(newVersion.version);
    } catch (error) {
      console.error("Failed to generate PRD:", error);
      toast.error("Failed to generate PRD", {
        description: String(error),
      });
    } finally {
      setIsGenerating(false);
    }
  };

  // Export PRD
  const handleExport = async (format: "markdown" | "pdf") => {
    if (selectedVersion === null) return;

    try {
      const filePath = await invoke<string>("export_prd", {
        meetingId,
        version: selectedVersion,
        format,
      });
      toast.success("PRD exported", {
        description: `Saved to ${filePath}`,
      });
    } catch (error) {
      console.error("Failed to export PRD:", error);
      toast.error("Failed to export PRD", {
        description: String(error),
      });
    }
  };

  // Initialize
  useEffect(() => {
    loadVersions();
    loadChangelog();

    // Poll for updates every 30 seconds
    const interval = setInterval(() => {
      loadVersions();
      loadChangelog();
    }, 30000);

    // Listen for PRD version generated events
    let unlisten: (() => void) | undefined;
    const setupListener = async () => {
      try {
        const unlistenFn = await listen<{ version: PRDVersion }>(
          "prd-version-generated",
          (event) => {
            toast.success("New PRD version generated", {
              description: `Version ${event.payload.version.version}`,
            });
            loadVersions();
          }
        );
        unlisten = unlistenFn;
      } catch (error) {
        console.error("Failed to setup PRD event listener:", error);
      }
    };

    setupListener();

    return () => {
      clearInterval(interval);
      if (unlisten) unlisten();
    };
  }, [meetingId]);

  // Load content when version changes
  useEffect(() => {
    if (selectedVersion !== null) {
      loadContent(selectedVersion);
    }
  }, [selectedVersion]);

  // Empty state - no versions yet
  if (versions.length === 0) {
    return (
      <div className="space-y-4">
        <div className="rounded-md border border-border p-6 text-center">
          <FileText className="mx-auto h-12 w-12 text-muted-foreground mb-3" />
          <h3 className="text-lg font-medium mb-2">No PRD Generated Yet</h3>
          <p className="text-sm text-muted-foreground mb-4">
            PRDs are automatically generated after 15+ transcript segments, or you
            can generate one manually now.
          </p>
          <Button
            onClick={handleGeneratePRD}
            disabled={isGenerating}
            variant="primary"
          >
            {isGenerating ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Generating...
              </>
            ) : (
              <>
                <FileText className="mr-2 h-4 w-4" />
                Generate PRD Now
              </>
            )}
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Top toolbar */}
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium">Version:</label>
          <select
            value={selectedVersion ?? ""}
            onChange={(e) => setSelectedVersion(Number(e.target.value))}
            className="rounded border border-border px-3 py-1 text-sm bg-background"
          >
            {versions.map((v) => (
              <option key={v.version} value={v.version}>
                v{v.version} ({v.version_type}) -{" "}
                {new Date(v.generated_at).toLocaleString()}
              </option>
            ))}
          </select>
        </div>

        <div className="flex items-center gap-2">
          <Button
            onClick={handleGeneratePRD}
            disabled={isGenerating}
            variant="secondary"
            size="sm"
          >
            {isGenerating ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
          </Button>

          <Button
            onClick={() => {
              setShowDiff(!showDiff);
              if (!showDiff && versions.length >= 2) {
                setCompareFromVersion(versions[0].version);
                setCompareToVersion(versions[versions.length - 1].version);
              }
            }}
            disabled={versions.length < 2}
            variant="secondary"
            size="sm"
          >
            <GitCompare className="mr-2 h-4 w-4" />
            Compare
          </Button>

          <Button
            onClick={() => handleExport("markdown")}
            disabled={selectedVersion === null}
            variant="secondary"
            size="sm"
          >
            <Download className="mr-2 h-4 w-4" />
            Export MD
          </Button>
        </div>
      </div>

      {/* Version info */}
      {selectedVersion !== null && (
        <div className="rounded-md border border-border p-3 text-sm text-muted-foreground">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
            <div>
              <span className="font-medium">Segments:</span>{" "}
              {versions.find((v) => v.version === selectedVersion)?.segment_range[0]}-
              {versions.find((v) => v.version === selectedVersion)?.segment_range[1]}
            </div>
            <div>
              <span className="font-medium">Words:</span>{" "}
              {versions.find((v) => v.version === selectedVersion)?.word_count}
            </div>
            <div>
              <span className="font-medium">Confidence:</span>{" "}
              {((versions.find((v) => v.version === selectedVersion)?.confidence ?? 0) * 100).toFixed(0)}%
            </div>
            <div>
              <span className="font-medium">Type:</span>{" "}
              {versions.find((v) => v.version === selectedVersion)?.version_type}
            </div>
          </div>
        </div>
      )}

      {/* Diff view */}
      {showDiff && versions.length >= 2 && (
        <div className="rounded-md border border-border p-4">
          <div className="flex items-center gap-4 mb-4">
            <div className="flex items-center gap-2">
              <label className="text-sm font-medium">From:</label>
              <select
                value={compareFromVersion ?? ""}
                onChange={(e) => setCompareFromVersion(Number(e.target.value))}
                className="rounded border border-border px-3 py-1 text-sm bg-background"
              >
                {versions.map((v) => (
                  <option key={v.version} value={v.version}>
                    v{v.version}
                  </option>
                ))}
              </select>
            </div>
            <div className="flex items-center gap-2">
              <label className="text-sm font-medium">To:</label>
              <select
                value={compareToVersion ?? ""}
                onChange={(e) => setCompareToVersion(Number(e.target.value))}
                className="rounded border border-border px-3 py-1 text-sm bg-background"
              >
                {versions.map((v) => (
                  <option key={v.version} value={v.version}>
                    v{v.version}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {compareFromVersion !== null && compareToVersion !== null && (
            <div className="space-y-2">
              {changelog?.changes
                .filter(
                  (c) =>
                    c.from_version === compareFromVersion &&
                    c.to_version === compareToVersion
                )
                .map((change, idx) => (
                  <div key={idx} className="space-y-3">
                    {change.added_user_stories.length > 0 && (
                      <div>
                        <div className="text-sm font-medium text-green-600 mb-1">
                          Added User Stories:
                        </div>
                        <ul className="list-disc pl-5 text-sm">
                          {change.added_user_stories.map((s, i) => (
                            <li key={i}>{s}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {change.modified_user_stories.length > 0 && (
                      <div>
                        <div className="text-sm font-medium text-yellow-600 mb-1">
                          Modified User Stories:
                        </div>
                        <ul className="list-disc pl-5 text-sm">
                          {change.modified_user_stories.map((s, i) => (
                            <li key={i}>{s}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {change.added_requirements.length > 0 && (
                      <div>
                        <div className="text-sm font-medium text-green-600 mb-1">
                          Added Requirements:
                        </div>
                        <ul className="list-disc pl-5 text-sm">
                          {change.added_requirements.map((r, i) => (
                            <li key={i}>{r}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {change.resolved_questions.length > 0 && (
                      <div>
                        <div className="text-sm font-medium text-blue-600 mb-1">
                          Resolved Questions:
                        </div>
                        <ul className="list-disc pl-5 text-sm">
                          {change.resolved_questions.map((q, i) => (
                            <li key={i}>{q}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {change.new_questions.length > 0 && (
                      <div>
                        <div className="text-sm font-medium text-purple-600 mb-1">
                          New Questions:
                        </div>
                        <ul className="list-disc pl-5 text-sm">
                          {change.new_questions.map((q, i) => (
                            <li key={i}>{q}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                ))}
            </div>
          )}
        </div>
      )}

      {/* Content viewer */}
      <div className="rounded-md border border-border p-6 prose prose-sm max-w-none dark:prose-invert">
        {isLoading ? (
          <div className="text-center py-8">
            <RefreshCw className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
            <p className="mt-2 text-sm text-muted-foreground">Loading PRD...</p>
          </div>
        ) : (
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{prdContent}</ReactMarkdown>
        )}
      </div>
    </div>
  );
};
