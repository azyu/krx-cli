# KRX OPEN API Reference

- 최종 확인: `2026-03-07`
- 공식 목록: [https://openapi.krx.co.kr/contents/OPP/INFO/service/OPPINFO004.cmd](https://openapi.krx.co.kr/contents/OPP/INFO/service/OPPINFO004.cmd)
- 범위: 서비스 목록 페이지에 공개된 API 31개

## 공통 호출 규칙

- 회원가입/로그인 후 `API 인증키`를 신청해야 하며, 공식 안내상 관리자 승인 후 실제 API 사용이 가능합니다.
- 공개 상세 페이지 기준 모든 API의 샘플 요청 방식은 `GET`입니다.
- 인증키는 Request 헤더의 `AUTH_KEY` 필드로 전달합니다.
- 샘플 응답 포맷은 `.json`, `.xml` 두 가지입니다.
- 공개 상세 페이지에서 확인된 query parameter는 전 API 공통으로 `basDd` 1개입니다.
- `basDd`는 `기준일자`, 타입은 `string`, 길이는 대부분 `8`, 형식은 `YYYYMMDD`입니다.
- 샘플 URL 규칙: `https://data-dbg.krx.co.kr/svc/sample/apis/{path}/{apiId}.{json|xml}`
- 개발 명세서(`Spec.docx`)에 적힌 서버 엔드포인트 규칙: `https://data-dbg.krx.co.kr/svc/apis/{path}/{apiId}`
- 공개 샘플 키 `74D1B99DFBF345BBA3FB4476510A4BED4C78D13A`로는 `/svc/sample/apis/...` 호출은 가능했지만 `/svc/apis/...` 호출은 `401 Unauthorized API Call`이었습니다. 실제 CLI는 발급/승인된 키를 사용해야 합니다.
- `esg_etp_info`, `esg_index_info` 상세 페이지의 `basDd` 샘플값은 비어 있습니다.

```bash
curl -H 'AUTH_KEY: <issued-key>' \
  'https://data-dbg.krx.co.kr/svc/apis/idx/krx_dd_trd?basDd=20200414'
```

## API 카탈로그

| 카테고리 | API ID | API 명 | 서버 엔드포인트 | `basDd` 예시 | 출력 필드 수 | 최근 수정일 |
| --- | --- | --- | --- | --- | ---: | --- |
| 지수 | `krx_dd_trd` | KRX 시리즈 일별시세정보 | `idx/krx_dd_trd` | `20200414` | 12 | `2026/01/16` |
| 지수 | `kospi_dd_trd` | KOSPI 시리즈 일별시세정보 | `idx/kospi_dd_trd` | `20200414` | 12 | `2026/01/16` |
| 지수 | `kosdaq_dd_trd` | KOSDAQ 시리즈 일별시세정보 | `idx/kosdaq_dd_trd` | `20200414` | 12 | `2026/01/16` |
| 지수 | `bon_dd_trd` | 채권지수 시세정보 | `idx/bon_dd_trd` | `20200414` | 15 | `2026/01/16` |
| 지수 | `drvprod_dd_trd` | 파생상품지수 시세정보 | `idx/drvprod_dd_trd` | `20200414` | 9 | `2026/01/16` |
| 주식 | `stk_bydd_trd` | 유가증권 일별매매정보 | `sto/stk_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 주식 | `ksq_bydd_trd` | 코스닥 일별매매정보 | `sto/ksq_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 주식 | `knx_bydd_trd` | 코넥스 일별매매정보 | `sto/knx_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 주식 | `sw_bydd_trd` | 신주인수권증권 일별매매정보 | `sto/sw_bydd_trd` | `20200414` | 20 | `2026/01/16` |
| 주식 | `sr_bydd_trd` | 신주인수권증서 일별매매정보 | `sto/sr_bydd_trd` | `20200414` | 19 | `2026/01/16` |
| 주식 | `stk_isu_base_info` | 유가증권 종목기본정보 | `sto/stk_isu_base_info` | `20200414` | 12 | `2026/01/16` |
| 주식 | `ksq_isu_base_info` | 코스닥 종목기본정보 | `sto/ksq_isu_base_info` | `20200414` | 12 | `2026/01/16` |
| 주식 | `knx_isu_base_info` | 코넥스 종목기본정보 | `sto/knx_isu_base_info` | `20200414` | 12 | `2026/01/16` |
| 증권상품 | `etf_bydd_trd` | ETF 일별매매정보 | `etp/etf_bydd_trd` | `20200414` | 19 | `2026/01/16` |
| 증권상품 | `etn_bydd_trd` | ETN 일별매매정보 | `etp/etn_bydd_trd` | `20200414` | 19 | `2026/01/16` |
| 증권상품 | `elw_bydd_trd` | ELW 일별매매정보 | `etp/elw_bydd_trd` | `20200414` | 16 | `2026/01/16` |
| 채권 | `kts_bydd_trd` | 국채전문유통시장 일별매매정보 | `bon/kts_bydd_trd` | `20200414` | 17 | `2026/01/16` |
| 채권 | `bnd_bydd_trd` | 일반채권시장 일별매매정보 | `bon/bnd_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 채권 | `smb_bydd_trd` | 소액채권시장 일별매매정보 | `bon/smb_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 파생상품 | `fut_bydd_trd` | 선물 일별매매정보 (주식선물外) | `drv/fut_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 파생상품 | `eqsfu_stk_bydd_trd` | 주식선물(유가) 일별매매정보 | `drv/eqsfu_stk_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 파생상품 | `eqkfu_ksq_bydd_trd` | 주식선물(코스닥) 일별매매정보 | `drv/eqkfu_ksq_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 파생상품 | `opt_bydd_trd` | 옵션 일별매매정보 (주식옵션外) | `drv/opt_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 파생상품 | `eqsop_bydd_trd` | 주식옵션(유가) 일별매매정보 | `drv/eqsop_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 파생상품 | `eqkop_bydd_trd` | 주식옵션(코스닥) 일별매매정보 | `drv/eqkop_bydd_trd` | `20200414` | 15 | `2026/01/16` |
| 일반상품 | `oil_bydd_trd` | 석유시장 일별매매정보 | `gen/oil_bydd_trd` | `20200414` | 6 | `2026/01/16` |
| 일반상품 | `gold_bydd_trd` | 금시장 일별매매정보 | `gen/gold_bydd_trd` | `20200414` | 11 | `2026/01/16` |
| 일반상품 | `ets_bydd_trd` | 배출권 시장 일별매매정보 | `gen/ets_bydd_trd` | `20200414` | 11 | `2026/01/16` |
| ESG | `esg_etp_info` | ESG 증권상품 | `esg/esg_etp_info` | `미표기` | 8 | `2026/01/16` |
| ESG | `sri_bond_info` | 사회책임투자채권 정보 | `esg/sri_bond_info` | `20200414` | 12 | `2026/01/16` |
| ESG | `esg_index_info` | ESG 지수 | `esg/esg_index_info` | `미표기` | 8 | `2026/01/16` |

## 카테고리별 메모

### 지수

- `krx_dd_trd`: KRX 시리즈 지수의 시세정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES001_S2.cmd?BO_ID=SsgXTEspyJESKvyXZtCU
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/idx/krx_dd_trd`
  출력 필드(12): `BAS_DD, IDX_CLSS, IDX_NM, CLSPRC_IDX, CMPPREVDD_IDX, FLUC_RT, OPNPRC_IDX, HGPRC_IDX, LWPRC_IDX, ACC_TRDVOL, ACC_TRDVAL, MKTCAP`
- `kospi_dd_trd`: KOSPI 시리즈 지수의 시세정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES001_S2.cmd?BO_ID=EREKZauXnMmxyIlqzeDN
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/idx/kospi_dd_trd`
  출력 필드(12): `BAS_DD, IDX_CLSS, IDX_NM, CLSPRC_IDX, CMPPREVDD_IDX, FLUC_RT, OPNPRC_IDX, HGPRC_IDX, LWPRC_IDX, ACC_TRDVOL, ACC_TRDVAL, MKTCAP`
- `kosdaq_dd_trd`: KOSDAQ 시리즈 지수의 시세정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES001_S2.cmd?BO_ID=nimebcamqFNIPNcRrHoO
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/idx/kosdaq_dd_trd`
  출력 필드(12): `BAS_DD, IDX_CLSS, IDX_NM, CLSPRC_IDX, CMPPREVDD_IDX, FLUC_RT, OPNPRC_IDX, HGPRC_IDX, LWPRC_IDX, ACC_TRDVOL, ACC_TRDVAL, MKTCAP`
- `bon_dd_trd`: 채권지수의 시세정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES001_S2.cmd?BO_ID=vMxIKCtPBUeRytCqkoFv
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/idx/bon_dd_trd`
  출력 필드(15): `BAS_DD, BND_IDX_GRP_NM, TOT_EARNG_IDX, TOT_EARNG_IDX_CMPPREVDD, NETPRC_IDX, NETPRC_IDX_CMPPREVDD, ZERO_REINVST_IDX, ZERO_REINVST_IDX_CMPPREVDD, CALL_REINVST_IDX, CALL_REINVST_IDX_CMPPREVDD, MKT_PRC_IDX, MKT_PRC_IDX_CMPPREVDD, AVG_DURATION, AVG_CONVEXITY_PRC, BND_IDX_AVG_YD`
- `drvprod_dd_trd`: 파생상품지수의 시세정보를 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES001_S2.cmd?BO_ID=rPBjbLtScMwmSXWDOYPd
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/idx/drvprod_dd_trd`
  출력 필드(9): `BAS_DD, IDX_CLSS, IDX_NM, CLSPRC_IDX, CMPPREVDD_IDX, FLUC_RT, OPNPRC_IDX, HGPRC_IDX, LWPRC_IDX`

### 주식

- `stk_bydd_trd`: 유가증권시장에 상장되어 있는 주권의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=JvJFzlAENzZlPBDNGAWC
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/sto/stk_bydd_trd`
  출력 필드(15): `BAS_DD, ISU_CD, ISU_NM, MKT_NM, SECT_TP_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL, MKTCAP, LIST_SHRS`
- `ksq_bydd_trd`: 코스닥시장에 상장되어 있는 주권의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=hZjGpkllgCBCWqeTsYFj
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/sto/ksq_bydd_trd`
  출력 필드(15): `BAS_DD, ISU_CD, ISU_NM, MKT_NM, SECT_TP_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL, MKTCAP, LIST_SHRS`
- `knx_bydd_trd`: 코넥스시장에 상장되어 있는 주권의 매매정보 제공 ('13년07월01일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=HSiRvxGSYnvaKuAuqpqp
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/sto/knx_bydd_trd`
  출력 필드(15): `BAS_DD, ISU_CD, ISU_NM, MKT_NM, SECT_TP_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL, MKTCAP, LIST_SHRS`
- `sw_bydd_trd`: 유가증권/코스닥시장에 상장되어 있는 신주인수권증권의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=erXKnEAzTqcGnkcoSdGA
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/sto/sw_bydd_trd`
  출력 필드(20): `BAS_DD, MKT_NM, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL, MKTCAP, LIST_SHRS, EXER_PRC, EXST_STRT_DD, EXST_END_DD, TARSTK_ISU_SRT_CD, TARSTK_ISU_NM, TARSTK_ISU_PRSNT_PRC`
- `sr_bydd_trd`: 유가증권/코스닥시장에 상장되어 있는 신주인수권증서의 매매정보 제공 ('10년02월12일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=YieGrzzJtKhbaNLuKmhz
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/sto/sr_bydd_trd`
  출력 필드(19): `BAS_DD, MKT_NM, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL, MKTCAP, LIST_SHRS, ISU_PRC, DELIST_DD, TARSTK_ISU_SRT_CD, TARSTK_ISU_NM, TARSTK_ISU_PRSNT_PRC`
- `stk_isu_base_info`: 유가증권 종목기본정보 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=PiwgMdTwmsenXhmqqxuj
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/sto/stk_isu_base_info`
  출력 필드(12): `ISU_CD, ISU_SRT_CD, ISU_NM, ISU_ABBRV, ISU_ENG_NM, LIST_DD, MKT_TP_NM, SECUGRP_NM, SECT_TP_NM, KIND_STKCERT_TP_NM, PARVAL, LIST_SHRS`
- `ksq_isu_base_info`: 코스닥 종목기본정보 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=CifLHplnUFMgpHIMMPXs
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/sto/ksq_isu_base_info`
  출력 필드(12): `ISU_CD, ISU_SRT_CD, ISU_NM, ISU_ABBRV, ISU_ENG_NM, LIST_DD, MKT_TP_NM, SECUGRP_NM, SECT_TP_NM, KIND_STKCERT_TP_NM, PARVAL, LIST_SHRS`
- `knx_isu_base_info`: 코넥스 종목기본정보 ('13년07월01일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES002_S2.cmd?BO_ID=COgTLqgmGlqyJvaEFNIc
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/sto/knx_isu_base_info`
  출력 필드(12): `ISU_CD, ISU_SRT_CD, ISU_NM, ISU_ABBRV, ISU_ENG_NM, LIST_DD, MKT_TP_NM, SECUGRP_NM, SECT_TP_NM, KIND_STKCERT_TP_NM, PARVAL, LIST_SHRS`

### 증권상품

- `etf_bydd_trd`: ETF(상장지수펀드)의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES003_S2.cmd?BO_ID=nrEpCLaZpoLCTzPUMxuF
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/etp/etf_bydd_trd`
  출력 필드(19): `BAS_DD, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, NAV, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL, MKTCAP, INVSTASST_NETASST_TOTAMT, LIST_SHRS, IDX_IND_NM, OBJ_STKPRC_IDX, CMPPREVDD_IDX, FLUC_RT_IDX`
- `etn_bydd_trd`: ETN(상장지수증권)의 매매정보 제공 ('14년11월17일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES003_S2.cmd?BO_ID=VujebrcOsZQMybnUuwLk
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/etp/etn_bydd_trd`
  출력 필드(19): `BAS_DD, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, PER1SECU_INDIC_VAL, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL, MKTCAP, INDIC_VAL_AMT, LIST_SHRS, IDX_IND_NM, OBJ_STKPRC_IDX, CMPPREVDD_IDX, FLUC_RT_IDX`
- `elw_bydd_trd`: ELW(주식위런트증권)의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES003_S2.cmd?BO_ID=brBhSEuDCUNpmfsCslfM
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/etp/elw_bydd_trd`
  출력 필드(16): `BAS_DD, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL, MKTCAP, LIST_SHRS, ULY_NM, ULY_PRC, CMPPREVDD_PRC_ULY, FLUC_RT_ULY`

### 채권

- `kts_bydd_trd`: 국채전문유통시장에 상장되어있는 채권의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES004_S2.cmd?BO_ID=CEnOyORzHgXWpdbUfWyf
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/bon/kts_bydd_trd`
  출력 필드(17): `BAS_DD, MKT_NM, ISU_CD, ISU_NM, BND_EXP_TP_NM, GOVBND_ISU_TP_NM, CLSPRC, CMPPREVDD_PRC, CLSPRC_YD, OPNPRC, OPNPRC_YD, HGPRC, HGPRC_YD, LWPRC, LWPRC_YD, ACC_TRDVOL, ACC_TRDVAL`
- `bnd_bydd_trd`: 일반채권시장에 상장되어있는 채권의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES004_S2.cmd?BO_ID=JfStBNhXISpVVfBHgspT
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/bon/bnd_bydd_trd`
  출력 필드(15): `BAS_DD, MKT_NM, ISU_CD, ISU_NM, CLSPRC, CMPPREVDD_PRC, CLSPRC_YD, OPNPRC, OPNPRC_YD, HGPRC, HGPRC_YD, LWPRC, LWPRC_YD, ACC_TRDVOL, ACC_TRDVAL`
- `smb_bydd_trd`: 소액채권시장에 상장되어있는 채권의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES004_S2.cmd?BO_ID=yrTTOsXuYzHprbWLuYzd
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/bon/smb_bydd_trd`
  출력 필드(15): `BAS_DD, MKT_NM, ISU_CD, ISU_NM, CLSPRC, CMPPREVDD_PRC, CLSPRC_YD, OPNPRC, OPNPRC_YD, HGPRC, HGPRC_YD, LWPRC, LWPRC_YD, ACC_TRDVOL, ACC_TRDVAL`

### 파생상품

- `fut_bydd_trd`: 파생상품시장의 선물 중 주식선물을 제외한 선물의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES005_S2.cmd?BO_ID=ilaVYOabbaicHbKTsqga
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/drv/fut_bydd_trd`
  출력 필드(15): `BAS_DD, PROD_NM, MKT_NM, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, SPOT_PRC, SETL_PRC, ACC_TRDVOL, ACC_TRDVAL, ACC_OPNINT_QTY`
- `eqsfu_stk_bydd_trd`: 파생상품시장의 주식선물 중 기초자산이 유가증권시장에 속하는 주식선물의 거래정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES005_S2.cmd?BO_ID=JzVvQnspImpuqtZlFWpJ
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/drv/eqsfu_stk_bydd_trd`
  출력 필드(15): `BAS_DD, PROD_NM, MKT_NM, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, SPOT_PRC, SETL_PRC, ACC_TRDVOL, ACC_TRDVAL, ACC_OPNINT_QTY`
- `eqkfu_ksq_bydd_trd`: 파생상품시장의 주식선물 중 기초자산이 코스닥시장에 속하는 주식선물의 거래정보 제공 ('15년08월03일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES005_S2.cmd?BO_ID=henfdJADfLTCUCBWIRCj
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/drv/eqkfu_ksq_bydd_trd`
  출력 필드(15): `BAS_DD, PROD_NM, MKT_NM, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, SPOT_PRC, SETL_PRC, ACC_TRDVOL, ACC_TRDVAL, ACC_OPNINT_QTY`
- `opt_bydd_trd`: 파생상품시장의 옵션 중 주식옵션을 제외한 옵션의 매매정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES005_S2.cmd?BO_ID=AoTvuFpukvuBsfypkZbq
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/drv/opt_bydd_trd`
  출력 필드(15): `BAS_DD, PROD_NM, RGHT_TP_NM, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, IMP_VOLT, NXTDD_BAS_PRC, ACC_TRDVOL, ACC_TRDVAL, ACC_OPNINT_QTY`
- `eqsop_bydd_trd`: 파생상품시장의 주식옵션 중 기초자산이 유가증권시장에 속하는 주식옵션의 거래정보 제공 ('10년01월04일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES005_S2.cmd?BO_ID=fwWKgzbevDVtAoECgkpA
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/drv/eqsop_bydd_trd`
  출력 필드(15): `BAS_DD, PROD_NM, RGHT_TP_NM, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, IMP_VOLT, NXTDD_BAS_PRC, ACC_TRDVOL, ACC_TRDVAL, ACC_OPNINT_QTY`
- `eqkop_bydd_trd`: 파생상품시장의 주식옵션 중 기초자산이 코스닥시장에 속하는 주식옵션의 거래정보 제공 ('17년06월26일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES005_S2.cmd?BO_ID=AFNbHSizSPnEssZoUqiS
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/drv/eqkop_bydd_trd`
  출력 필드(15): `BAS_DD, PROD_NM, RGHT_TP_NM, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, IMP_VOLT, NXTDD_BAS_PRC, ACC_TRDVOL, ACC_TRDVAL, ACC_OPNINT_QTY`

### 일반상품

- `oil_bydd_trd`: KRX 석유시장의 매매정보 제공 ('12년03월30일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES006_S2.cmd?BO_ID=rTvrZvAFKfcaLPOggJtW
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/gen/oil_bydd_trd`
  출력 필드(6): `BAS_DD, OIL_NM, WT_AVG_PRC, WT_DIS_AVG_PRC, ACC_TRDVOL, ACC_TRDVAL`
- `gold_bydd_trd`: KRX 금시장 매매정보 제공 ('14년03월24일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES006_S2.cmd?BO_ID=sxveSnWzWNzWxQASsgEG
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/gen/gold_bydd_trd`
  출력 필드(11): `BAS_DD, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL`
- `ets_bydd_trd`: KRX 탄소배출권 시장의 매매정보 제공 ('15년01월12일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES006_S2.cmd?BO_ID=IZiYdcgRQFMeENJPEMKG
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/gen/ets_bydd_trd`
  출력 필드(11): `BAS_DD, ISU_CD, ISU_NM, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, TDD_OPNPRC, TDD_HGPRC, TDD_LWPRC, ACC_TRDVOL, ACC_TRDVAL`

### ESG

- `esg_etp_info`: ESG 증권상품 정보를 제공 ('20년01월02일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES007_S2.cmd?BO_ID=dpRoGGhdnfSZSrMFtUCz
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/esg/esg_etp_info`
  출력 필드(8): `BAS_DD, ISU_ABBRV, TDD_CLSPRC, CMPPREVDD_PRC, FLUC_RT, LIST_SHRS, ACC_TRDVOL, ACC_TRDVAL`
- `sri_bond_info`: 사회책임투자채권 정보를 제공 ('19년01월01일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES007_S2.cmd?BO_ID=MwsSXzVIceQhMSJUeCdp
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/esg/sri_bond_info`
  출력 필드(12): `BAS_DD, ISUR_NM, ISU_CD, SRI_BND_TP_NM, ISU_NM, LIST_DD, ISU_DD, REDMPT_DD, ISU_RT, ISU_AMT, LIST_AMT, BND_TP_NM`
- `esg_index_info`: ESG 지수 정보를 제공 ('20년01월02일 데이터부터 제공)
  상세: https://openapi.krx.co.kr/contents/OPP/USES/service/OPPUSES007_S2.cmd?BO_ID=WgFYvEvsseQMARfMVZCq
  샘플: `https://data-dbg.krx.co.kr/svc/sample/apis/esg/esg_index_info`
  출력 필드(8): `BAS_DD, IDX_NM, CLSPRC_IDX, PRV_DD_CMPR, UPDN_RATE, TRD_ISU_CNT, ACC_TRDVOL, ACC_TRDVAL`
