---
name: Security by Design — 보안 설계 원칙
description: >
  보안 철학과 설계 원칙을 정의하는 기준 문서. 언어·프레임워크에 독립적이며,
  언어별 구현 패턴은 security-rust.md 등 별도 파일을 참조한다.

  포함 내용:
  - §1 악용 시나리오 기반 보안 사고 (Business Logic Vulnerabilities)
  - §2 Threat Modeling — Microsoft SDL 5단계 + STRIDE 위협 분류
  - §3 신뢰 경계(Trust Boundary) 코드 표현 — 외부 입력·서비스 간·권한 경계
  - §4 검증되지 않은 가정 제거 — 입력 신뢰·권한 과신·환경 신뢰
  - §5 OWASP Top 10 2025 패턴 + 자동화 도구(SAST/DAST/SCA/SBOM/시크릿)
  - §6 침해 전제 설계 — Defense in Depth + Assume Breach

  참조 시점: 보안 리뷰·설계 단계 체크·취약점 패턴 판단 기준이 필요할 때
---

#  Security by Design Rules

현대 소프트웨어 개발에서 보안은 더 이상 "나중에 붙이는 옵션"이 아니다. 보안은 설계 이전에 존재하는 사고 방식(mental model)이다. 제품이 어떻게 동작해야 하는가가 아니라 어떻게 악용될 수 있는가를 먼저 생각하는 것이다. 결국 보안은 기술적 대응이 아니라, 가정과 위험을 지속적으로 검증하는 설계 중심 사고 체계다.

> **이 문서의 코드 예시는 개념 설명을 위한 참고 용도다.** 언어별 구체적인 구현 패턴과 체크리스트는 `security-rust.md` 등 언어별 보안 규칙 파일을 참조한다.

---

## 1. 보안은 악용 시나리오에서 시작한다

Microsoft SDL(Security Development Lifecycle)의 핵심 명제는 단순하다.

> "Adding security during the design process is much cheaper and less risky than adding it after the fact."

비용 문제만이 아니다. 설계 단계에서 잘못 정의된 신뢰 경계나 인가 모델은 구현 패치만으로는 근본적으로 고치기 어렵다. OWASP **A06 - Insecure Design**(2025 기준; 2021 버전에서는 A04)이 이를 독립 항목으로 분류하는 이유다. 잘못된 설계에서 비롯된 취약점은 코드 수준의 수정이 아니라 설계 재검토가 필요하다.

SDL Practice 3는 Threat Modeling을 "어떻게 동작하는가(how products should work)"에서 "어떻게 악용될 수 있는가(how products might be abused)"로 시각을 전환하는 과정으로 정의한다. 기능 명세와 함께 악용 명세를 작성하는 습관이 보안 설계의 출발점이다.

Business Logic Vulnerabilities는 이 관점이 실질적으로 요구되는 이유다. 기능 자체는 정상적으로 동작하지만 공격자가 그 흐름을 예상치 못한 순서로 조합해 악용하는 취약점으로, SAST·DAST로는 탐지할 수 없다. 할인 코드 중복 적용, 결제 단계 우회, 권한 확인 없는 다단계 처리 같은 공격은 코드 결함이 아니라 설계 단계에서 악용 시나리오를 검토하지 않은 결과다.

---

## 2. Threat Modeling: 설계에 공격 관점을 통합하라

그 악용 명세를 체계적으로 작성하는 방법론이 Threat Modeling이다. Threat Modeling은 문서를 만드는 작업이 아니라, 설계 단계에서 공격 관점을 통합하는 지속적인 과정이다. 보안 담당자만의 산출물이 아니라 설계자·개발자·운영자가 함께 수행하는 협업 활동이라는 점이 실무에서 중요하다.

SDL은 이를 다섯 단계로 구체화한다.

1. **자산 파악**: 시스템이 처리하는 데이터와 보호 대상 식별. 개인정보·인증 토큰·내부 API 키 등 실제 코드에서 다루는 민감 데이터를 명시한다.
2. **아키텍처 개요 작성**: Data Flow Diagram(DFD)과 신뢰 경계(Trust Boundary) 정의
3. **위협 식별**: STRIDE 방법론 적용
4. **완화 방안 개발**: 계층화된 방어 구조 설계
5. **추적·커뮤니케이션**: 발견 사항을 팀 전체와 공유하고 설계에 반영

