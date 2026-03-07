# krx-cli

한국거래소(KRX) Open API용 Rust CLI입니다. 공식 바이너리 이름은 `krw`이며, 터미널에서 공개된 KRX 읽기 API를 조회하고, 내장 스키마 카탈로그로 요청 형태를 확인할 수 있습니다.

현재 범위는 읽기 전용 CLI입니다. 공개된 31개 API 메타데이터를 내장하고, `schema` 조회, 샘플/실서버 호출, 구조화 출력, 엄격한 입력 검증, `--dry-run`, `--body-only`, `--fields`를 지원합니다. 저장소는 `krx-cli`와 `krx-core` 두 crate로 나뉘며, clap 없이 정규화된 query map으로 호출할 수 있는 library-first runtime 표면은 `krx-core`가 제공합니다.

> [!IMPORTANT]
> 샘플 호출은 공개 샘플 키로 바로 실행할 수 있지만, 실서버 호출은 KRX 포털에서 발급받고 승인된 인증키가 필요합니다.

## 설치

### Homebrew

```bash
brew tap <owner>/<tap>
brew install <owner>/<tap>/krw
krw --help
```

Homebrew tap 이름은 배포자가 공개한 tap 저장소를 사용해야 합니다. 이 저장소 자체만으로는 tap 이름이 고정되지 않으므로, 실제 배포 시점에는 release 안내나 프로젝트 문서에 적힌 `<owner>/<tap>` 값을 넣으면 됩니다. 현재 Homebrew 배포 타깃은 `linux amd64`, `linux arm64`, `macOS arm64`입니다.

### GitHub Release 아카이브

