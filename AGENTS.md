# ziran-engine — Agent Notes

## Commands

```bash
cargo build   # build
cargo run     # run CLI demo (reads from stdin)
```

No lint, test, or typecheck scripts configured yet. `cargo build` is the primary verification step.

## Architecture

Inspired by [librime](https://github.com/rime/librime), implemented in Rust with zero external dependencies.

```
Engine → Pipeline → Segmentor → Translator → Sort → Candidates
                ↕
             Context (shared state)
```

| Module | File | Role |
|--------|------|------|
| Engine | `src/engine.rs` | Entry point, owns Context + Pipeline |
| Context | `src/context.rs` | Shared state: raw_input → segments → candidates |
| Pipeline | `src/pipeline.rs` | Orchestrates segment → translate → sort |
| Segmentor | `src/segmentor.rs` | Currently treats entire input as one segment |
| Translator | `src/translator.rs` | Loads `data/dict.txt`, matches pinyin to words |
| Segment | `src/segment.rs` | Segment data struct |
| Candidate | `src/candidate.rs` | Candidate data struct (text + score) |

## Dictionary

`data/dict.txt` — plain text, one entry per line: `pinyin word`
- Multiple words per pinyin key are supported (order determines initial weight)
- Lines starting with `#` or empty lines are skipped

## Git Conventions

- Use `lucidforge@users.noreply.github.com` for commits (GitHub email privacy enabled)
- Commit style: conventional commits (`feat:`, `fix:`, `chore:`)

## Current Limitations (don't assume these will stay)

- Segmentor does NOT split pinyin (e.g., `nihao` stays as one segment, not `ni` + `hao`)
- No trie/prefix matching — exact HashMap lookup only
- No user frequency learning or bigram scoring
- No tests yet
