use dashmap::DashMap;
use serde_json::Value;

///
/// Read cached response
///
pub fn get_user_tasks(
    cache: &DashMap<String, Value>,
    user_id: &str,
) -> Option<Value> {

    cache
        .get(user_id)
        .map(|entry| entry.clone())
}

///
/// Store cached response
///
pub fn set_user_tasks(
    cache: &DashMap<String, Value>,
    user_id: String,
    response: Value,
) {

    cache.insert(
        user_id,
        response,
    );
}

///
/// Remove cached response
///
pub fn invalidate_user_tasks(
    cache: &DashMap<String, Value>,
    user_id: &str,
) {

    cache.remove(user_id);
}

///
/// Clear all cache
///
pub fn clear_all(
    cache: &DashMap<String, Value>,
) {

    cache.clear();
}