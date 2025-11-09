# MeetingCoder - Product Requirements Documentation

## Document Index

This directory contains the complete Product Requirements Document (PRD) for MeetingCoder - a desktop application that transforms stakeholder meetings into working code in real-time.

### 📋 Core Documents

1. **[00-OVERVIEW.md](00-OVERVIEW.md)** - Start here
   - Product vision and value proposition
   - Target users and success metrics
   - Competitive landscape
   - High-level roadmap

2. **[01-PHASE1.md](01-PHASE1.md)** - Foundation (4-6 weeks)
   - System audio capture (Zoom/Google Meet)
   - Continuous transcription
   - Speaker diarization
   - Transcript storage

3. **[02-PHASE2.md](02-PHASE2.md)** - Intelligence (4-6 weeks)
   - LLM provider integration (Claude, OpenAI, Ollama)
   - Requirement extraction
   - Code generation pipeline
   - Incremental updates

4. **[03-PHASE3.md](03-PHASE3.md)** - Real-time Features (3-4 weeks)
5. **[04-PHASE4.md](04-PHASE4.md)** - GitHub Integration (2-3 weeks)
6. **[05-PHASE5.md](05-PHASE5.md)** - Discovery Mode (2-3 weeks)
7. **[06-PHASE6.md](06-PHASE6.md)** - Import Audio & YouTube (2-3 weeks)
   - Live code generation during meetings
   - Code preview server
   - Meeting insights and suggestions
   - Performance optimization

### 🔧 Technical Documents

5. **[TECHNICAL_ARCHITECTURE.md](TECHNICAL_ARCHITECTURE.md)**
   - System architecture and component breakdown
   - Rust backend design (managers, services)
   - React frontend structure
   - Data flow and communication patterns
   - Platform-specific implementations
   - Dependencies and tooling

6. **[API_SPECIFICATIONS.md](API_SPECIFICATIONS.md)**
   - Tauri command interfaces (Frontend ↔ Backend)
   - Tauri event schemas (Backend → Frontend)
   - Internal Rust API contracts
   - LLM provider integration specs
   - Error handling patterns

## Quick Start Guide

### For Product Managers
1. Read: [00-OVERVIEW.md](00-OVERVIEW.md) for vision and goals
2. Review: Phase documents for feature roadmap
3. Focus on: Success criteria and user stories in each phase

### For Engineers
1. Read: [TECHNICAL_ARCHITECTURE.md](TECHNICAL_ARCHITECTURE.md) for system design
2. Review: [API_SPECIFICATIONS.md](API_SPECIFICATIONS.md) for implementation details
3. Start with: Phase 1 implementation checklist

### For LLMs/AI Assistants
When implementing features:
1. Read the relevant phase document for context and requirements
2. Refer to TECHNICAL_ARCHITECTURE.md for implementation patterns
3. Use API_SPECIFICATIONS.md for exact interface contracts
4. Follow the testing strategy outlined in each phase

## Development Timeline

```
┌─────────────┬─────────────┬─────────────┬─────────────┬────────┐
│   Phase 1   │   Phase 2   │   Phase 3   │   Phase 4   │ Launch │
│   (4-6w)    │   (4-6w)    │   (3-4w)    │   (2-3w)    │        │
├─────────────┼─────────────┼─────────────┼─────────────┼────────┤
│ Audio +     │ LLM +       │ Real-time + │ GitHub      │ Beta   │
│ Transcribe  │ CodeGen     │ Polish      │ Integration │ → v1.0 │
└─────────────┴─────────────┴─────────────┴─────────────┴────────┘
      6              12             16            19         21
                  (weeks)
```

**Total Timeline**: 11-16 weeks to MVP
**Target Launch**: Q2 2025

## Key Features Summary

### Phase 1: Audio & Transcription
- ✅ System audio capture (both sides of call)
- ✅ Continuous real-time transcription
- ✅ Speaker diarization (2-3 participants)
- ✅ Timestamped transcript storage

### Phase 2: LLM & Code Generation
- ✅ Multi-provider LLM support (Claude, OpenAI, Ollama)
- ✅ Automatic requirement extraction
- ✅ Code generation (React, Node.js, Python)
- ✅ Incremental updates
- ✅ Code validation and review UI

### Phase 3: Real-Time & Polish
- ✅ Live code generation during meeting
- ✅ Embedded preview server
- ✅ AI-powered insights and suggestions
- ✅ Meeting summary generation
- ✅ Performance optimization (<2min latency)

## Tech Stack

### Core Framework
- **Tauri** - Rust + React desktop app framework
- **Rust** - Backend (audio, transcription, code generation)
- **React + TypeScript** - Frontend UI

### Key Technologies
- **Whisper/Parakeet** - Speech-to-text transcription
- **Claude API / OpenAI API / Ollama** - LLM integration
- **System Audio APIs** - Platform-specific capture
  - macOS: Core Audio
  - Windows: WASAPI
  - Linux: PulseAudio/PipeWire

## System Requirements

### Minimum
- 8GB RAM
- 4-core CPU
- 5GB disk space
- Internet connection (for LLM APIs)

### Recommended
- 16GB RAM
- 8-core CPU
- GPU (for Whisper acceleration)
- Virtual audio device (BlackHole on macOS, VB-Cable on Windows)

## Success Metrics

### Phase 1
- ✅ <5% word error rate on transcription
- ✅ Speaker identification 80%+ accurate
- ✅ No audio dropouts in 60+ min calls

### Phase 2
- ✅ 70%+ of features extracted correctly
- ✅ Generated code passes linting
- ✅ 3+ programming languages supported

### Phase 3
- ✅ Speech → Code in <2 minutes
- ✅ Beta tester satisfaction: 4+ stars
- ✅ 500+ installs in first week

