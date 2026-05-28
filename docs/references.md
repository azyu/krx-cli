# Project References

## Project Overview

`krx-cli`는 한국거래소 Open API를 호출하는 Rust 기반 CLI 도구다. 1차 목표는 KRX가 공개한 31개 읽기 전용 API를 안정적으로 호출할 수 있는 실행 파일을 만드는 것이고, 2차 목표는 사람뿐 아니라 AI 에이전트도 안전하게 사용할 수 있는 인터페이스를 제공하는 것이다. 현재 구조는 `krx-cli`와 `krx-core` 두 crate로 나뉘며, `krx` 바이너리를 배포한다. 범위는 내장 API 카탈로그, 구조화된 스키마 조회, `basDd` 검증, 샘플/실서버 엔드포인트 전환, `--dry-run`, 출력 필드 메타데이터, 최소 응답 축소(`--body-only`), 선택 필드 축소(`--fields`), 그리고 `krx mcp serve` 기반의 최소 read-only MCP 노출까지 포함한다. `--fields`는 JSON body의 row 객체만 줄이고 `OutBlock_1` 같은 최상위 컨테이너는 유지한다. 실서버 non-2xx 응답은 알려진 `401` 메시지는 전용 오류로, 그 외는 일반 `krx_api_error`로 정규화한다. 공용 runtime 표면은 `krx-core`에 두고, CLI와 MCP가 함께 재사용한다.

## Tech Stack

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| Language | Rust | edition 2024 | workspace 구성과 단일 `krx` 바이너리 배포를 함께 가져가기에 적합 |
| CLI | clap | 4.x | Rust 생태계 표준 CLI 파서, derive 기반 선언형 인터페이스 제공 |
| HTTP | reqwest (blocking) | 0.12.x | 단순 GET 호출, TLS 지원, 초기 스캐폴드에 충분 |
| Serialization | serde / serde_json | 1.x | 구조화된 출력과 JSON 입력 파싱에 필요 |
| Error Handling | thiserror | 2.x | 명확한 사용자 오류와 내부 오류 구분 |
| Documentation | Markdown | n/a | 런타임 외부 문서와 에이전트용 컨텍스트 유지 |

## Architecture Decisions

### ADR-1: Agent-friendly CLI를 초기 설계에 포함한다

- **Status:** Accepted
- **Context:** 사용자 요청이 명시적으로 참고한 글은 사람 중심 CLI와 에이전트 중심 CLI가 다르며, 후속 개조보다 처음부터 구조화 출력과 스키마 인트로스펙션을 넣는 편이 낫다고 주장한다.
- **Decision:** `schema list`, `schema show`, `--output json`, `--dry-run`, `--params` JSON 입력을 초기 스캐폴드에 포함한다.
- **Consequences:** 초기 구현량은 조금 늘어나지만, 이후 MCP 노출이나 자동화 통합이 쉬워진다.

### ADR-2: 런타임 문서 의존 대신 내장 API 카탈로그를 둔다

- **Status:** Accepted
- **Context:** KRX Open API는 HTML 페이지와 다운로드 명세서에 정보가 흩어져 있고, 에이전트가 매번 외부 문서를 검색하면 토큰과 시간이 낭비된다.
- **Decision:** 현재 공개된 31개 API의 핵심 메타데이터를 바이너리에 내장하고, 상세 스펙은 `docs/reference.md`에 유지한다.
- **Consequences:** 바이너리 자체가 최소한의 자기 기술 스키마를 가지며, API 변경 시 카탈로그 업데이트가 필요하다.

### ADR-3: 입력 검증을 엄격하게 둔다

- **Status:** Accepted
- **Context:** 참고 글은 에이전트 입력을 신뢰하지 말고, 웹 API가 사용자 입력을 검증하듯 CLI도 검증해야 한다고 강조한다.
- **Decision:** 지원하지 않는 `api_id` 차단, `basDd` 형식 검증, JSON 파라미터 객체 강제, 미지원 필드 거부, 제어 문자 거부를 기본 동작으로 둔다.
- **Consequences:** 초기에는 유연성이 낮지만, 할루시네이션이나 잘못된 자동화 호출을 줄일 수 있다.

### ADR-4: 인증은 환경 변수 우선으로 둔다

- **Status:** Accepted
- **Context:** 브라우저 리디렉션이 필요한 인증 흐름은 CLI/에이전트 자동화에 맞지 않는다.
- **Decision:** 실제 서버 호출은 `--auth-key`, `KRX_API_KEY`, `~/.config/krx/config.json`의 `auth_key` 순서로 처리하고, 샘플 엔드포인트는 공개 샘플 키를 기본값으로 사용한다.
- **Consequences:** 로컬 자동화가 단순해지고, 사용자 문서에 키 발급 절차와 저장 위치를 분리해서 설명해야 한다.

### ADR-5: 설정 경로는 OS별 네이티브 경로 대신 `~/.config/krx`로 통일한다

- **Status:** Accepted
- **Context:** 사용자 요구사항은 macOS/Linux/Windows 모두 지원하되 설정 저장 경로를 일관되게 `~/.config/krx`로 유지하는 것이다.
- **Decision:** 홈 디렉터리를 OS별로 해석한 뒤, 그 아래 `.config/krx/config.json`을 공통 설정 경로로 사용한다.
- **Consequences:** 문서와 자동화 예제가 단순해진다. 대신 Windows 네이티브 `AppData` 경로를 따르지 않는 점을 명시해야 한다.

