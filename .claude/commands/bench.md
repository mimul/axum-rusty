<!-- .claude/commands/bench.md -->
---
description: 성능 기준선 측정 및 이전 결과와 비교
allowed-tools: Bash
---

!`cargo bench 2>&1 | tee bench_$(date +%Y%m%d).txt`

결과를 이전 bench_*.txt 파일과 비교해서 성능 회귀가 있는지 알려줘.