**STRIDE**는 위협을 여섯 범주로 분류한다.

| 범주 | 설명 | 소프트웨어 대응 |
|------|------|----------------|
| Spoofing | 신원 위장 | 강한 인증, 토큰 검증 |
| Tampering | 데이터 위조 | 입력 검증, 서명 검증 |
| Repudiation | 행위 부인 | 감사 로그, 불변 로그 저장 |
| Information Disclosure | 정보 노출 | 암호화, 응답 필드 최소화 |
| Denial of Service | 서비스 거부 | 레이트 리미팅, 타임아웃 설계 |
| Elevation of Privilege | 권한 상승 | 최소 권한, 객체 레벨 인가 검증 |

Threat Modeling은 일회성 문서 작업으로 끝내면 효과가 크게 줄어든다. 기능이 추가되거나 변경될 때마다 해당 기능의 Data Flow와 Trust Boundary를 재점검하는 것이 올바른 적용 방식이다.

---

## 3. 신뢰 경계를 코드로 표현하라

Threat Modeling에서 정의한 Trust Boundary는 다이어그램 위에만 머물면 의미가 없다. 실제 코드에서 그 경계를 명시적으로 표현해야 한다. 경계를 넘는 모든 데이터는 검증되어야 하며, 내부 서비스라 해도 예외는 없다.

소프트웨어에서 Trust Boundary는 구체적으로 다음 지점에 존재한다.

- **외부 입력 경계**: HTTP 요청 파라미터, 파일 업로드, 환경 변수, CLI 인자
- **서비스 간 경계**: 내부 마이크로서비스 호출, 메시지 큐 메시지, 외부 서비스 응답
- **권한 경계**: 신뢰 수준이 다른 역할 간 전환(일반 사용자 → 관리자), 상승된 권한이 필요한 기능 접근

검증이 없는 신뢰 경계는 공격 표면이 된다. **OWASP API Security Top 10**(웹 애플리케이션 Top 10과는 독립된 목록)의 **API7 - SSRF**는 서버가 내부 리소스 요청을 검증 없이 처리할 때 발생한다. 서버 내부에서 만들어진 요청이라도 목적지를 신뢰해서는 안 된다.

```typescript
// 잘못된 예: 외부 입력 URL을 그대로 사용
async function fetchResource(url: string) {
  return fetch(url) // SSRF 취약점
}

// 올바른 예: 허용 목록 기반 검증
const ALLOWED_HOSTS = ['api.internal', 'data.internal']

async function fetchResource(url: string) {
  let hostname: string
  try {
    hostname = new URL(url).hostname
  } catch {
    throw new Error('유효하지 않은 URL')
  }
  if (!ALLOWED_HOSTS.includes(hostname)) {
    throw new Error('허용되지 않은 호스트')
  }
  return fetch(url)
}
```

각 경계마다 검증 레이어를 두는 것이 핵심이다. "내부에서 온 요청이니 안전하다"는 가정 자체가 공격 표면을 만든다.

---

## 4. 검증되지 않은 가정이 취약점이 된다

§3이 "어디서 검증할 것인가(경계의 위치)"를 다뤘다면, 이 절은 "무엇을 의심해야 하는가(가정의 종류)"를 점검한다. 보안 실패의 근본 원인은 코드 자체보다, 검증되지 않은 가정(Assumption)에 있다. "신뢰하되 검증하라(Trust, but verify)"는 충분하지 않다. 보안에서 정확한 원칙은 "검증 없이는 신뢰하지 않는다(Never trust, always verify)"다. 신뢰를 전제하느냐 배제하느냐의 차이가 설계 전체를 바꾼다.

실제로 많은 취약점은 세 가지 잘못된 가정에서 발생한다.

