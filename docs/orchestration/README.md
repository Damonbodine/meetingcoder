# MeetingCoder Orchestration System

**Transform every meeting into working code automatically.**

## Quick Links

- [📋 Overview & Architecture](./00_OVERVIEW.md) - Start here
- [🔍 Phase 1: Codebase Analysis](./PHASE_1_CODEBASE_ANALYSIS.md) - Parallel analysis during meeting
- [📦 Phase 2: Task Management](./PHASE_2_TASK_MANAGEMENT.md) - Task queue system
- [🎯 Phase 3: Task Generation](./PHASE_3_TASK_GENERATION.md) - PRD to tasks conversion
- [⚙️ Phase 4: Orchestration](./PHASE_4_ORCHESTRATION.md) - Automated execution
- [🚀 Phase 5: Intelligence](./PHASE_5_INTELLIGENCE.md) - Advanced features

## What is This?

The Orchestration System converts meeting discussions into actual working code. It runs **during your meeting** - analyzing your codebase in parallel while capturing requirements - then autonomously implements the discussed features.

## The Flow

```
Meeting Starts (project path set)
        ↓
    ┌───────────────────────────────┐
    │   PARALLEL EXECUTION          │
    │                               │
    │  Track A: Conversation        │  Track B: Codebase Analysis
    │  → Transcript                 │  → Scan file structure
    │  → Updates (20s)              │  → Detect tech stack
    │  → PRD (15min intervals)      │  → Map features
    │  → Final PRD (meeting end)    │  → Find patterns
    │                               │
    └───────────┬───────────────────┘
                ↓
        Both Complete
                ↓
        Task Generation (PRD + Codebase Context)
                ↓
        Task Queue (Dependency-resolved)
                ↓
        Autonomous Execution (One task at a time)
                ↓
        Verification (Compile + Test)
                ↓
        Git Commit → PR
```

## Key Insight

**Codebase analysis happens IN PARALLEL with your meeting.** By the time the PRD is ready, the system already understands your project structure, conventions, and where to add new code.

## Implementation Phases

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 1 week | Parallel codebase analysis |
| Phase 2 | 1 week | Task queue system |
| Phase 3 | 1 week | Context-aware task generation |
| Phase 4 | 1 week | Autonomous execution |
| Phase 5 | 1 week | Production polish |
| **Total** | **5 weeks** | **Full orchestration** |

## Success Criteria

- ✅ Codebase analysis completes before PRD finalization
- ✅ 90%+ of generated tasks have correct file locations
- ✅ 80%+ of tasks execute successfully
- ✅ 100% of generated code compiles
- ✅ Zero manual intervention needed for simple features

## Getting Started

1. **Read the architecture**: [00_OVERVIEW.md](./00_OVERVIEW.md)
2. **Implement Phase 1 first**: Codebase analysis is foundational
3. **Test parallel execution**: Ensure analysis doesn't block meeting
4. **Implement phases sequentially**: Each builds on previous

---

**Ready to start?** → [Read the Overview](./00_OVERVIEW.md) → [Implement Phase 1](./PHASE_1_CODEBASE_ANALYSIS.md)
