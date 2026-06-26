# MDC Investor Net Buy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `krx experimental mdc investor-net-buy` for KRX MDCSTAT024 while keeping the existing Open API `call` and `schema` paths unchanged.

**Architecture:** The stable Open API client remains in `src/client.rs`. A new `src/mdc.rs` owns MDC request planning, code mappings, live HTTP execution, error mapping, and JSON row limiting. `src/cli.rs` only adds the experimental command tree; `src/lib.rs` wires dry-run and execution output.

**Tech Stack:** Rust 2024, `clap` derive/value enums, blocking `reqwest` with rustls, `serde`, `serde_json`, `thiserror`.

## Global Constraints

- Keep stable Open API behavior unchanged.
- Do not add MDCSTAT024 to `schema list`.
- Do not add pykrx, cloudscraper, Selenium, or browser automation.
- Do not generalize all MDC screens before this one screen works.
- Do not silently fall back to stale or alternate data sources.
- Follow TDD: write each failing test and run it before implementation.

---

## File Structure

- Modify `src/lib.rs`: export `mdc`, dispatch `Commands::Experimental`, render MDC dry-runs, and print MDC response JSON/text.
- Modify `src/cli.rs`: add `Experimental -> Mdc -> InvestorNetBuy` command tree plus `MdcInvestorNetBuyArgs`.
- Create `src/mdc.rs`: code mappings, request planner, response executor, error mapping, row limiting.
- Modify `src/error.rs`: add explicit MDC session/unexpected response errors.
- Use existing unit test modules in `src/cli.rs`, `src/lib.rs`, and new `src/mdc.rs`; do not create a separate `tests/` directory unless command-level integration tests are needed later.

---

### Task 1: MDC request planner and mappings

**Files:**
- Create: `src/mdc.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `pub enum MdcMarket`, `pub enum MdcInvestor`, `pub struct MdcRequestPlan`, `pub fn build_investor_net_buy_plan(args: &crate::cli::MdcInvestorNetBuyArgs) -> MdcRequestPlan`.
- Consumes later: `src/cli.rs` will use `MdcMarket` and `MdcInvestor` as `clap::ValueEnum` argument types.

- [ ] **Step 1: Write the failing planner tests**

Add this new file `src/mdc.rs` with tests first:

```rust
use std::collections::BTreeMap;

