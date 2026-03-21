use serde::Deserialize;

use super::_entities::sea_orm_active_enums::{OrderType, Side};

impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "buy" => Ok(Side::Buy),
            "sell" => Ok(Side::Sell),
            other => Err(serde::de::Error::unknown_variant(other, &["buy", "sell"])),
        }
    }
}

impl schemars::JsonSchema for Side {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Side".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "enum": ["buy", "sell"]
        })
    }
}

impl<'de> Deserialize<'de> for OrderType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "limit" => Ok(OrderType::Limit),
            "market" => Ok(OrderType::Market),
            "stop_limit" => Ok(OrderType::StopLimit),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["limit", "market", "stop_limit"],
            )),
        }
    }
}

impl schemars::JsonSchema for OrderType {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "OrderType".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "enum": ["limit", "market", "stop_limit"]
        })
    }
}
