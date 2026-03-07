# krx-cli

한국거래소(KRX) Open API용 Rust CLI입니다. 공식 바이너리 이름은 `krw`이며, 터미널에서 공개된 KRX 읽기 API를 조회하고, 내장 스키마 카탈로그로 요청 형태를 확인할 수 있습니다.

현재 범위는 읽기 전용 CLI입니다. 공개된 31개 API 메타데이터를 내장하고, `schema` 조회, 샘플/실서버 호출, 구조화 출력, 엄격한 입력 검증, `--dry-run`, `--body-only`를 지원합니다.

> [!IMPORTANT]
> 샘플 호출은 공개 샘플 키로 바로 실행할 수 있지만, 실서버 호출은 KRX 포털에서 발급받고 승인된 인증키가 필요합니다.

## 설치

Rust toolchain이 준비되어 있다면 release 바이너리를 바로 빌드할 수 있습니다.

```bash
cargo build --release --bin krw
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
cargo run -- --output json schema list
```

## 설정

### 1. KRX Open API 인증키 발급

[KRX Open API 서비스 이용 안내](https://openapi.krx.co.kr/contents/OPP/INFO/OPPINFO003.jsp)를 참고해 인증키를 발급받고, 필요한 API 이용신청을 완료합니다.

### 2. 설정 파일 저장

```bash
cargo run -- config path
cargo run -- config set-auth-key YOUR_ISSUED_KEY
cargo run -- config show
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
- `call`: 샘플/실서버 GET 호출, `--date` 또는 `--params` 입력, `--dry-run`, `--format json|xml`, `--body-only`
- `config`: 설정 경로 확인, 저장된 인증키 확인, 인증키 저장/삭제
- 내장 카탈로그: 지수, 주식, 증권상품, 채권, 파생상품, 일반상품, ESG까지 공개된 31개 API 메타데이터 포함
- 구조화 출력: `--output json`으로 기계 친화적 출력 제공, `schema show`에는 `output_field_names` 포함
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

> [!NOTE]
> 현재 공개 스키마에서는 모든 API가 `basDd`를 사용합니다. `--date`와 `--params`는 함께 쓸 수 없고, 미지원 필드는 허용하지 않습니다.

> [!NOTE]
> 실서버 `401` 응답은 `Unauthorized Key`와 `Unauthorized API Call`을 구분해서 안내합니다. 키 자체가 잘못된 경우와 API 이용신청이 아직 승인되지 않은 경우를 서로 다른 메시지로 보여줍니다.

## 프로젝트 구조

```text
krx-cli/
├── src/main.rs        # 바이너리 엔트리포인트
├── src/lib.rs         # 명령 디스패치와 출력 모드 처리
├── src/cli.rs         # clap 기반 CLI 정의
├── src/catalog.rs     # 내장 KRX API 카탈로그와 스키마 뷰
├── src/client.rs      # 파라미터 검증, 요청 계획, HTTP 호출
├── src/config.rs      # ~/.config/krx/config.json 관리
├── src/error.rs       # 사용자 대상 오류 타입
├── src/output.rs      # JSON/text 출력 헬퍼
└── docs/reference.md  # 공개 API 조사 문서
```

자세한 API 인벤토리는 [`docs/reference.md`](docs/reference.md), 설계 근거와 참고 자료는 [`docs/references.md`](docs/references.md)에 정리되어 있습니다.

## 테스트

```bash
cargo fmt --all
cargo test
cargo run -- --output json schema show krx_dd_trd
cargo run -- --output json call krx_dd_trd --date 20200414 --sample --dry-run
```

실서버 호출까지 확인하려면 승인된 인증키를 준비한 뒤 아래 명령을 사용합니다.

```bash
krw --output json call krx_dd_trd --date 20240131
```
