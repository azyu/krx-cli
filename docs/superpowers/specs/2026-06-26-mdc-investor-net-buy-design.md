# MDC Investor Net Buy CLI Design

## Goal

Add an explicitly experimental CLI path for KRX MDCSTAT024, `투자자별 순매수상위종목`, without changing the existing Open API catalog or the stable `call` command.

The target command is:

```bash
krw experimental mdc investor-net-buy \
  --date 20260626 \
  --market ALL \
  --investor foreigner \
  --limit 50 \
  --output json
```

## Context

The existing CLI is built around KRX Open API endpoints under `https://data-dbg.krx.co.kr/svc/apis/{path}/{api_id}`. The saved `~/.config/krx/config.json` key works for that path.

MDCSTAT024 is a KRX Data Marketplace web statistics screen, not one of the current Open API catalog entries. Its identifiers are:

- `screenId`: `MDCSTAT024`
- `bld`: `dbms/MDC/STAT/standard/MDCSTAT02401`
- data endpoint candidate: `https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd`

A direct POST with session cookie and `AUTH_KEY` currently returns `LOGOUT`, so implementation must treat the MDC web path as a separate transport with explicit errors.

## Approach

Use a hybrid model:

1. Keep stable Open API behavior unchanged.
2. Add `experimental mdc investor-net-buy` as a separate command tree.
3. Keep MDC request planning and error mapping in a new module, separate from `client.rs`.
4. Implement request planning and deterministic tests before attempting live network behavior.

This avoids mixing a brittle web-screen integration into the stable schema catalog.

## CLI Contract

Command:

```bash
krw experimental mdc investor-net-buy [OPTIONS]
```

Options:

- `--date YYYYMMDD`: required trade date. Used for both start and end date in the initial implementation.
- `--market <ALL|STK|KSQ>`: defaults to `ALL`.
- `--investor <foreigner|institution|individual|other-corp>`: required.
- `--limit N`: optional client-side output limit after parsing.
- `--dry-run`: prints the planned MDC request without network execution.
- global `--output json|text` continues to apply.

Code mappings:

| CLI value | KRX code |
| --- | --- |
| `foreigner` | `9000` |
| `institution` | `7050` |
| `individual` | `8000` |
| `other-corp` | `7100` |

Market mappings:

| CLI value | KRX code |
| --- | --- |
| `ALL` | `ALL` |
| `STK` | `STK` |
| `KSQ` | `KSQ` |

## Module Boundaries

Add `src/mdc.rs` with:

- `MdcInvestorNetBuyArgs`: normalized command inputs.
- `MdcRequestPlan`: URL, method, headers summary, and form body.
- `build_investor_net_buy_plan(args)`: deterministic planner.
- `execute_mdc_request(plan)`: live HTTP execution.
- `map_mdc_error(status, body)`: maps `LOGOUT`, empty body, non-JSON, and HTTP failures to user-facing errors.

The existing `client.rs` remains responsible only for Open API calls.

## Request Flow

Initial live implementation attempts this sequence:

1. GET `https://data.krx.co.kr/contents/MDC/MDI/mdiLoader/index.cmd?screenId=MDCSTAT024` to obtain cookies.
2. POST form data to `https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd`.
3. Include browser-like headers only where required by evidence: `User-Agent`, `Referer`, `X-Requested-With`, and `Content-Type`.
4. If KRX returns `LOGOUT`, return a specific MDC session error rather than a generic HTTP error.

If evidence shows a required OTP endpoint, add it as a focused step in `execute_mdc_request`; do not introduce browser automation.

## Output Shape

For JSON output, preserve the KRX row fields when possible and wrap metadata only when needed for errors or dry-run.

Expected row fields include:

- 종목코드
- 종목명
- 매도 거래량
- 매수 거래량
- 순매수 거래량
- 매도 거래대금
- 매수 거래대금
- 순매수 거래대금

Field names in the live response must be discovered from actual KRX JSON before adding any renaming layer. The first implementation may output raw KRX keys if they are stable and documented by test fixtures.

## Error Handling

Specific errors:

- Invalid market or investor value: CLI validation error.
- `LOGOUT`: `MDC session rejected; KRX web statistics session flow changed or requires an additional token`.
- Empty JSON data: successful response with empty rows, unless body indicates an error.
- Non-JSON body: include status and a short body preview.
- Network failure: preserve the underlying request error.

## Testing Plan

Follow TDD:

1. Test investor and market code mapping.
2. Test dry-run request plan contains the correct `bld`, `mktId`, `invstTpCd`, `strtDd`, and `endDd`.
3. Test `LOGOUT` maps to the specific MDC session error.
4. Test limit handling on parsed rows using a fixture.
5. Run the targeted Rust tests.
6. Run one live CLI command against KRX and report the observed result.

## Non-goals

- Do not add MDCSTAT024 to `schema list`.
- Do not replace Open API calls with MDC calls.
- Do not add pykrx, cloudscraper, Selenium, or browser automation.
- Do not generalize all MDC screens before this one screen works.
- Do not silently fall back to stale or alternate data sources.