- **입력 신뢰**: 사용자 입력이 의도대로만 들어올 것이라는 가정
- **권한 과신**: 인증된 사용자는 모든 리소스에 접근 가능하다는 가정
- **환경 신뢰**: 내부 서비스 또는 외부 라이브러리는 안전하다는 가정

이 가정들은 코드로 명시적으로 깨뜨릴 수 있다. 타입 시스템으로 잘못된 입력을 컴파일 시점에 차단하고, 스키마 검증으로 런타임 경계를 명시하며, 객체 레벨 권한 검증으로 "인증 = 모든 데이터 접근 가능"이라는 가정을 제거한다.

```typescript
// 잘못된 예: 인증만 확인, 소유권은 가정
async function getDocument(userId: string, docId: string) {
  return db.documents.findById(docId) // 인증된 사용자라면 타인의 문서도 조회 가능
}

// 올바른 예: 객체 레벨 권한 검증 (OWASP BOLA 대응)
async function getDocument(userId: string, docId: string) {
  const doc = await db.documents.findById(docId)
  if (!doc || doc.ownerId !== userId) {
    throw new ForbiddenError('접근 권한 없음')
  }
  return doc
}
```

세 가지 가정은 서로 다른 공격 벡터로 이어진다. "환경 신뢰" 가정의 가장 흔한 사례는 시크릿 하드코딩이다. API 키나 DB 비밀번호를 코드에 직접 넣으면 git 이력에 영구 보존되고, 저장소가 공개되는 순간 즉시 노출된다.

```typescript
// 잘못된 예: 환경을 신뢰, 시크릿을 코드에 직접 기재
const apiKey = 'sk-proj-xxxxxxxxxxxx'
const db = new Database({ password: 'admin1234' })

// 올바른 예: 환경 변수로 분리, 미설정 시 명시적으로 실패
const apiKey = process.env.OPENAI_API_KEY
if (!apiKey) throw new Error('OPENAI_API_KEY가 설정되지 않았습니다')
const db = new Database({ password: process.env.DB_PASSWORD })
```

또 다른 "환경 신뢰" 사례로는 AI 에이전트의 간접 프롬프트 인젝션이 있다(GitHub Security Lab 연구). 코드 저장소·이슈·PR 같은 개발 환경 컨텍스트에 공격자가 악성 지시를 삽입하면, AI 에이전트는 이를 신뢰된 명령으로 처리해 인증 토큰 탈취나 악성 커밋으로 이어질 수 있다. "AI 에이전트가 읽는 컨텍스트는 안전하다"는 가정 자체가 공격 표면이 된다. 공격자는 코드를 깨는 것이 아니라, 개발자의 가정을 깨는 존재다.

KISA 소프트웨어 개발보안 가이드가 입력 데이터 검증, 보안 기능, 에러 처리, 코드 품질을 독립된 항목으로 구분해 다루는 것도 같은 맥락이다. 각 항목은 "여기서 무엇을 신뢰하고 있는가"를 점검하는 체크리스트로 읽힌다.

---

## 5. 알려진 패턴을 반복 적용하라

보안 취약점의 대부분은 새로운 문제가 아니라 반복되는 패턴이다. OWASP는 이를 체계화한 가장 대표적인 기준이다.

**OWASP Top 10 2025** 주요 항목을 보면 패턴이 보인다(2021 버전과 항목 순서 및 번호가 다르다).

| 순위 | 항목 | 설계 실패 원인 |
|------|------|----------------|
| A01 | 접근 제어 우회 | 권한 검증 누락 |
| A02 | 보안 설정 오류 | 기본값 신뢰 |
| A03 | 소프트웨어 공급망 실패 | 외부 의존성 신뢰 |
| A04 | 암호화 결함 | 민감 데이터 처리 미흡 |
| A05 | 인젝션 | 입력 검증 부재 |
| A06 | 안전하지 않은 설계 | 설계 단계 보안 누락 |
| A07 | 인증 실패 | 인증 메커니즘 약점 |
| A08 | 소프트웨어·데이터 무결성 실패 | 업데이트·데이터 검증 부재 |
| A09 | 보안 로깅·경보 실패 | 탐지·감시 미비 |
| A10 | 예외 조건 오류 처리 | 오류 처리 취약점 |

