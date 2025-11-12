Technical Specifications & Requirements - MeetingCoder (Tauri Desktop App)

0. LLM Implementation Workflow
Read First: Inspect this file, open issues, and recent commits before making changes. Confirm there are no conflicting high-priority tasks.
Clarify Scope: Summarize the requested change in your own words and highlight unknowns back to the user before coding anything significant.
Design Before Code: Propose the approach (data flow, Rust/React ownership, Tauri command shape) and wait for approval when impact is medium or higher.
Reuse > Rebuild: Prefer existing managers, utilities, and Handy components. Flag gaps before introducing new abstractions.
Guardrails: Do not edit Cargo.lock, package-lock.json, or bun.lockb directly. Use cargo add/bun add commands. Never bypass Tauri's IPC layer.
Verification: Always run cargo build, cargo clippy, bun run build, and cargo test for any touched surface. Share command results or blockers with the user.
Documentation: Update or create README snippets, inline comments, and changelog notes whenever behavior or interfaces shift.

1. Technical Stack

1.1 Core Technologies - Frontend
Package Manager: Bun (preferred) or npm
Language: TypeScript (strict mode)
Framework: React with Vite
UI Components: shadcn/ui
Styling: Tailwind CSS
Animations: Framer Motion
Form Management: React Hook Form
Schema Validation: Zod
State Management: Zustand or React Context
Icons: Lucide React or Font Awesome
Date Handling: date-fns

1.1 Core Technologies - Backend (Rust)
Language: Rust (stable channel, latest)
Framework: Tauri v2
Audio I/O: cpal (cross-platform audio)
Transcription: whisper-rs or parakeet-rs
Voice Activity Detection: vad-rs (Silero VAD)
Audio Processing: rubato (resampling), hound (WAV)
Async Runtime: tokio
Serialization: serde, serde_json
Error Handling: anyhow, thiserror
Global Shortcuts: rdev or tauri-plugin-global-shortcut
System Tray: tauri-plugin-tray

1.2 Platform Services & Storage
LLM APIs: Claude API, OpenAI API, or local LLM support (llama.cpp bindings)
Local Storage: Tauri Store Plugin (settings, user data)
File System: Tauri FS Plugin (code generation output, transcripts)
System Audio Capture: Platform-specific (BlackHole/VB-Cable integration)
Logging: tracing + tracing-subscriber (structured logging)
Error Tracking: Optional Sentry integration via tauri-plugin-sentry
Auto-Updates: tauri-plugin-updater (for distribution)

1.3 Tooling & Testing
Linting (Rust): cargo clippy with project-specific rules in clippy.toml
Linting (TypeScript): ESLint with React + TypeScript recommended config
Formatting (Rust): rustfmt with project rustfmt.toml
Formatting (TypeScript): Prettier (project config)
Unit Tests (Rust): cargo test with #[cfg(test)] modules
Unit Tests (Frontend): Vitest + Testing Library (React)
Integration Tests: Tauri test harness for command/event testing
Static Analysis: cargo check, TypeScript tsc --noEmit
Performance Profiling: cargo flamegraph, Chrome DevTools for frontend

2. Development Practices

Modular Architecture:
  - Rust: Group features by domain in src-tauri/src/managers/, shared utilities in src-tauri/src/lib/
  - Frontend: Group components by feature in src/components/, shared hooks in src/hooks/, utilities in src/lib/

Manager Pattern (Rust Backend):
  - Core business logic lives in managers (AudioManager, TranscriptionManager, ModelManager, LLMManager, CodeGenerationManager)
  - Managers are initialized at startup, stored in Tauri state, and accessed via commands
  - Each manager owns its resources and provides a clear public API

