# krx-cli

 Rust + `clap` 기반 KRX Open API CLI 스캐폴드입니다. 현재는 읽기 전용 API 호출, 내장 API 카탈로그, 구조화 출력, 입력 검증, `--dry-run`, 출력 필드 메타데이터, `--body-only`를 지원합니다.

설정 파일은 모든 OS에서 홈 디렉터리 기준 `~/.config/krx/config.json`에 저장합니다. Windows에서는 개념적으로 `%USERPROFILE%\\.config\\krx\\config.json`으로 해석됩니다.

## Quickstart

```bash
cargo run -- schema list
cargo run -- schema show krx_dd_trd
cargo run -- call krx_dd_trd --date 20200414 --sample
```

릴리즈 바이너리를 로컬 명령으로 설치하려면 아래 명령을 실행합니다.

```bash
./scripts/install-release.sh
~/.local/bin/krw --help
```

위 스크립트는 릴리즈 빌드 후 `~/.local/bin/krw`로 설치합니다.

발급 키를 저장해 두고 싶다면 다음 명령을 사용할 수 있습니다.

```bash
cargo run -- config path
cargo run -- config set-auth-key YOUR_ISSUED_KEY
cargo run -- config show
```

실서버 호출에는 발급된 인증키가 필요합니다.

```bash
cargo run -- call krx_dd_trd --date 20240131
```

## Commands

- `schema list`: 지원하는 API 목록을 출력합니다.
- `schema show <api-id>`: API 메타데이터와 요청 스키마를 출력합니다.
- `config path`: 설정 디렉터리와 설정 파일 경로를 출력합니다.
- `config show`: 현재 설정과 저장된 인증키 여부를 출력합니다.
- `config set-auth-key <key>`: `~/.config/krx/config.json`에 인증키를 저장합니다.
- `config clear-auth-key`: 저장된 인증키를 제거합니다.
- `call <api-id>`: API를 호출합니다.

핵심 옵션:

- `--output json`: 기계 판독 가능한 결과를 출력합니다.
- `--params '{"basDd":"20240131"}'`: JSON 객체로 요청 파라미터를 전달합니다.
- `--date 20240131`: 현재 공개 API 공통 파라미터를 단축 입력합니다.
- `--sample`: 샘플 엔드포인트와 공개 샘플 키를 사용합니다.
- `--dry-run`: 실제 호출 없이 요청 계획만 출력합니다.
- `--body-only`: `--output json`에서 envelope 없이 API body만 출력합니다.

`schema show`는 출력 필드 개수와 함께 실제 `output_field_names`도 제공합니다. 실서버 `401` 실패는 `Unauthorized Key`와 `Unauthorized API Call`을 구분해서 설명합니다.

## Design Notes

- 참고 글: “AI 에이전트를 위해선 CLI를 다시 작성해야 합니다”
- 설계 반영 사항:
- 구조화 출력을 제공합니다.
- 런타임 스키마 조회를 지원합니다.
- JSON 입력을 우선합니다.
- 입력을 검증합니다.
- 안전한 기본값을 유지합니다.
- OS별 동작 차이를 줄이기 위해 설정 경로는 공통적으로 `~/.config/krx`를 사용합니다.

자세한 조사 내용은 [docs/reference.md](/Volumes/EXTSSD/code/personal/krx-cli/docs/reference.md), 설계 근거는 [docs/references.md](/Volumes/EXTSSD/code/personal/krx-cli/docs/references.md)에 정리했습니다.
