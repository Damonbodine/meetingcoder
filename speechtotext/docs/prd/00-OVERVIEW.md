# MeetingCoder - Product Requirements Document

## Product Vision

MeetingCoder is a desktop application that transforms stakeholder meetings into working code in real-time. By capturing, transcribing, and interpreting conversations with potential stakeholders during Zoom or Google Meet calls, the application uses AI to generate MVP code as requirements are discussed.

## Problem Statement

Software developers and founders currently face a multi-step, time-consuming process:
1. Have stakeholder meetings and take notes
2. Transcribe or recall requirements after the meeting
3. Translate requirements into technical specifications
4. Begin coding, often days after the conversation
5. Return to stakeholders for clarification on forgotten details

This results in lost context, misunderstood requirements, and slower iteration cycles.

## Solution

MeetingCoder automates the entire pipeline:
- **Capture**: Record both sides of video conference calls (system audio)
- **Transcribe**: Real-time speech-to-text with speaker identification
- **Interpret**: AI model understands requirements from natural conversation
- **Generate**: Produces working MVP code during or immediately after the meeting
- **Iterate**: Updates code as conversation evolves and requirements clarify

## Core Value Propositions

1. **Speed**: MVP code ready by meeting end (vs. days of manual development)
2. **Accuracy**: Captures exact stakeholder language and intent
3. **Context Preservation**: No lost details between meeting and implementation
4. **Iteration**: Live updates as requirements are discussed and refined
5. **Free & Private**: Based on open-source Handy foundation, works offline (except LLM calls)

## Success Metrics

### Phase 1 Success
- Successfully captures system audio from Zoom/Google Meet
- Continuous transcription with <5% word error rate
- Transcripts saved with timestamps and speaker labels

### Phase 2 Success
- LLM successfully extracts requirements from transcripts
- Generates syntactically valid code in target language
- Code addresses 70%+ of discussed features

### Phase 3 Success
- Real-time code generation during meeting (<2 min lag)
- Generated code passes basic tests
- User satisfaction: 4+ stars from beta testers

## Target Users

**Primary**:
- Solo founders conducting customer discovery
- Indie developers building MVPs for clients
- Technical co-founders in early-stage startups

**Secondary**:
- Product managers gathering requirements
- Engineering teams doing discovery calls
- Consultants scoping projects

## Platform Requirements

- **Desktop Application** (Windows, macOS, Linux)
- **Built on Handy foundation** (Tauri + Rust + React/TypeScript)
- **System Requirements**:
  - 8GB RAM minimum (16GB recommended for LLM streaming)
  - Multi-core processor (4+ cores recommended)
  - Internet connection (for LLM API calls)
  - Virtual audio driver support (macOS: BlackHole, Windows: VB-Cable)

## Competitive Landscape

| Solution | Limitation | MeetingCoder Advantage |
|----------|-----------|------------------------|
| Otter.ai + Manual coding | No code generation | Automated code output |
| GitHub Copilot | No meeting context | Direct stakeholder requirements |
| Manual note-taking | Slow, error-prone | Real-time, accurate capture |
| Cursor/Windsurf | Requires manual requirement entry | Auto-extracts from conversation |

## Non-Goals (v1)

- ❌ Video recording/analysis
- ❌ Multi-language support (English only in v1)
- ❌ Hosting/deployment of generated code
- ❌ Team collaboration features
- ❌ Integration with project management tools
- ❌ Custom AI model training

## Development Phases

### Phase 1: Audio Capture & Transcription (4-6 weeks)
Foundation for capturing and transcribing meetings continuously

### Phase 2: LLM Integration & Code Generation (4-6 weeks)
Connect transcriptions to LLM and generate initial code

### Phase 3: Real-Time Streaming & Polish (3-4 weeks)
### Phase 4: GitHub Integration (2-3 weeks)
### Phase 5: Discovery Mode (2-3 weeks)
### Phase 6: Import Audio & YouTube (2-3 weeks)
Live code generation with quality improvements

**Total Estimated Timeline**: 11-16 weeks for MVP

## Tech Stack Decisions

**Inherited from Handy**:
- Tauri (Rust + React/TypeScript framework)
- Whisper/Parakeet for transcription
- cpal for audio I/O
- VAD (Voice Activity Detection)

**New Additions**:
- System audio capture (platform-specific)
- Claude API / OpenAI API / Local LLM support
- File system operations for code generation
- WebSocket or streaming protocol for real-time updates

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Poor transcription accuracy | High | Use Whisper Large, add custom vocabulary |
| LLM hallucination/bad code | High | Add validation layer, human review UI |
| High API costs | Medium | Support local LLMs, implement caching |
| System audio capture complexity | High | Phase 1 focus, platform-specific testing |
| Privacy concerns | Medium | Clear consent UI, local-only option |
| Context window limits | Medium | Intelligent summarization, sliding window |

## Revenue Model (Future Consideration)

While v1 is free and open-source:
- **Freemium**: Free with local LLM, paid for hosted AI
- **Usage-based**: Pay per meeting hour processed
- **Pro tier**: Advanced features (team sharing, custom prompts)
- **Enterprise**: Self-hosted with priority support

## Next Steps

1. Review and approve PRD with stakeholders
2. Set up development environment and project structure
3. Begin Phase 1 implementation (see `01-PHASE1.md`)
4. Establish testing framework and success criteria
5. Create beta tester recruitment plan