각 항목의 설계 실패 원인을 보면 공통 패턴이 드러난다. 입력 검증 부재, 권한 검증 누락, 기본값 신뢰가 A01·A02·A05·A07을 포함한 다수 항목의 근본 원인이다. 이 세 가지를 코드 리뷰 체크리스트에 명시적으로 포함하는 것이 실질적인 출발점이 된다. A03(공급망)·A04(암호화)처럼 별도의 아키텍처 결정이 필요한 항목도 있지만, 세 가지 원인이 교차되는 취약점 영역이 가장 넓다.

**A04 암호화 결함**은 잘못된 알고리즘 선택이 코드 한 줄에서 발생한다. 패스워드 해싱이 대표적이다. MD5·SHA1은 연산 속도가 빠르다는 이유로 여전히 사용되지만, 그 빠름이 곧 레인보우 테이블 공격의 이점이 된다.

```typescript
// 잘못된 예: MD5·SHA1은 레인보우 테이블 공격에 취약
import crypto from 'node:crypto'
const hashed = crypto.createHash('sha1').update(password).digest('hex')

// 올바른 예: bcrypt 또는 Argon2 — 솔트 자동 포함, 연산 비용 조절 가능
// Argon2는 bcrypt의 72바이트 입력 제한이 없어 OWASP Password Storage Cheat Sheet가 우선 권장
import bcrypt from 'bcrypt'
const hashed = await bcrypt.hash(password, 12)       // cost factor 12 이상 권장
const isMatch = await bcrypt.compare(input, hashed)
```

2025 버전에서 **A03 소프트웨어 공급망 실패**가 독립 항목으로 분리된 점은 주목할 만하다. 2021 버전까지는 A08(소프트웨어·데이터 무결성 실패) 안에 포함됐지만, npm 등 패키지 레지스트리를 통한 공급망 공격의 빈도와 영향 범위가 커지면서 별도 항목이 됐다. 외부 의존성은 이제 코드베이스의 일부가 아니라 독립적인 공격 표면이다. SBOM(Software Bill of Materials)은 이 공격 표면을 가시화하는 수단이다. 소프트웨어에 포함된 모든 구성 요소와 버전을 기록한 명세서로, 새로운 CVE가 공개됐을 때 영향받는 컴포넌트를 즉시 특정할 수 있고, 침해 발생 후 범위를 신속하게 파악하는 데도 활용된다.

패턴을 알았다면, 그것을 반복 적용하는 과정을 자동화해야 한다. 코드 자체의 취약점은 SAST로, 런타임 취약점은 DAST로, 의존성 취약점은 SCA로, 시크릿 노출은 전용 스캐너로 탐지한다.