Command-Event Architecture:
  - Frontend → Backend: Tauri commands (async functions marked with #[tauri::command])
  - Backend → Frontend: Tauri events (emit via app.emit() for state updates)
  - Type safety: Share types via typescript generation from Rust (specta + tauri-specta)

Single Responsibility Principle: Rust files approaching 500 lines or frontend files approaching 300 lines should be split. Each component, function, or module should have one clear purpose.

Separation of Concerns:
  - UI logic (React components) separate from business logic (Rust managers)
  - Audio pipeline: Capture → VAD → Transcription → LLM → Code Generation
  - Each stage independently testable

DRY Principle: Reuse code via shared utilities, hooks, and components. Abstract common patterns when they appear in 3+ places.

Git Hygiene: Create topic branches, use conventional commit messages (feat:, fix:, refactor:, docs:), and keep PRs scoped to a single feature or bug fix.

Performance Mindset:
  - Prefer streaming responses for transcription and LLM output
  - Use tokio channels for async communication between managers
  - Minimize Tauri IPC overhead by batching events when appropriate
  - Profile audio pipeline to maintain <100ms latency for real-time transcription

API Design (Tauri Commands):
  - Commands return Result<T, String> for proper error propagation
  - Use strongly-typed payloads (serialize via serde)
  - Document command contracts in inline comments: /// # Arguments, /// # Returns, /// # Errors
  - Version commands if breaking changes occur (e.g., start_recording_v2)

Error Handling & Resiliency:
  - Rust: Use Result/Option, avoid unwrap() in production code, prefer ? operator
  - Frontend: Use try-catch for Tauri invoke, display user-friendly error messages
  - Graceful degradation: If LLM fails, still save transcripts; if transcription fails, save raw audio
  - Log structured errors with context (tracing::error!)

Type Safety:
  - Rust: Leverage type system, avoid String-typed errors in public APIs (use thiserror)
  - TypeScript: Avoid any, use strict mode, define shared types in src/lib/types.ts
  - Generate TypeScript types from Rust structs using specta

Dependency Management:
  - Rust: Add via cargo add <crate>, document rationale in PR
  - Frontend: Add via bun add <package>, prefer peer-compatible versions
  - Remove unused deps promptly: cargo machete, depcheck

Settings & Config:
  - Store settings via tauri-plugin-store (JSON file in app data directory)
  - Never hardcode API keys; use environment variables or secure storage
  - Provide sensible defaults for all settings

Platform Considerations:
  - Test on macOS, Windows, and Linux (different audio backends, permissions)
  - Handle platform-specific permissions (macOS: microphone, accessibility)
  - Adapt UI for platform conventions (macOS uses Cmd, Windows/Linux use Ctrl)

Form Errors: Surface field-level errors from Zod on the field instead of just a single error state for the form. For every disabled button, add a tooltip hover explaining why it's disabled.

3. Non-Functional Requirements

UI/UX:
  - Maintain a clean, professional aesthetic using shadcn/ui components
  - Favor consistent spacing, typography, and interactive states from Tailwind design tokens
  - Support light and dark modes (respect system preference)
  - Minimize app to system tray; provide tray menu for quick actions

Responsiveness: Support window resizing down to 800x600px minimum. Provide responsive layouts that adapt to window size changes.

Accessibility:
  - Meet WCAG 2.1 AA standards
  - Provide keyboard access for all actions (global shortcuts for recording)
  - ARIA labels when semantic HTML is insufficient
  - Focus outlines, respect prefers-reduced-motion

Performance:
  - Target startup time < 2s, UI interactions < 16ms (60fps)
  - Audio pipeline latency < 100ms for real-time feel
  - LLM streaming to show progress (no blocking on long generations)
  - Memory efficient: unload models when not in use

Security:
  - Sanitize all user inputs before passing to LLM or file system
  - Validate file paths to prevent directory traversal
  - Encrypt sensitive data at rest (API keys via keyring if available)
  - Minimize attack surface: disable unnecessary Tauri allowlist APIs

Data Privacy:
  - Store transcripts and recordings locally only (opt-in cloud backup)
  - Clear consent UI before recording meetings
  - Provide easy data deletion (transcripts, audio files, generated code)
  - Comply with GDPR/CCPA: minimal data collection, user control

Loading & Empty States:
  - Show loading indicators for async operations (model download, transcription)
  - Provide helpful empty states (no transcripts yet, no models downloaded)
  - Progress bars for long operations (model download percentage)

System Integration:
  - Single instance enforcement (prevent multiple app instances)
  - Global keyboard shortcuts configurable and non-conflicting
  - Clipboard integration for code output
  - File system notifications when code is generated

4. Observability & Operations

Logging (Rust):
  - Use tracing crate with structured logging
  - Log levels: trace (verbose), debug (development), info (production events), warn (recoverable issues), error (failures)
  - Include context: user actions, request IDs, timestamps
  - Redact sensitive data (API keys, PII)
  - Write logs to app data directory for debugging

Logging (Frontend):
  - Console logging in development only
  - Send critical errors to Rust backend for persistent logging
  - Structured logging with context (component name, user action)

Monitoring:
  - Optional Sentry integration for crash reporting
  - Track key metrics: transcription accuracy, LLM latency, code generation success rate
  - Monitor resource usage: CPU, memory, disk space for audio/transcripts

Feature Flags:
  - Use settings or environment variables to gate experimental features
  - Allow toggling between LLM providers (Claude, OpenAI, local)
  - Enable debug mode for verbose logging

Metrics:
  - Track product metrics: meetings transcribed, code files generated, user satisfaction ratings
  - Document metrics in docs/metrics.md

5. Environment, Deployment & Data

Environment Parity:
  - Maintain .env.example with required variables (LLM API keys, model paths)
  - Provide sane defaults for local development

Build & Release:
  - Development: bun run tauri dev (hot reload frontend, rebuild Rust on save)
  - Production: bun run tauri build (creates platform-specific installers)
  - Code signing: Configure in tauri.conf.json for macOS/Windows distribution
  - Versioning: Follow semver, update in Cargo.toml and package.json

Auto-Updates:
  - Use tauri-plugin-updater for seamless updates
  - Check for updates on startup, prompt user to install
  - Provide release notes in update dialog

Data Storage:
  - Settings: Tauri Store (JSON file in app data directory)
  - Transcripts: Organize by date/meeting in app data directory
  - Generated Code: User-configurable output directory
  - Models: Cache in resources/models/ directory

Backup & Recovery:
  - Document backup locations for user data (transcripts, settings)
  - Provide export/import functionality for settings
  - Graceful recovery from corrupted settings (fall back to defaults)

6. Documentation & Knowledge Sharing

Living Docs:
  - Update docs/prds/ and docs/apis/ when behavior changes
  - Include architecture diagrams for audio pipeline, manager interactions
  - Document Tauri command contracts (inputs, outputs, errors)

Developer Notes:
  - Rust: Use doc comments (///) for public APIs, inline comments (//) for non-obvious logic
  - TypeScript: Use JSDoc for complex functions, inline comments sparingly
  - Explain performance or security decisions (e.g., why a specific audio buffer size)

Changelogs:
  - Record noteworthy updates (features, breaking changes, dependency bumps) in CHANGELOG.md
  - Use conventional commit format for automatic changelog generation

README:
  - Keep CLAUDE.md updated with development commands and architecture overview
  - Provide troubleshooting section for common issues (audio permissions, model downloads)