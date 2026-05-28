use krx_core::drift::{
    ApiInventoryEntry, compare_inventories, parse_detail_page, parse_service_list,
};

#[test]
fn parse_service_list_extracts_category_and_detail_links() {
    let html = r#"
<div class="tableWrap">
  <table>
    <tbody>
      <tr>
        <td rowspan="2" class="aC">지수</td>
        <td><a href="/contents/OPP/USES/service/OPPUSES001_S2.cmd?BO_ID=AAA" class="link">KRX 시리즈 일별시세정보</a></td>
        <td>KRX 시리즈 지수의 시세정보 제공</td>
      </tr>
      <tr>
        <td><a href="/contents/OPP/USES/service/OPPUSES001_S2.cmd?BO_ID=BBB" class="link">KOSPI 시리즈 일별시세정보</a></td>
        <td>KOSPI 시리즈 지수의 시세정보 제공</td>
      </tr>
      <tr>
        <td rowspan="1" class="aC">주식</td>
        <td><a href="/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=CCC" class="link">유가증권 일별매매정보</a></td>
        <td>유가증권시장에 상장되어 있는 주권의 매매정보 제공</td>
      </tr>
    </tbody>
  </table>
</div>
"#;

    let entries = parse_service_list(html).expect("service list should parse");

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].category, "지수");
    assert_eq!(
        entries[0].detail_url,
        "https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES001_S2.cmd?BO_ID=AAA"
    );
    assert_eq!(entries[1].category, "지수");
    assert_eq!(entries[2].category, "주식");
}

#[test]
fn parse_detail_page_extracts_inventory_fields() {
    let html = r#"
<form>
  <input type="hidden" name="path" value="idx" />
  <div class="tableWrap">
    <dl class="col2">
      <dt class="bdT">API 명</dt>
      <dd class="bdT point1">KRX 시리즈 일별시세정보</dd>
      <dt>최근 수정일</dt>
      <dd>2026/01/16</dd>
      <dt>설명</dt>
      <dd class="w100">KRX 시리즈 지수의 시세정보 제공</dd>
    </dl>
  </div>
  <table>
    <tbody>
      <tr>
        <th>API ID</th>
        <td><input type="text" name="apiId" value="krx_dd_trd" /></td>
      </tr>
    </tbody>
  </table>
</form>
"#;

    let entry = parse_detail_page("지수", html).expect("detail page should parse");

    assert_eq!(entry.category, "지수");
    assert_eq!(entry.api_id, "krx_dd_trd");
    assert_eq!(entry.path, "idx");
    assert_eq!(entry.name, "KRX 시리즈 일별시세정보");
    assert_eq!(entry.description, "KRX 시리즈 지수의 시세정보 제공");
}

#[test]
fn compare_inventories_reports_missing_and_changed_entries() {
    let expected = vec![
        ApiInventoryEntry {
            category: "지수".to_string(),
            api_id: "krx_dd_trd".to_string(),
            name: "KRX 시리즈 일별시세정보".to_string(),
            description: "KRX 시리즈 지수의 시세정보 제공".to_string(),
            path: "idx".to_string(),
        },
        ApiInventoryEntry {
            category: "주식".to_string(),
            api_id: "stk_bydd_trd".to_string(),
            name: "유가증권 일별매매정보".to_string(),
            description: "유가증권시장에 상장되어 있는 주권의 매매정보 제공".to_string(),
            path: "sto".to_string(),
        },
    ];
    let actual = vec![
        ApiInventoryEntry {
            category: "지수".to_string(),
            api_id: "krx_dd_trd".to_string(),
            name: "이름이 바뀜".to_string(),
            description: "설명이 바뀜".to_string(),
            path: "other".to_string(),
        },
        ApiInventoryEntry {
            category: "ESG".to_string(),
            api_id: "esg_index_info".to_string(),
            name: "ESG 지수".to_string(),
            description: "ESG 지수 정보를 제공".to_string(),
            path: "esg".to_string(),
        },
    ];

    let report = compare_inventories(&expected, &actual);

    assert_eq!(report.missing_upstream, vec!["stk_bydd_trd"]);
    assert_eq!(report.missing_local, vec!["esg_index_info"]);
    assert_eq!(report.changed.len(), 1);
    assert_eq!(report.changed[0].api_id, "krx_dd_trd");
    assert!(report.changed[0].fields.contains(&"name".to_string()));
    assert!(report.changed[0].fields.contains(&"path".to_string()));
}