| 분류 | 도구 | 특징 | 적합한 상황 |
|------|------|------|-------------|
| SAST | [Semgrep](https://semgrep.dev) | 정적 분석, 500+ 보안 규칙 내장, 커스텀 규칙 작성 가능 | 코드 작성·PR 단계 |
| DAST | [OWASP ZAP](https://www.zaproxy.org) | 실행 중인 애플리케이션을 대상으로 런타임 취약점 탐지 | 스테이징·QA 단계 |
| SCA | [Dependabot](https://docs.github.com/en/code-security/dependabot) | GitHub 기본 제공, 취약 의존성 자동 PR 생성 | GitHub 프로젝트 |
| SCA | [OSV-Scanner](https://github.com/google/osv-scanner) | Google 개발, OSV.dev DB 활용, 11개 이상 언어·19개 이상 락파일 지원 | 로컬·CI 통합, 다양한 언어 생태계 |
| SCA + 기타 | [Trivy](https://github.com/aquasecurity/trivy) | 컨테이너 이미지·파일시스템·Git 저장소 스캔, 시크릿·설정 오류도 탐지 | 컨테이너 기반 서비스 |
| SBOM | [syft](https://github.com/anchore/syft) | 컨테이너·파일시스템에서 SBOM 자동 생성, CycloneDX·SPDX 포맷 지원 | 공급망 보안, 의존성 가시화 |
| 시크릿 탐지 | [Gitleaks](https://github.com/gitleaks/gitleaks) | Git 이력 전체에서 시크릿 패턴 탐지, pre-commit 훅 통합 가능 | 시크릿 하드코딩 방지 |

**SCA 도구 선택 기준**: GitHub 프로젝트는 Dependabot을 기본으로 활성화한다. 다중 언어 생태계이거나 GitHub 외 환경이라면 OSV-Scanner를 병행 또는 대체한다. 컨테이너 기반 서비스는 Trivy 단독으로 SCA·시크릿·설정 오류를 함께 커버할 수 있다.

도구는 실행 시점이 중요하다. 배포 후에 발견하면 git 이력 재작성, 시크릿 로테이션, 영향 범위 파악까지 수습 비용이 커진다. SAST → SCA → 시크릿 탐지를 PR 머지 전 CI 게이트에 순서대로 두는 것이 Shift Left의 핵심이다. 문제를 오른쪽(배포·운영)에서 발견할수록 비용은 기하급수적으로 증가한다.

다만 도구는 알려진 CVE와 시크릿 패턴을 탐지하는 것이지, 설계 결함이나 인가 오류를 찾아주지 않는다. 패턴 지식 없이 도구만 실행하면 알림이 쏟아지는데 어디서부터 대응해야 할지 판단하기 어렵다. 도구는 OWASP 패턴 지식의 보완재이지 대체재가 아니다.

보안은 창의적 대응이 아니라, 검증된 취약점 패턴을 이해하고 반복적으로 적용하는 구조적 학습의 문제다. 그러나 패턴과 도구가 모든 공격을 막아주지는 않는다. 남은 질문은 "공격이 성공했을 때 어떻게 살아남을 것인가"다.

---

## 6. 침해를 전제로 설계하라

보안은 모든 공격을 완전히 차단하는 것이 아니라, 공격의 성공을 지연시키고 피해 범위를 제한하는 전략이다. 이 전략은 시간축에 따라 두 원칙으로 나뉜다. 침해가 발생하기 전의 방어 설계(Defense in Depth)와 침해가 발생한 후의 생존 설계(Assume Breach)다.

**Defense in Depth**: 여러 계층의 통제를 쌓아 단일 방어가 뚫려도 다음 계층이 탐지하거나 지연시킨다. 각 계층이 완벽하지 않더라도 공격자의 시간과 비용을 높이는 것 자체가 의미 있다. 코드 레벨에서는 스키마 검증, 인가 검증, 에러 처리, 감사 로그가 각각 독립적인 방어 계층으로 작동한다. 어느 하나가 누락돼도 나머지가 보완한다.

**Assume Breach**: 침해가 언젠가 발생한다는 전제 아래 시스템을 설계한다. 공격이 성공했을 때 피해 범위(Blast Radius)를 어떻게 제한할 것인가를 사전에 결정한다.

구체적으로는 최소 권한 원칙으로 계정 하나가 뚫려도 시스템 전체가 노출되지 않게 하고, 서비스 간 격리로 이동 경로를 차단하며, 감사 로그로 침해 시점과 범위를 빠르게 파악한다. **OWASP A09 - 보안 로깅·경보 실패**가 강조하는 이유가 여기 있다. 공격이 진행 중일 때 탐지하지 못하면 대응 시간 자체를 잃는다.

```typescript
// Information Disclosure 방어 — 내부 오류 정보를 응답에 포함하지 않는다
app.use((err: Error, req: Request, res: Response, next: NextFunction) => {
  // 스택 트레이스는 로그에만, 응답에는 절대 포함하지 않음
  logger.error({ err, requestId: req.id })
  res.status(500).json({ error: '요청을 처리할 수 없습니다' })
})
```

공급망 보안도 같은 논리다. 외부 의존성이 이미 침해됐다고 가정하고, 의존성 서명 검증과 최소 권한 토큰으로 피해 범위를 사전에 제한한다.

안전한 시스템은 "뚫리지 않는 시스템"이 아니라, 뚫려도 무너지지 않는 시스템이다.

