<!-- .claude/commands/check.md -->
---
description: cargo 전체 검증 (fmt + clippy + test)
allowed-tools: Bash
---

아래 순서로 검증을 실행하고 결과를 요약해줘:

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test --all`

각 단계 실패 시 원인과 수정 방법을 알려줘.
