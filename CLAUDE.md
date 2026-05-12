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
Engine → Pipeline → [build_dag → beam_search → backtrack → finalize] → Candidates
```

### Pipeline Flow
1. **build_dag** — Trie-based prefix matching builds a DAG of all possible syllable edges
2. **try_english_fallback** — if Chinese doesn't fully cover input, try English prefix match
3. **beam_search** — log-weight scoring with segment penalty finds optimal path
4. **backtrack** — extract candidate strings from the best beam paths
5. **finalize** — deduplicate, apply user frequency boost, annotate with bilingual translations, sort

### Module Responsibilities

| Module | File | Role |
|--------|------|------|
| Engine | `src/engine.rs` | Owns Pipeline + UserFreq, entry point |
| Pipeline | `src/pipeline.rs` | DAG → beam search → backtrack → finalize |
| Trie | `src/trie.rs` | Generic `Trie<V>` for O(n×m) prefix matching |
| Dict | `src/dict.rs` | YAML dict loading, builds `Arc<Trie>` + bilingual index |
| DictCompiler | `src/dict_compiler.rs` | Binary cache (ZIRC format) for fast startup |
| UserFreq | `src/user_freq.rs` | User selection frequency tracking (TSV) |
| Candidate | `src/candidate.rs` | Result item: text + score + optional annotation |
| Schema | `src/schema.rs` | YAML schema config loader |

## Dictionary Format

**Chinese** (tab-separated): `词语\t拼音1 拼音2\t权重`
**English** (tab-separated): `单词\t单词\t权重`
**Bilingual** (tab-separated): `中文\t英文\t权重`

Dictionary files referenced in `data/default.yaml` under `translator.dictionaries`.

## Current Features

- Trie-based DAG construction (O(n×m) instead of O(n×K))
- Beam search with log-weight scoring and normalized segment penalty
- Binary dict cache with FNV checksum for fast startup
- User frequency learning (persisted to `data/user_freq.tsv`)
- Bilingual annotations (Chinese→English translations)
- English fallback input

## Git Conventions

- Commits: conventional style (`feat:`, `fix:`, `chore:`)
- GitHub email: `lucidforge@users.noreply.github.com` (privacy enabled)
