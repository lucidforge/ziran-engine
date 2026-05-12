# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A lightweight pinyin input method engine written in Rust, inspired by [librime](https://github.com/rime/librime). Zero external dependencies.

## Commands

```bash
cargo build   # build the project
cargo run     # run CLI demo (reads pinyin from stdin)
```

No lint, test, or typecheck scripts configured yet.

## Architecture

```
Engine → Pipeline → [Segmentor | Translator] → Sort → Candidates
              ↓
           Context (shared state: raw_input → segments → candidates)
```

### Pipeline Flow
1. **Segmentor** splits raw pinyin input into syllables (greedy longest-match)
2. **Translator** looks up phrases first, falls back to per-character lookup
3. **Combinations** generated from segments if phrase matches are insufficient
4. **English fallback** if no Chinese candidates found
5. **Sort** by weight descending, truncate to 50 candidates

### Module Responsibilities

| Module | File | Role |
|--------|------|------|
| Engine | `src/engine.rs` | Owns Context + Pipeline, entry point |
| Context | `src/context.rs` | Shared state: raw_input, segments, candidates |
| Pipeline | `src/pipeline.rs` | Orchestration: segment → translate → combine → sort |
| PinyinSegmentor | `src/segmentor.rs` | Greedy longest-match syllable splitting |
| SimpleTranslator | `src/translator.rs` | HashMap-based phrase/char/en lookup |
| Segment | `src/segment.rs` | Pinyin syllable container |
| Candidate | `src/candidate.rs` | Result item with text + score |

## Dictionary Loading

The translator loads multiple dictionary files at startup:
- Chinese: `data/dict.txt`, `data/ext.dict.yaml`, `data/others.dict.yaml`
- English: `data/en.dict.yaml`, `data/en_ext.dict.yaml`

**Chinese format** (tab-separated): `词语\t拼音1 拼音2\t权重`
**English format** (tab-separated): `单词\t单词\t权重`

The segmentor loads syllables from the same Chinese dictionary files to build its pinyin set.

## Current Limitations

- Greedy (not DP) segmentation — cannot handle ambiguous cases like `xian` → `xi'an` vs `xian`
- Exact match only (HashMap) — no trie/prefix matching
- No user frequency learning or n-gram scoring
- No unit tests yet

## Git Conventions

- Commits: conventional style (`feat:`, `fix:`, `chore:`)
- GitHub email: `lucidforge@users.noreply.github.com` (privacy enabled)