### ADR-6: MCP보다 library-first runtime을 먼저 고정한다

- **Status:** Accepted
- **Context:** 현재 read-only CLI 경로가 안정화되었지만, 바로 MCP를 열면 CLI, library, MCP가 동시에 진화해 표면이 흔들릴 수 있다.
- **Decision:** 먼저 clap 없는 runtime API로 요청 계획과 실행 경로를 고정하고, MCP는 그 공용 표면을 재사용하는 후속 작업으로 둔다.
- **Consequences:** 이번 단계 구현 범위는 작게 유지되고, 이후 MCP 작업은 CLI 로직 복제 없이 runtime API 어댑터에 집중할 수 있다.

### ADR-7: 실서버 비정상 응답은 모두 KRX 오류로 정규화한다

- **Status:** Accepted
- **Context:** KRX 실서버는 알려진 `401` 두 케이스 외에도 다른 상태 코드나 비정상 본문을 돌려줄 수 있다. 이를 그대로 통과시키면 CLI와 library 사용자가 일관된 오류 계약을 얻지 못한다.
- **Decision:** `401 Unauthorized Key`와 `401 Unauthorized API Call`은 전용 오류로 유지하고, 그 외 모든 non-2xx 응답은 `krx_api_error`로 정규화한다. JSON body의 `respCode`/`respMsg`가 있으면 메시지에 반영하고, 없으면 HTTP 상태만 담은 일반 오류로 반환한다.
- **Consequences:** read-only 호출 경로의 오류 계약이 안정화된다. 다만 `401` 외의 KRX 세부 메시지별 전용 enum은 당분간 늘리지 않는다.

### ADR-8: MCP는 `krx` 바이너리의 `mcp serve` 서브커맨드로 1단계를 연다

- **Status:** Accepted
- **Context:** runtime library 표면이 고정된 뒤에는 MCP를 붙일 수 있지만, 별도 바이너리나 별도 crate로 바로 확장하면 배포 표면과 결정 비용이 함께 커진다.
- **Decision:** phase 1 MCP는 별도 바이너리 없이 `krx mcp serve`로 제공한다. transport는 stdio, capability는 tools만 열고, tool 표면은 `krx_list_apis`, `krx_get_api_schema`, `krx_call_api` 세 가지로 제한한다. `krx_call_api`는 기존 CLI 검증과 runtime을 재사용하고 `dry_run`도 함께 노출한다. 프로토콜은 `2025-06-18`을 기본으로 협상하고, `2025-03-26`도 수용한다.
- **Consequences:** 사용자에게는 여전히 `krx` 하나만 배포하면 되고, MCP 어댑터는 CLI 로직 복제 없이 `krx-core` 위에 얹힌다. 대신 prompts/resources 같은 추가 MCP capability는 후속 단계로 남는다.

### ADR-9: 카탈로그 드리프트 감지는 deterministic parser test와 live check를 분리한다

- **Status:** Accepted
- **Context:** 내장 카탈로그는 런타임과 MCP 모두의 기반이라 upstream 서비스 목록 변경을 빨리 감지해야 한다. 다만 CI의 기본 테스트는 deterministic해야 하고, KRX 사이트 live HTML 파싱은 네트워크와 페이지 상태에 따라 흔들릴 수 있다.
- **Decision:** `krx-core`에 HTML 파서와 inventory diff 로직을 두고, 단위 테스트는 고정 fixture 문자열로 검증한다. 실제 upstream 확인은 `cargo run -p krx-core --example catalog_drift`와 `./scripts/check-catalog-drift.sh`로 분리하고, drift가 있으면 non-zero exit으로 실패시킨다. gating 기준은 `api_id`, `path`, `name`, `category`, 전체 개수에 둔다. `description`은 현재 내장 카탈로그가 더 짧게 정규화되어 있어 정보용으로만 유지하고 실패 조건에는 넣지 않는다.
- **Consequences:** 기본 `cargo test`는 안정적으로 유지되고, live drift check는 필요할 때나 스케줄드 CI에서 별도로 돌릴 수 있다. 새 서비스가 생기거나 `api_id/path/name/category/count`가 바뀌면 보고서로 드러난다.

## Key References

- KRX Open API 서비스 목록: https://openapi.krx.co.kr/contents/OPP/INFO/service/OPPINFO004.cmd
- KRX Open API 서비스 이용방법: https://openapi.krx.co.kr/contents/OPP/INFO/OPPINFO003.jsp
- 프로젝트 내 API 조사 문서: `docs/reference.md`
- GeekNews 요약: https://news.hada.io/topic?id=27246
- 원문: https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-agents/
- 설정 경로 원칙: `~/.config/krx/config.json`

## Prior Art

- Google Workspace CLI(`gws`)의 설계 원칙에서 차용할 포인트:
  - 명령 플래그보다 JSON 페이로드 입력 우선
  - 기계 판독 가능한 스키마 조회 기능
  - `--output json` 중심의 구조화 출력
  - `--dry-run`으로 실행 전 요청 확인
  - 에이전트 안전성 관점의 입력 검증
- KRX Open API 특성:
  - 현재 공개 서비스는 31개
  - 공개 HTML 기준 요청 방식은 `GET`
  - 모든 공개 API는 `basDd` 쿼리 파라미터를 사용
  - 샘플 엔드포인트와 실제 엔드포인트가 분리됨

## Open Questions

현재 열린 질문 없음
