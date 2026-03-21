
use super::_entities::sea_orm_active_enums::{OrderType, Side, Status};
impl schemars::JsonSchema for Status {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Status".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "enum": ["waiting", "in progress", "cancelled", "completed"]
        })
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