## How to Use These Documents

### Implementation Workflow

1. **Planning Phase**
   - Review 00-OVERVIEW.md with stakeholders
   - Align on priorities and timeline
   - Confirm success criteria

2. **Development Phase**
   - Implement phases sequentially (1 → 2 → 3)
   - Follow acceptance criteria in each phase
   - Refer to TECHNICAL_ARCHITECTURE.md for patterns
   - Use API_SPECIFICATIONS.md for exact contracts

3. **Testing Phase**
   - Execute testing strategy from phase documents
   - Validate against acceptance criteria
   - Performance benchmarks from Phase 3

4. **Launch Phase**
   - Complete Phase 3 checklist
   - Beta testing with target users
   - Marketing and distribution prep

### Feeding to LLMs

When asking an LLM to implement a feature:

```
Please implement [FEATURE_NAME] for MeetingCoder.

Context Documents:
1. Phase specification: [PHASE_X.md]
2. Architecture: [TECHNICAL_ARCHITECTURE.md - relevant section]
3. API spec: [API_SPECIFICATIONS.md - relevant interfaces]

Requirements:
- [Copy relevant acceptance criteria from phase doc]
- Follow the architecture patterns in TECHNICAL_ARCHITECTURE.md
- Implement interfaces defined in API_SPECIFICATIONS.md
- Include unit tests as specified

Please provide:
1. Implementation code (Rust/TypeScript)
2. Tests
3. Documentation updates
```

## Directory Structure

```
docs/prd/
├── README.md                      # This file - index and guide
├── 00-OVERVIEW.md                 # Product vision and goals
├── 01-PHASE1.md                   # Audio capture & transcription
├── 02-PHASE2.md                   # LLM integration & code generation
├── 03-PHASE3.md                   # Real-time streaming & polish
├── 04-PHASE4.md                   # GitHub integration (attach/create repo)
├── TECHNICAL_ARCHITECTURE.md      # System design and implementation
├── 05-PHASE5.md                   # Discovery Mode (live MVP scaffolding)
├── 06-PHASE6.md                   # Import Audio & YouTube as offline meetings
└── API_SPECIFICATIONS.md          # Interface contracts and schemas
```

## Related Resources

### External Documentation
- [Tauri Docs](https://tauri.app/v2/docs) - Desktop app framework
- [Whisper](https://github.com/openai/whisper) - Speech recognition
- [Claude API](https://docs.anthropic.com/claude/reference/messages) - LLM provider
- [OpenAI API](https://platform.openai.com/docs/api-reference) - LLM provider
- [Ollama](https://ollama.ai/) - Local LLM hosting

### Similar Projects
- [Handy](https://github.com/cjpais/Handy) - Foundation for audio/transcription
- [Cursor](https://cursor.sh/) - AI-powered code editor
- [GitHub Copilot](https://github.com/features/copilot) - Code completion

## Contributing

### Updating These Documents

When making changes to requirements:

1. **Create a PR** with document updates
2. **Explain rationale** for changes in PR description
3. **Update version** history at bottom of changed docs
4. **Notify stakeholders** if success criteria change

### Document Versioning

Current version: **v1.0.0** (Initial PRD)

Version format: `MAJOR.MINOR.PATCH`
- **MAJOR**: Significant scope changes, feature additions/removals
- **MINOR**: Clarifications, additional details, non-breaking changes
- **PATCH**: Typo fixes, formatting improvements

### Feedback & Questions

For questions about these requirements:
- Open a GitHub issue with tag `[PRD-Question]`
- Contact: [Your email/Slack]

## Changelog

### v1.0.0 - 2025-01-XX - Initial Release
- Complete PRD for MeetingCoder
- 3-phase development plan (11-16 weeks)
- Full technical architecture
- API specifications
- Success metrics and testing strategy

---

## Quick Reference Tables

### Phase Completion Checklist

| Phase | Duration | Key Deliverables | Status |
|-------|----------|------------------|--------|
| Phase 1 | 4-6 weeks | System audio + transcription | 🔲 Not Started |
| Phase 2 | 4-6 weeks | LLM integration + code gen | 🔲 Not Started |
| Phase 3 | 3-4 weeks | Real-time + polish | 🔲 Not Started |

### LLM Provider Comparison

| Provider | Cost/Hour | Latency | Best For |
|----------|-----------|---------|----------|
| Claude 3.5 Sonnet | ~$8 | Medium | Complex reasoning |
| GPT-4 Turbo | ~$3 | Low | Balanced performance |
| Ollama (Local) | $0 | High | Privacy, offline |

### Platform Support Matrix

| Platform | Audio Capture | Transcription | Code Gen | Preview |
|----------|---------------|---------------|----------|---------|
| macOS (Intel) | ✅ | ✅ | ✅ | ✅ |
| macOS (Apple Silicon) | ✅ | ✅ | ✅ | ✅ |
| Windows x64 | ✅ | ✅ | ✅ | ✅ |
| Linux x64 | ✅ | ✅ | ✅ | ✅ |

### Supported Project Types

| Type | Stack | Generation | Preview |
|------|-------|------------|---------|
| React Web App | React + TypeScript + Vite | ✅ | ✅ |
| Node.js API | Express + TypeScript | ✅ | ⚠️ Limited |
| Python CLI | Click/argparse | ✅ | ❌ |
| Full-Stack | React + Node.js | ✅ | ✅ |
| Static HTML | HTML + CSS + JS | ✅ | ✅ |

---

**Last Updated**: 2025-01-XX
**Document Version**: v1.0.0
**Project Status**: Pre-Development

For the latest version of these documents, see the [GitHub repository](https://github.com/yourusername/meetingcoder).
