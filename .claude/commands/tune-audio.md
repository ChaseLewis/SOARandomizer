---
description: Run ONE iteration of the GC↔DC audio-tuning optimization loop. Use with `/loop /tune-audio` for self-paced autonomous tuning.
---

# Audio-tuning loop — one iteration

You are running **one iteration** of a closed optimization loop that tunes the GameCube
music mix toward the Dreamcast reference. Do exactly one targeted change, prove it helps,
keep it or revert it, then stop (the `/loop` driver will tick you again).

The tool is `dcaudio2gcaudio` (in `bin/dcaudio2gcaudio`). State lives in `audio_tuning/`.

## Steps (do them in order)

1. **Build gate.** Run `cargo build -p dcaudio2gcaudio`. If it fails, your only job this
   iteration is to fix the build (or `git restore` the offending change) — then stop.

2. **Score.** Run `dcaudio2gcaudio score --all --out audio_tuning/scores/current.json`.
   Read the global score and the per-track / per-instrument breakdown.

3. **Load state** from `audio_tuning/state.json` (create it on first run: record this score
   as `baseline` and `best`, set a `target`, `iter: 0`, empty `history`). Read `changelog.md`
   so you don't repeat a change that was already tried and rejected.

4. **Diagnose.** Rank the worst tracks, then the worst instruments within them. Pick the
   single highest-leverage problem to attack this iteration.

5. **Make ONE change** — the smallest change that should reduce the global score:
   - run the deterministic inner optimizer: `dcaudio2gcaudio tune <track>` (adjusts
     per-instrument level/pan), **or**
   - improve the tool's *method* (renderer fidelity, the comparison metric, the optimizer
     strategy) when the scores reveal a systematic issue, **or**
   - fix a format/parse bug the scores exposed.
   Do not hand-edit raw level bytes — that's the inner optimizer's job.

6. **Re-gate + re-score.** `cargo build -p dcaudio2gcaudio` must pass; then re-run `score`.

7. **Accept or revert.**
   - **Accept** iff the build is green AND the new global score is better than `best`:
     update `state.json` (`best`, `iter += 1`, append to `history`), and every 10 accepted
     iterations copy the tuned GC data to `audio_tuning/best/`.
   - **Otherwise REVERT**: `git restore` the files you changed (and discard any param edits)
     so the working tree returns to best-so-far. The working tree must ALWAYS hold the best
     result.

8. **Log.** Append one line to `audio_tuning/changelog.md`: iteration #, what you changed,
   old→new global score, ACCEPTED or REVERTED, and why.

9. **Decide.** Stop the loop (don't schedule another tick) if ANY of:
   - global score ≤ `target`, or
   - **plateau**: no accepted improvement over the last **K=8** iterations, or
   - **budget**: `iter` ≥ 200 or you're low on the turn's token budget.
   Otherwise this iteration is done; the `/loop` driver will tick again.

## Guardrails (hard rules)
- **Never** accept a change that breaks the build or worsens the global score.
- **Never** `git commit`, push, or run destructive git (`reset --hard`, `clean`, force) —
  revert only via `git restore` of files you touched this iteration.
- One change per iteration; keep diffs small and reviewable.
- The scorer must stay **deterministic** (seeded); don't introduce randomness into scoring.
- The metric optimizes the **tunable** subspace (levels/pan/balance). Do **not** chase the
  Dreamcast reverb/effect tails — they can't be reproduced in the GC engine, and trying will
  stall convergence.

## Notes
- After a good run, the user repacks with `alx_rs --build-iso` and A/B-tests in Dolphin —
  the metric steers, but human ears make the final "solid" call.
- Keep `audio_tuning/state.json` and `changelog.md` tracked; `scores/`, `best/`, and any
  `*.wav` are gitignored build artifacts.