use clap::ValueEnum;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MdcMarket {
    All,
    Stk,
    Ksq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum MdcInvestor {
    Foreigner,
    Institution,
    Individual,
    OtherCorp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_values_map_to_krx_codes() {
        assert_eq!(MdcMarket::All.krx_code(), "ALL");
        assert_eq!(MdcMarket::Stk.krx_code(), "STK");
        assert_eq!(MdcMarket::Ksq.krx_code(), "KSQ");
    }

    #[test]
    fn investor_values_map_to_krx_codes() {
        assert_eq!(MdcInvestor::Foreigner.krx_code(), "9000");
        assert_eq!(MdcInvestor::Institution.krx_code(), "7050");
        assert_eq!(MdcInvestor::Individual.krx_code(), "8000");
        assert_eq!(MdcInvestor::OtherCorp.krx_code(), "7100");
    }
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```bash
cargo test mdc::tests::market_values_map_to_krx_codes mdc::tests::investor_values_map_to_krx_codes
```

Expected: compilation fails because `pub mod mdc;` is not exported from `src/lib.rs`, or because `krx_code` methods are missing after adding the module export. Either failure is the expected missing-feature failure.

- [ ] **Step 3: Export module and implement mappings**

Add to `src/lib.rs` near the other `pub mod` declarations:

```rust
pub mod mdc;
```

Add these implementations above the test module in `src/mdc.rs`:

```rust
impl MdcMarket {
    pub fn krx_code(self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::Stk => "STK",
            Self::Ksq => "KSQ",
        }
    }
}

impl MdcInvestor {
    pub fn krx_code(self) -> &'static str {
        match self {
            Self::Foreigner => "9000",
            Self::Institution => "7050",
            Self::Individual => "8000",
            Self::OtherCorp => "7100",
        }
    }
}
```

- [ ] **Step 4: Run tests and verify GREEN**

Run:

```bash
cargo test mdc::tests::market_values_map_to_krx_codes mdc::tests::investor_values_map_to_krx_codes
```

Expected: both tests pass.

- [ ] **Step 5: Add request-plan failing test**

Append this test to `src/mdc.rs`:

```rust
    #[test]
    fn investor_net_buy_plan_contains_mdcstat024_form_fields() {
        let args = crate::cli::MdcInvestorNetBuyArgs {
            date: "20260626".to_string(),
            market: MdcMarket::All,
            investor: MdcInvestor::Foreigner,
            limit: Some(50),
            dry_run: true,
        };

        let plan = build_investor_net_buy_plan(&args);

        assert_eq!(plan.method, "POST");
        assert_eq!(plan.url, MDC_JSON_URL);
        assert_eq!(plan.form["bld"], MDCSTAT024_BLD);
        assert_eq!(plan.form["mktId"], "ALL");
        assert_eq!(plan.form["invstTpCd"], "9000");
        assert_eq!(plan.form["strtDd"], "20260626");
        assert_eq!(plan.form["endDd"], "20260626");
        assert_eq!(plan.form["share"], "1");
        assert_eq!(plan.form["money"], "1");
    }
```

- [ ] **Step 6: Run test and verify RED**

Run:

```bash
cargo test mdc::tests::investor_net_buy_plan_contains_mdcstat024_form_fields
```

Expected: compilation fails because `MdcInvestorNetBuyArgs`, constants, `MdcRequestPlan`, or `build_investor_net_buy_plan` do not exist yet.

- [ ] **Step 7: Implement request-plan types**

Add these items to `src/mdc.rs` above the mapping impls:

```rust
pub const MDC_LOADER_URL: &str =
    "https://data.krx.co.kr/contents/MDC/MDI/mdiLoader/index.cmd?screenId=MDCSTAT024";
pub const MDC_JSON_URL: &str = "https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd";
pub const MDCSTAT024_BLD: &str = "dbms/MDC/STAT/standard/MDCSTAT02401";

#[derive(Debug, Clone, Serialize)]
pub struct MdcRequestPlan {
    pub screen_id: &'static str,
    pub bld: &'static str,
    pub url: &'static str,
    pub method: &'static str,
    pub form: BTreeMap<String, String>,
}

pub fn build_investor_net_buy_plan(args: &crate::cli::MdcInvestorNetBuyArgs) -> MdcRequestPlan {
    let mut form = BTreeMap::new();
    form.insert("bld".to_string(), MDCSTAT024_BLD.to_string());
    form.insert("locale".to_string(), "ko_KR".to_string());
    form.insert("mktId".to_string(), args.market.krx_code().to_string());
    form.insert("invstTpCd".to_string(), args.investor.krx_code().to_string());
    form.insert("strtDd".to_string(), args.date.clone());
    form.insert("endDd".to_string(), args.date.clone());
    form.insert("share".to_string(), "1".to_string());
    form.insert("money".to_string(), "1".to_string());
    form.insert("csvxls_isNo".to_string(), "false".to_string());

    MdcRequestPlan {
        screen_id: "MDCSTAT024",
        bld: MDCSTAT024_BLD,
        url: MDC_JSON_URL,
        method: "POST",
        form,
    }
}
```

- [ ] **Step 8: Add the CLI args struct needed by the planner**

In `src/cli.rs`, add imports and a temporary args struct. It is okay if the command is not wired yet; Task 2 wires parsing.

```rust
use crate::mdc::{MdcInvestor, MdcMarket};
```

Add after `CallArgs`:

```rust
#[derive(Debug, Args)]
pub struct MdcInvestorNetBuyArgs {
    #[arg(long, help = "Trade date in YYYYMMDD format")]
    pub date: String,

    #[arg(long, default_value_t = MdcMarket::All, value_enum)]
    pub market: MdcMarket,

    #[arg(long, value_enum)]
    pub investor: MdcInvestor,

    #[arg(long, help = "Client-side maximum row count")]
    pub limit: Option<usize>,

    #[arg(long, help = "Validate and render the MDC request without calling KRX")]
    pub dry_run: bool,
}
```

- [ ] **Step 9: Run request-plan test and verify GREEN**

Run:

```bash
cargo test mdc::tests::investor_net_buy_plan_contains_mdcstat024_form_fields
```

Expected: test passes.

- [ ] **Step 10: Commit Task 1**

Run:

```bash
git add src/lib.rs src/cli.rs src/mdc.rs
git commit -m "feat: add MDC request planner"
```

---

### Task 2: Experimental CLI dry-run wiring

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `mdc::build_investor_net_buy_plan` from Task 1.
- Produces: `Commands::Experimental { command: ExperimentalCommands }` dispatch and dry-run rendering.

- [ ] **Step 1: Write failing CLI parse test**

Append this test to the existing `#[cfg(test)] mod tests` in `src/cli.rs`:

```rust
    #[test]
    fn experimental_mdc_investor_net_buy_parses() {
        let cli = Cli::try_parse_from([
            "krx",
            "experimental",
            "mdc",
            "investor-net-buy",
            "--date",
            "20260626",
            "--market",
            "ALL",
            "--investor",
            "foreigner",
            "--limit",
            "10",
            "--dry-run",
        ])
        .unwrap();

        match cli.command {
            Commands::Experimental { command } => match command {
                ExperimentalCommands::Mdc { command } => match command {
                    MdcCommands::InvestorNetBuy(args) => {
                        assert_eq!(args.date, "20260626");
                        assert_eq!(args.market, MdcMarket::All);
                        assert_eq!(args.investor, MdcInvestor::Foreigner);
                        assert_eq!(args.limit, Some(10));
                        assert!(args.dry_run);
                    }
                },
            },
            _ => panic!("expected experimental mdc investor-net-buy command"),
        }
    }
```

- [ ] **Step 2: Run parse test and verify RED**

Run:

```bash
cargo test cli::tests::experimental_mdc_investor_net_buy_parses
```

Expected: compilation fails because `ExperimentalCommands` and `MdcCommands` do not exist.

- [ ] **Step 3: Add command enums**

Modify `src/cli.rs`.

Add a variant to `Commands`:

```rust
    Experimental {
        #[command(subcommand)]
        command: ExperimentalCommands,
    },
```

Add enum definitions after `SchemaCommands`:

```rust
#[derive(Debug, Subcommand)]
pub enum ExperimentalCommands {
    Mdc {
        #[command(subcommand)]
        command: MdcCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum MdcCommands {
    InvestorNetBuy(MdcInvestorNetBuyArgs),
}
```

- [ ] **Step 4: Run parse test and verify GREEN**

Run:

```bash
cargo test cli::tests::experimental_mdc_investor_net_buy_parses
```

Expected: test passes.

- [ ] **Step 5: Add dry-run dispatch test**

In `src/lib.rs`, add a small renderer test rather than spawning the binary.

Add these structs near `DryRunEnvelope`:

```rust
#[derive(Debug, Serialize)]
struct MdcDryRunEnvelope {
    mode: &'static str,
    plan: mdc::MdcRequestPlan,
}
```

Append this test to `src/lib.rs` tests:

```rust
    #[test]
    fn render_mdc_dry_run_includes_form_fields() {
        let args = crate::cli::MdcInvestorNetBuyArgs {
            date: "20260626".to_string(),
            market: crate::mdc::MdcMarket::All,
            investor: crate::mdc::MdcInvestor::Foreigner,
            limit: Some(10),
            dry_run: true,
        };
        let plan = crate::mdc::build_investor_net_buy_plan(&args);

        let rendered = render_mdc_dry_run(&plan);

        assert!(rendered.contains("POST https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd"));
        assert!(rendered.contains("dbms/MDC/STAT/standard/MDCSTAT02401"));
        assert!(rendered.contains("invstTpCd"));
        assert!(rendered.contains("9000"));
    }
```

- [ ] **Step 6: Run renderer test and verify RED**

Run:

```bash
cargo test lib::tests::render_mdc_dry_run_includes_form_fields
```

Expected: compilation fails because `render_mdc_dry_run` does not exist.

- [ ] **Step 7: Implement dry-run renderer and dispatch**

Modify `src/lib.rs` imports:

```rust
use crate::cli::{
    Cli, Commands, ConfigCommands, ExperimentalCommands, MdcCommands, OutputMode, SchemaCommands,
};
use crate::mdc::build_investor_net_buy_plan;
```

Add dispatch arm inside `match cli.command` after `Commands::Call(args) => { ... }` with correct comma separation:

```rust
        Commands::Experimental { command } => match command {
            ExperimentalCommands::Mdc { command } => match command {
                MdcCommands::InvestorNetBuy(args) => {
                    let plan = build_investor_net_buy_plan(&args);

                    if args.dry_run {
                        let envelope = MdcDryRunEnvelope {
                            mode: "dry-run",
                            plan,
                        };

                        match output_mode {
                            OutputMode::Json => print_json(&envelope)?,
                            OutputMode::Text => print_text(&render_mdc_dry_run(&envelope.plan))?,
                        }

                        return Ok(());
                    }

                    let response = mdc::execute_mdc_request(&plan, args.limit)?;
                    match output_mode {
                        OutputMode::Json => print_json(&response.body)?,
                        OutputMode::Text => print_text(&response.body.to_string())?,
                    }
                }
            },
        },
```

Add renderer near `render_dry_run`:

```rust
fn render_mdc_dry_run(plan: &mdc::MdcRequestPlan) -> String {
    format!(
        "{method} {url}\nform: {form}",
        method = plan.method,
        url = plan.url,
        form = serde_json::to_string_pretty(&plan.form).unwrap_or_else(|_| "{}".to_string())
    )
}
```

Add a temporary stub to `src/mdc.rs` so dispatch compiles until Task 3 replaces it:

```rust
#[derive(Debug, Serialize)]
pub struct MdcResponseEnvelope {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: serde_json::Value,
}

pub fn execute_mdc_request(_plan: &MdcRequestPlan, _limit: Option<usize>) -> crate::error::Result<MdcResponseEnvelope> {
    Err(crate::error::KrxCliError::InvalidInput(
        "MDC live execution is not implemented yet".to_string(),
    ))
}
```

- [ ] **Step 8: Run dry-run tests and verify GREEN**

Run:

```bash
cargo test cli::tests::experimental_mdc_investor_net_buy_parses lib::tests::render_mdc_dry_run_includes_form_fields
```

Expected: both tests pass.

- [ ] **Step 9: Verify dry-run command manually**

Run:

```bash
cargo run --quiet -- experimental mdc investor-net-buy --date 20260626 --market ALL --investor foreigner --limit 10 --dry-run
```

Expected text includes `POST https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd`, `mktId`, `ALL`, `invstTpCd`, and `9000`.

- [ ] **Step 10: Commit Task 2**

Run:

```bash
git add src/cli.rs src/lib.rs src/mdc.rs
git commit -m "feat: wire experimental MDC dry run"
```

---

### Task 3: MDC error mapping and row limiting

**Files:**
- Modify: `src/error.rs`
- Modify: `src/mdc.rs`

**Interfaces:**
- Consumes: `MdcRequestPlan` from Task 1.
- Produces: `pub fn map_mdc_error(status: u16, body: &str) -> Option<KrxCliError>` and `pub fn limit_json_rows(body: serde_json::Value, limit: Option<usize>) -> serde_json::Value`.

- [ ] **Step 1: Write failing error mapping test**

Append to `src/mdc.rs` tests:

```rust
    #[test]
    fn logout_body_maps_to_mdc_session_error() {
        let error = map_mdc_error(400, "LOGOUT").expect("LOGOUT should map to MDC error");

        assert_eq!(
            error.to_string(),
            "MDC session rejected; KRX web statistics session flow changed or requires an additional token"
        );
    }
```

- [ ] **Step 2: Run test and verify RED**

Run:

```bash
cargo test mdc::tests::logout_body_maps_to_mdc_session_error
```

Expected: compilation fails because `map_mdc_error` or the error variant does not exist.

- [ ] **Step 3: Add MDC error variants**

Modify `src/error.rs` inside `KrxCliError`:

```rust
    #[error("MDC session rejected; KRX web statistics session flow changed or requires an additional token")]
    MdcSessionRejected,

    #[error("MDC returned an unexpected response: {0}")]
    MdcUnexpectedResponse(String),
```

- [ ] **Step 4: Implement error mapper**

Add to `src/mdc.rs` above tests:

```rust
use crate::error::{KrxCliError, Result};

pub fn map_mdc_error(status: u16, body: &str) -> Option<KrxCliError> {
    let trimmed = body.trim();
    if trimmed == "LOGOUT" {
        return Some(KrxCliError::MdcSessionRejected);
    }

    if status >= 400 {
        return Some(KrxCliError::MdcUnexpectedResponse(body_preview(trimmed)));
    }

    None
}

fn body_preview(body: &str) -> String {
    const MAX: usize = 200;
    if body.chars().count() <= MAX {
        return body.to_string();
    }
    body.chars().take(MAX).collect()
}
```

If `use crate::error::Result;` is unused before Task 4, keep only `KrxCliError` until `execute_mdc_request` uses `Result`.

- [ ] **Step 5: Run error test and verify GREEN**

Run:

```bash
cargo test mdc::tests::logout_body_maps_to_mdc_session_error
```

Expected: test passes.

- [ ] **Step 6: Write failing row-limit test**

Append to `src/mdc.rs` tests:

```rust
    #[test]
    fn limit_json_rows_truncates_top_level_arrays() {
        let body = serde_json::json!({
            "output": [
                {"ISU_CD":"005930"},
                {"ISU_CD":"000660"},
                {"ISU_CD":"035420"}
            ],
            "meta": {"keep": true}
        });

        let limited = limit_json_rows(body, Some(2));

        assert_eq!(limited["output"].as_array().unwrap().len(), 2);
        assert_eq!(limited["meta"]["keep"], true);
    }
```

- [ ] **Step 7: Run row-limit test and verify RED**

Run:

```bash
cargo test mdc::tests::limit_json_rows_truncates_top_level_arrays
```

Expected: compilation fails because `limit_json_rows` does not exist.

- [ ] **Step 8: Implement row limiting**

Add to `src/mdc.rs`:

```rust
pub fn limit_json_rows(mut body: serde_json::Value, limit: Option<usize>) -> serde_json::Value {
    let Some(limit) = limit else {
        return body;
    };

    match &mut body {
        serde_json::Value::Array(rows) => rows.truncate(limit),
        serde_json::Value::Object(object) => {
            for value in object.values_mut() {
                if let serde_json::Value::Array(rows) = value {
                    rows.truncate(limit);
                }
            }
        }
        _ => {}
    }

    body
}
```

- [ ] **Step 9: Run Task 3 tests and verify GREEN**

Run:

```bash
cargo test mdc::tests::logout_body_maps_to_mdc_session_error mdc::tests::limit_json_rows_truncates_top_level_arrays
```

Expected: both tests pass.

- [ ] **Step 10: Commit Task 3**

Run:

```bash
git add src/error.rs src/mdc.rs
git commit -m "feat: map MDC errors and limit rows"
```

---

### Task 4: Live MDC HTTP execution

**Files:**
- Modify: `src/mdc.rs`
- Modify: `src/lib.rs` only if Task 2 dispatch needs minor compile fixes.

**Interfaces:**
- Consumes: `MdcRequestPlan`, `map_mdc_error`, and `limit_json_rows`.
- Produces: `execute_mdc_request(plan, limit)` that performs the loader GET, extracts cookies, posts the MDC form, parses JSON, and maps `LOGOUT` clearly.

- [ ] **Step 1: Write failing cookie extraction test**

Add this helper test to `src/mdc.rs` tests:

```rust
    #[test]
    fn cookie_header_uses_name_value_pairs_from_set_cookie_headers() {
        let values = [
            "JSESSIONID=abc.def; Path=/; HttpOnly",
            "__smVisitorID=visitor; Expires=Sat, 26-Jun-2027 07:32:28 GMT; Path=/",
        ];

        assert_eq!(
            cookie_header_from_set_cookie_values(values.iter().copied()),
            Some("JSESSIONID=abc.def; __smVisitorID=visitor".to_string())
        );
    }
```

- [ ] **Step 2: Run cookie test and verify RED**

Run:

```bash
cargo test mdc::tests::cookie_header_uses_name_value_pairs_from_set_cookie_headers
```

Expected: compilation fails because `cookie_header_from_set_cookie_values` does not exist.

- [ ] **Step 3: Implement cookie extraction**

Add to `src/mdc.rs`:

```rust
fn cookie_header_from_set_cookie_values<'a>(values: impl Iterator<Item = &'a str>) -> Option<String> {
    let pairs = values
        .filter_map(|value| value.split(';').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    if pairs.is_empty() {
        None
    } else {
        Some(pairs.join("; "))
    }
}
```

- [ ] **Step 4: Run cookie test and verify GREEN**

Run:

```bash
cargo test mdc::tests::cookie_header_uses_name_value_pairs_from_set_cookie_headers
```

Expected: test passes.

- [ ] **Step 5: Replace execute stub with live implementation**

Replace the Task 2 stub in `src/mdc.rs` with:

```rust
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE, REFERER, SET_COOKIE, USER_AGENT};

pub fn execute_mdc_request(plan: &MdcRequestPlan, limit: Option<usize>) -> Result<MdcResponseEnvelope> {
    let client = Client::builder().build()?;

    let loader_response = client
        .get(MDC_LOADER_URL)
        .header(USER_AGENT, browser_user_agent())
        .send()?;

    let cookie_header = cookie_header_from_set_cookie_values(
        loader_response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok()),
    );

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(browser_user_agent()));
    headers.insert(REFERER, HeaderValue::from_static(MDC_LOADER_URL));
    headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
    );
    if let Some(cookie_header) = cookie_header {
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&cookie_header)
                .map_err(|_| KrxCliError::InvalidInput("invalid MDC cookie header".to_string()))?,
        );
    }

    let response = client.post(plan.url).headers(headers).form(&plan.form).send()?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body_text = response.text()?;

    if let Some(error) = map_mdc_error(status, &body_text) {
        return Err(error);
    }

    let body = serde_json::from_str(&body_text)
        .map_err(|_| KrxCliError::MdcUnexpectedResponse(body_preview(&body_text)))?;
    let body = limit_json_rows(body, limit);

    Ok(MdcResponseEnvelope {
        status,
        content_type,
        body,
    })
}

fn browser_user_agent() -> &'static str {
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0 Safari/537.36"
}
```

- [ ] **Step 6: Run targeted unit tests**

Run:

```bash
cargo test mdc::tests
```

Expected: all `mdc::tests` pass.

- [ ] **Step 7: Run all Rust tests**

Run:

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 8: Run dry-run CLI verification**

Run:

```bash
cargo run --quiet -- --output json experimental mdc investor-net-buy --date 20260626 --market ALL --investor foreigner --limit 10 --dry-run
```

Expected JSON includes:

```json
{
  "mode": "dry-run",
  "plan": {
    "screen_id": "MDCSTAT024",
    "bld": "dbms/MDC/STAT/standard/MDCSTAT02401",
    "method": "POST"
  }
}
```

- [ ] **Step 9: Run live CLI verification**

Run:

```bash
cargo run --quiet -- --output json experimental mdc investor-net-buy --date 20260626 --market ALL --investor foreigner --limit 10
```

Expected acceptable outcomes:

1. Success: JSON object with a top-level array containing no more than 10 rows.
2. Known MDC session rejection: command exits non-zero and prints `MDC session rejected; KRX web statistics session flow changed or requires an additional token`.

If outcome 2 occurs, do not hide it. Record it as the current verified transport blocker and investigate the additional token/session requirement as a separate follow-up before claiming live data retrieval works.

- [ ] **Step 10: Commit Task 4**

Run:

```bash
git add src/mdc.rs src/lib.rs
git commit -m "feat: execute MDC investor net buy request"
```

---

## Plan Self-Review

- Spec coverage: covered experimental command tree, separate module, dry-run, MDC web transport, `LOGOUT` mapping, row limiting, and live verification.
- Placeholder scan: no `TBD`, `TODO`, or unspecified edge handling remains.
- Type consistency: `MdcMarket`, `MdcInvestor`, `MdcInvestorNetBuyArgs`, `MdcRequestPlan`, `MdcResponseEnvelope`, `build_investor_net_buy_plan`, `map_mdc_error`, `limit_json_rows`, and `execute_mdc_request` are named consistently across tasks.
