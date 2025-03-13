use spin_sdk::http_wasip3::{Request, IntoResponse};

#[spin_sdk::http_wasip3::http_service]
async fn handle(_request: Request) -> impl IntoResponse {
    "Business logic executed!\n"
}
