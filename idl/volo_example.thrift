namespace rs volo.example

struct Item {
    1: required i64 id,
    2: required string title,
    3: required string content,

    10: optional map<string, string> extra,
}

struct GetItemRequest {
    1: required string ops,
    2: required string key,
    3: required string value,
    4: required string port,
}

struct GetItemResponse {
    1: required string value,
    2: required bool stat,
}

service ItemService {
    GetItemResponse GetItem (1: GetItemRequest req),
}