Rust 없이 설치하려면 [GitHub Releases](https://github.com/azyu/krx-cli/releases)에서 현재 OS/아키텍처에 맞는 아카이브를 내려받아 `krw` 바이너리만 설치하면 됩니다. 현재 릴리스 타깃은 `linux amd64`, `linux arm64`, `macOS arm64`, `Windows x64`, `Windows arm64`입니다.

```bash
tar -xzf krw_<version>_<platform>_<arch>.tar.gz
install -m 755 krw ~/.local/bin/krw
krw --help
```

Windows PowerShell 예시:

```powershell
Expand-Archive krw_<version>_windows_amd64.zip .
.\krw.exe --help
```

### From source

Rust toolchain이 준비되어 있다면 release 바이너리를 바로 빌드할 수 있습니다.

```bash
cargo build --release -p krx-cli --bin krw
install -m 755 target/release/krw ~/.local/bin/krw
krw --help
```

저장소에 포함된 설치 스크립트를 써도 됩니다.

```bash
./scripts/install-release.sh
~/.local/bin/krw --help
```

설치 없이 바로 실행하려면:

```bash
cargo run -p krx-cli -- --output json schema list
```

## 설정

### 1. KRX Open API 인증키 발급

[KRX Open API 서비스 이용 안내](https://openapi.krx.co.kr/contents/OPP/INFO/OPPINFO003.jsp)를 참고해 인증키를 발급받고, 필요한 API 이용신청을 완료합니다.

### 2. 설정 파일 저장

```bash
cargo run -p krx-cli -- config path
cargo run -p krx-cli -- config set-auth-key YOUR_ISSUED_KEY
cargo run -p krx-cli -- config show
```

설정 파일 경로는 모든 OS에서 홈 디렉터리 기준 `~/.config/krx/config.json`입니다.

`~/.config/krx/config.json`:

```json
{
  "auth_key": "발급받은 인증키"
}
```

### 3. 환경변수 대안

```bash
export KRX_API_KEY="발급받은 인증키"
krw call krx_dd_trd --date 20240131
```

실서버 인증키 우선순위는 `--auth-key` > `KRX_API_KEY` > `~/.config/krx/config.json`입니다.

### 4. 샘플 호출로 동작 확인

```bash
krw schema show krx_dd_trd
krw call krx_dd_trd --date 20200414 --sample
krw --output json call krx_dd_trd --date 20200414 --sample --body-only
krw --output json call krx_dd_trd --date 20200414 --sample --body-only --fields BAS_DD,IDX_NM
krw --output json call krx_dd_trd --date 20200414 --sample --fields BAS_DD,IDX_NM
```

## 사용 예시

### 스키마 조회

```bash
krw schema list
krw schema show krx_dd_trd
krw --output json schema show krx_dd_trd
```

### 요청 계획 확인

```bash
krw --output json call krx_dd_trd --date 20200414 --sample --dry-run
krw --output json call krx_dd_trd --params '{"basDd":"20200414"}' --sample --dry-run
```

### 샘플 / 실서버 호출

```bash
krw call krx_dd_trd --date 20200414 --sample
krw --output json call krx_dd_trd --date 20200414 --sample
krw --output json call krx_dd_trd --date 20240131
krw call krx_dd_trd --date 20200414 --sample --format xml
```

### 설정 확인 / 정리

```bash
krw config path
krw config show
krw config clear-auth-key
```

## 지원 표면

- `schema`: 지원 API 목록 조회와 API별 요청/응답 스키마 출력
- `call`: 샘플/실서버 GET 호출, `--date` 또는 `--params` 입력, `--dry-run`, `--format json|xml`, `--body-only`, `--fields`
- `config`: 설정 경로 확인, 저장된 인증키 확인, 인증키 저장/삭제
- `krx-core` library surface: 정규화된 query map과 선택 필드로 `plan_call` / `execute_call` 가능, MCP는 이 표면 위에 나중에 얹을 예정
- 내장 카탈로그: 지수, 주식, 증권상품, 채권, 파생상품, 일반상품, ESG까지 공개된 31개 API 메타데이터 포함
- 구조화 출력: `--output json`으로 기계 친화적 출력 제공, 실패 시에도 `{ "error": { "code", "message" } }` 계약을 stdout에 유지, `schema show`에는 `output_field_names` 포함
- 입력 검증: 미지원 `api_id`, 잘못된 `basDd`, 알 수 없는 query field, 제어 문자, 예약 URL 문자 거부

## 글로벌 플래그

| 플래그 | 설명 |
|--------|------|
| `--output <text\|json>` | 출력 모드. 터미널에서는 기본적으로 text, 파이프/리다이렉션 시에는 json |

## 호출 플래그

| 플래그 | 설명 |
|--------|------|
| `--sample` | 샘플 엔드포인트와 공개 샘플 키 사용 |
| `--date <YYYYMMDD>` | 현재 공개 스키마의 공통 파라미터 `basDd` 단축 입력 |
| `--params '{"basDd":"..."}'` | JSON 객체로 query 파라미터 전달 |
| `--format <json\|xml>` | KRX 응답 포맷 선택 |
| `--auth-key <key>` | 실서버 호출용 인증키 직접 지정 |
| `--dry-run` | 실제 호출 없이 요청 URL, 메서드, 마스킹된 키, query만 출력 |
| `--body-only` | `--output json`일 때 응답 envelope 없이 body만 출력 |
| `--fields <FIELD,...>` | `--output json --format json`일 때 JSON body 안의 row 필드 일부만 유지. `--dry-run`과 함께 쓸 수 없음 |

> [!NOTE]
> 현재 공개 스키마에서는 모든 API가 `basDd`를 사용합니다. `--date`와 `--params`는 함께 쓸 수 없고, 미지원 필드는 허용하지 않습니다.

> [!NOTE]
> `--fields`는 API 카탈로그에 등록된 `output_field_names`만 허용합니다. `OutBlock_1` 같은 최상위 컨테이너는 유지한 채 body 안의 row 객체만 줄이며, envelope 모드와 `--body-only` 모두에 먼저 적용됩니다.

> [!NOTE]
> 실서버 `401` 응답은 `Unauthorized Key`와 `Unauthorized API Call`을 구분해서 안내합니다. 키 자체가 잘못된 경우와 API 이용신청이 아직 승인되지 않은 경우를 서로 다른 메시지로 보여줍니다.

## 프로젝트 구조

```text
krx-cli/
├── crates/
│   ├── cli/
│   │   └── src/
│   │       ├── main.rs    # 바이너리 엔트리포인트
│   │       ├── app.rs     # 명령 디스패치와 출력 모드 처리
│   │       ├── cli.rs     # clap 기반 CLI 정의
│   │       └── output.rs  # JSON/text 출력 헬퍼
│   └── core/
│       └── src/
│           ├── catalog.rs # 내장 KRX API 카탈로그와 스키마 뷰
│           ├── client.rs  # 파라미터 검증, 요청 계획, HTTP 호출
│           ├── runtime.rs # clap 없이 재사용 가능한 read-only runtime 표면
│           ├── config.rs  # ~/.config/krx/config.json 관리
│           └── error.rs   # 사용자 대상 오류 타입
└── docs/reference.md      # 공개 API 조사 문서
```

자세한 API 인벤토리는 [`docs/reference.md`](docs/reference.md), 설계 근거와 참고 자료는 [`docs/references.md`](docs/references.md)에 정리되어 있습니다.

> [!NOTE]
> MCP 서버/어댑터는 아직 구현하지 않습니다. 먼저 `krx-core` runtime library 표면을 고정하고, 이후 단계에서 그 위에 얹는 순서를 따릅니다.

## 테스트

```bash
cargo fmt --all
cargo test
cargo run -p krx-cli -- --output json schema show krx_dd_trd
cargo run -p krx-cli -- --output json call krx_dd_trd --date 20200414 --sample --dry-run
```

실서버 호출까지 확인하려면 승인된 인증키를 준비한 뒤 아래 명령을 사용합니다.

```bash
krw --output json call krx_dd_trd --date 20240131
```
