use bidir_map::BidirMap;
use serde_json;
use serde_json::Value;
use serde_json::value::Map;

use error;
use pair::Pair;
use pair::Pair::*;

lazy_static! {
    static ref PAIRS_STRING: BidirMap<Pair, &'static str> = {
        let mut m = BidirMap::new();
        m.insert(BTC_ETH, "BTC_ETH");
        m.insert(BTC_ZEC, "BTC_ZEC");
        m
    };
}

/// Return the name associated to pair used by Poloniex
/// If the Pair is not supported, None is returned.
pub fn get_pair_string(pair: &Pair) -> Option<&&str> {
    PAIRS_STRING.get_by_first(pair)
}

/// Return the Pair enum associated to the string used by Poloniex
/// If the Pair is not supported, None is returned.
pub fn get_pair_enum(pair: &str) -> Option<&Pair> {
    PAIRS_STRING.get_by_second(&pair)
}

pub fn deserialize_json(json_string: String) -> Result<Map<String, Value>, error::Error> {
    let data: Value = match serde_json::from_str(&json_string) {
        Ok(data) => data,
        Err(_) => return Err(error::Error::BadParse),
    };

    match data.as_object() {
        Some(value) => Ok(value.clone()),
        None => Err(error::Error::BadParse),
    }
}


/// If error array is null, return the result (encoded in a json object)
/// else return the error string found in array
pub fn parse_result(response: Map<String, Value>) -> Result<Map<String, Value>, error::Error> {
    let error_array = match response.get("error") {
        Some(array) => array.as_array().unwrap(),
        None => return Err(error::Error::BadParse),
    };
    if error_array.is_empty() {
        return Ok(response.get("result").unwrap().as_object().unwrap().clone());
    }
    let error_msg = error_array[0].as_str().unwrap().to_string();

    match error_msg.as_ref() {
        "EService:Unavailable" => Err(error::Error::ServiceUnavailable),
        "EOrder:Rate limit exceeded" => Err(error::Error::RateLimitExceeded),
        "EQuery:Unknown asset pair" => Err(error::Error::PairUnsupported),
        "EGeneral:Invalid arguments" => Err(error::Error::InvalidArguments),
        other => Err(error::Error::ExchangeSpecificError(other.to_string())),
    }
}