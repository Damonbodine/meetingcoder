# MeetingCoder Orchestration System - Architecture Overview

## Vision

Turn stakeholder conversations into working code while the meeting is still happening. No delays, no manual steps - just autonomous implementation from discussion to deployment.

## Current State vs. The Gap

**What Exists:**
- ✅ Transcript with speaker identification
- ✅ Meeting Updates (20-second intervals)
- ✅ PRD Generation (15-minute intervals + final)
- ✅ Git integration (commit, push, PR)

**What's Missing:** The autonomous bridge from PRD to working code.

## Solution: Parallel Intelligence + Autonomous Execution

```
┌─────────────────────────────────────────────────────────────┐
│  MEETING IN PROGRESS                                        │
│                                                              │
│  ┌──────────────────┐         ┌─────────────────────┐      │
│  │  Conversation    │         │  Codebase Analysis  │      │
│  │  ↓               │         │  (PARALLEL)         │      │
│  │  Transcript      │         │  ↓                  │      │
│  │  ↓               │         │  Scan files         │      │
│  │  Updates (20s)   │         │  ↓                  │      │
│  │  ↓               │         │  Detect tech stack  │      │
│  │  PRD (15min)     │         │  ↓                  │      │
│  │  ↓               │         │  Map features       │      │
│  │  Final PRD       │         │  ↓                  │      │
│  │                  │         │  Find patterns      │      │
│  └────────┬─────────┘         └──────────┬──────────┘      │
│           │                              │                  │
│           └──────────┬───────────────────┘                  │
│                      ↓                                       │
│               Both Complete                                 │
└─────────────────────┼──────────────────────────────────────┘
                      ↓
              Task Generation (LLM-powered)
              PRD + CodebaseContext → Concrete Tasks
                      ↓
              Task Queue (Priority + Dependencies)
                      ↓
              Orchestrator (Execute one-by-one)
                      ↓
              Verify (Compile + Test)
                      ↓
              Git Commit → PR
```

## Core Insight: Parallel Analysis

**The Innovation:** Codebase analysis runs IN PARALLEL with the meeting conversation.

**Why This Matters:**
- Zero delay: Analysis completes before PRD is finalized
- Better accuracy: Tasks include exact file paths and integration points
- Faster execution: No "figuring out where to add code" step

## Implementation Phases

### Phase 1: Parallel Codebase Analysis (Week 1)
- **Trigger:** Meeting starts + project path set
- **Runs:** In background during meeting
- **Outputs:** Tech stack, architecture, features, integration points

### Phase 2: Task Management (Week 1)
- **Purpose:** Infrastructure for task queue and dependency resolution
- **Provides:** Task storage, priority queue, dependency graph

### Phase 3: Context-Aware Task Generation (Week 1)
- **Trigger:** PRD finalized + Codebase Context available
- **Process:** LLM analyzes PRD, suggests exact files, infers dependencies
- **Output:** Concrete CodingTasks with implementation context

### Phase 4: Autonomous Orchestration (Week 1)
- **Process:** Pull task → Build prompt → Execute → Verify → Retry if failed
- **Outputs:** Working code, commits, status updates

### Phase 5: Intelligence & Safety (Week 1)
- **Adds:** Code review, analytics, smart scheduling, circuit breakers
- **Ensures:** Production reliability and cost control

## Success Metrics

**Phase 4 Complete:**
- 80%+ task success rate
- 100% of generated code compiles
- Average 30 minutes from PRD to working code

**Phase 5 Complete:**
- 95%+ task success rate
- Code review catches 90%+ of issues
- Analytics provide actionable insights

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Analysis blocks meeting start | Async background execution, < 30s timeout |
| Generated code doesn't compile | Verification + auto-retry with errors |
| Circular task dependencies | Dependency resolver with cycle detection |
| LLM costs too high | Rate limiting, caching, safety limits |
| Orchestrator gets stuck | Max retries, timeout, skip/block options |

## Dependencies Between Phases

```
Phase 1 (Codebase Analysis)
    ↓ (provides CodebaseContext)
Phase 2 (Task Management)
    ↓ (provides Task Queue)
Phase 3 (Task Generation) ← needs Phase 1 + 2
    ↓ (provides populated tasks)
Phase 4 (Orchestration) ← needs Phase 1 + 2 + 3
    ↓ (provides execution results)
Phase 5 (Intelligence) ← builds on all phases
```

## Next Steps

1. Read [Phase 1 PRD](./PHASE_1_CODEBASE_ANALYSIS.md) in detail
2. Understand the parallel execution model
3. Implement codebase scanner
4. Test with MeetingCoder itself
5. Move to Phase 2
