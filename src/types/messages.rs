#![allow(missing_docs)]

//! WebSocket message types.

use serde::{Deserialize, Serialize};

use super::order::{Action, SelfTradePrevention, Side};
use super::{
    deserialize_count, deserialize_dollars, deserialize_optional_count,
    deserialize_optional_dollars, TimestampMs,
};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum WsCommand {
    Subscribe {
        id: u64,
        params: SubscribeParams,
    },
    Unsubscribe {
        id: u64,
        params: UnsubscribeParams,
    },
    UpdateSubscription {
        id: u64,
        params: UpdateSubscriptionParams,
    },
    ListSubscriptions {
        id: u64,
    },
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SubscribeParams {
    pub channels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_initial_snapshot: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnsubscribeParams {
    pub sids: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateSubscriptionAction {
    AddMarkets,
    DeleteMarkets,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateSubscriptionParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_initial_snapshot: Option<bool>,
    pub action: UpdateSubscriptionAction,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Subscribed(SubscribedMsg),
    Unsubscribed(UnsubscribedMsg),
    #[serde(rename = "ok")]
    Ok(OkMsg),
    Error(ErrorMsg),
    OrderbookSnapshot(OrderbookSnapshotMsg),
    OrderbookDelta(OrderbookDeltaMsg),
    Ticker(TickerMsg),
    Trade(TradeMsg),
    Fill(FillMsg),
    MarketPosition(MarketPositionMsg),
    UserOrder(UserOrderMsg),
    #[serde(rename = "market_lifecycle_v2")]
    MarketLifecycle(MarketLifecycleMsg),
    EventLifecycle(EventLifecycleMsg),
    OrderGroupUpdates(OrderGroupUpdatesMsg),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscribedMsg {
    pub id: Option<u64>,
    pub msg: SubscriptionInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionInfo {
    pub channel: String,
    pub sid: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnsubscribedMsg {
    pub id: Option<u64>,
    pub sid: u64,
    pub seq: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OkMsg {
    pub id: Option<u64>,
    pub sid: Option<u64>,
    pub seq: Option<u64>,
    #[serde(default)]
    pub msg: Option<OkMsgData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OkMsgData {
    SubscriptionList(Vec<SubscriptionInfo>),
    SubscriptionUpdate(SubscriptionUpdateOk),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionUpdateOk {
    #[serde(default)]
    pub market_tickers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMsg {
    pub id: Option<u64>,
    pub msg: ErrorDetails,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorDetails {
    pub code: u32,
    pub msg: String,
    #[serde(default)]
    pub market_id: Option<String>,
    #[serde(default)]
    pub market_ticker: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookSnapshotMsg {
    pub sid: u64,
    pub seq: u64,
    pub msg: OrderbookSnapshotData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookSnapshotData {
    pub market_ticker: String,
    pub market_id: String,
    #[serde(default)]
    pub yes_dollars_fp: Vec<[String; 2]>,
    #[serde(default)]
    pub no_dollars_fp: Vec<[String; 2]>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookDeltaMsg {
    pub sid: u64,
    pub seq: u64,
    pub msg: OrderbookDeltaData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookDeltaData {
    pub market_ticker: String,
    pub market_id: String,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub price_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub delta_fp: i64,
    pub side: Side,
    #[serde(default)]
    pub ts: Option<String>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub subaccount: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickerMsg {
    pub sid: u64,
    pub msg: TickerData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickerData {
    pub market_ticker: String,
    pub market_id: String,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub price_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_bid_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_ask_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub volume_fp: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub open_interest_fp: i64,
    pub dollar_volume: u64,
    pub dollar_open_interest: u64,
    pub ts: TimestampMs,
    pub time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeMsg {
    pub sid: u64,
    pub msg: TradeData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeData {
    pub trade_id: String,
    pub market_ticker: String,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_price_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub no_price_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub count_fp: i64,
    pub taker_side: Side,
    pub ts: TimestampMs,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FillMsg {
    pub sid: u64,
    pub msg: FillData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FillData {
    pub trade_id: String,
    pub order_id: String,
    pub market_ticker: String,
    pub is_taker: bool,
    pub side: Side,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_price_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub count_fp: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub fee_cost: i64,
    pub action: Action,
    pub ts: TimestampMs,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(deserialize_with = "deserialize_count")]
    pub post_position_fp: i64,
    pub purchased_side: Side,
    #[serde(default)]
    pub subaccount: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketPositionMsg {
    pub sid: u64,
    pub msg: MarketPositionData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketPositionData {
    pub user_id: String,
    pub market_ticker: String,
    #[serde(deserialize_with = "deserialize_count")]
    pub position_fp: i64,
    pub position_cost: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub position_cost_dollars: i64,
    pub realized_pnl: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub realized_pnl_dollars: i64,
    pub fees_paid: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub fees_paid_dollars: i64,
    pub position_fee_cost: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub position_fee_cost_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub volume_fp: i64,
    #[serde(default)]
    pub subaccount: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserOrderMsg {
    pub sid: u64,
    pub msg: UserOrderData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserOrderData {
    pub order_id: String,
    pub user_id: String,
    pub ticker: String,
    pub status: String,
    pub side: Side,
    pub is_yes: bool,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub yes_price_dollars: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub fill_count_fp: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub remaining_count_fp: i64,
    #[serde(deserialize_with = "deserialize_count")]
    pub initial_count_fp: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub taker_fill_cost_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub maker_fill_cost_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub taker_fees_dollars: i64,
    #[serde(deserialize_with = "deserialize_dollars")]
    pub maker_fees_dollars: i64,
    pub client_order_id: String,
    #[serde(default)]
    pub order_group_id: Option<String>,
    #[serde(default)]
    pub self_trade_prevention_type: Option<SelfTradePrevention>,
    pub created_time: String,
    #[serde(default)]
    pub last_update_time: Option<String>,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub subaccount_number: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketLifecycleMsg {
    pub sid: u64,
    pub msg: MarketLifecycleData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketLifecycleData {
    pub market_ticker: String,
    pub event_type: String,
    #[serde(default)]
    pub open_ts: Option<TimestampMs>,
    #[serde(default)]
    pub close_ts: Option<TimestampMs>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub determination_ts: Option<TimestampMs>,
    #[serde(default, deserialize_with = "deserialize_optional_dollars")]
    pub settlement_value: Option<i64>,
    #[serde(default)]
    pub settled_ts: Option<TimestampMs>,
    #[serde(default)]
    pub is_deactivated: Option<bool>,
    #[serde(default)]
    pub additional_metadata: Option<MarketLifecycleMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketLifecycleMetadata {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub yes_sub_title: Option<String>,
    #[serde(default)]
    pub no_sub_title: Option<String>,
    #[serde(default)]
    pub rules_primary: Option<String>,
    #[serde(default)]
    pub rules_secondary: Option<String>,
    #[serde(default)]
    pub can_close_early: Option<bool>,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub expected_expiration_ts: Option<TimestampMs>,
    #[serde(default)]
    pub strike_type: Option<String>,
    #[serde(default)]
    pub floor_strike: Option<f64>,
    #[serde(default)]
    pub cap_strike: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventLifecycleMsg {
    pub sid: u64,
    pub msg: EventLifecycleData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventLifecycleData {
    pub event_ticker: String,
    pub title: String,
    pub subtitle: String,
    pub collateral_return_type: String,
    pub series_ticker: String,
    #[serde(default)]
    pub strike_date: Option<TimestampMs>,
    #[serde(default)]
    pub strike_period: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderGroupUpdatesMsg {
    pub sid: u64,
    pub seq: u64,
    pub msg: OrderGroupUpdatesData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderGroupUpdatesData {
    pub event_type: String,
    pub order_group_id: String,
    #[serde(default, deserialize_with = "deserialize_optional_count")]
    pub contracts_limit_fp: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_command_serialization() {
        let cmd = WsCommand::Subscribe {
            id: 1,
            params: SubscribeParams {
                channels: vec!["orderbook_delta".to_string()],
                market_ticker: None,
                market_tickers: Some(vec!["KXBTC-25JAN".to_string()]),
                send_initial_snapshot: None,
            },
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("orderbook_delta"));
        assert!(json.contains("KXBTC-25JAN"));
    }

    #[test]
    fn test_orderbook_delta_deserialization() {
        let json = r#"{
            "type": "orderbook_delta",
            "sid": 1,
            "seq": 42,
            "msg": {
                "market_ticker": "KXBTC-25JAN",
                "market_id": "123e4567-e89b-12d3-a456-426614174000",
                "price_dollars": "0.5500",
                "delta_fp": "-10.00",
                "side": "yes",
                "ts": "2024-01-15T12:00:00Z"
            }
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::OrderbookDelta(delta) => {
                assert_eq!(delta.seq, 42);
                assert_eq!(delta.msg.market_ticker, "KXBTC-25JAN");
                assert_eq!(delta.msg.price_dollars, 5_500);
                assert_eq!(delta.msg.delta_fp, -1_000);
            }
            _ => panic!("Expected OrderbookDelta"),
        }
    }
}
