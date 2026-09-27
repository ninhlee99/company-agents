            continue;
        }
        let name = item
            .get("groupName")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let kind = item
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let percentage_bps = if kind.eq_ignore_ascii_case("percentage") {
            item.get("percentage")
                .and_then(|v| v.as_f64())
                .filter(|v| v.is_finite() && *v >= 0.0)
                .map(|v| (v * 100.0).round().min(SCORE_MAX as f64) as u32)
        } else {
            None
        };
        let fixed_amount = if kind.eq_ignore_ascii_case("fix") {
            item.get("amount")
                .and_then(|v| v.as_f64())
                .filter(|v| v.is_finite() && *v >= 0.0)
        } else {
            None
        };
        let currency = item
            .get("currency")
            .and_then(|v| v.as_str())
            .map(ToOwned::to_owned);
        let is_default =
            code.eq_ignore_ascii_case("default") || name.to_ascii_lowercase().contains("default");
        out.push(ParsedCommissionGroup {
            code,
            is_default,
            percentage_bps,
            fixed_amount,
            currency,
        });
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
struct AwinVoucher {