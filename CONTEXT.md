# Agent Context

이 저장소의 CLI는 사람뿐 아니라 에이전트도 주요 사용자로 가정한다.

## Usage Rules

- 항상 `schema list` 또는 `schema show <api-id>`로 지원 범위를 먼저 확인한다.
- 가능하면 `--output json`을 사용한다.
- 처음 호출할 때는 `--dry-run`으로 요청 계획을 검증한다.
- 발급 키가 없으면 `--sample`로만 탐색한다.
- 설정 경로는 모든 OS에서 `~/.config/krx`로 통일한다.
- 실서버 키는 우선순위상 `--auth-key` > `KRX_API_KEY` > `~/.config/krx/config.json` 순서로 해석된다.
- URL을 직접 조합하지 말고 `api_id` 기반 명령만 사용한다.
- 현재 공개 API는 모두 `basDd`를 요구하므로 날짜를 명시적으로 전달한다.

## Safety Rules

- 미지원 `api_id`를 추측해서 사용하지 않는다.
- 파라미터는 `--params` JSON 객체 또는 `--date`로만 전달한다.
- 제어 문자, 쿼리 문자열 삽입, 이중 인코딩 같은 비정상 입력은 거부된다.
