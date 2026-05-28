use std::process::ExitCode;

use krx_core::drift::{
    built_in_inventory, compare_inventories, parse_detail_page, parse_service_list,
};
use reqwest::blocking::Client;

const SERVICE_LIST_URL: &str = "https://openapi.krx.co.kr/contents/OPP/INFO/service/OPPINFO004.cmd";

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            serde_json::to_writer_pretty(std::io::stdout(), &report)
                .expect("drift report should serialize");
            println!();

            if report.has_drift() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<krx_core::drift::DriftReport, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("krx-cli-catalog-drift/0.1.0")
        .build()?;

    let list_html = client
        .get(SERVICE_LIST_URL)
        .send()?
        .error_for_status()?
        .text()?;
    let services = parse_service_list(&list_html)?;

    let mut upstream = Vec::with_capacity(services.len());
    for service in services {
        let detail_html = client
            .get(&service.detail_url)
            .send()?
            .error_for_status()?
            .text()?;
        upstream.push(parse_detail_page(&service.category, &detail_html)?);
    }
    upstream.sort();

    Ok(compare_inventories(&built_in_inventory(), &upstream))
}
