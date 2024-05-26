pub mod space_traders_types;

use serde::Deserialize;
pub use space_traders_types as types;

use types::{Agent, System};

#[derive(Debug, Deserialize)]
struct GetAgentResponse {
    data: Agent,
}

pub async fn get_agent(token: &str) -> Result<Agent, String> {
    let client = reqwest::Client::new();
    let url = "https://api.spacetraders.io/v2/my/agent";

    let result = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|err| format!("Failed to fetch: {err}"))?;

    let response = result
        .json::<GetAgentResponse>()
        .await
        .map_err(|err| format!("Failed to parse response: {err}"))?;

    Ok(response.data)
}


#[derive(Debug, Deserialize)]
struct GetSystemsResponseMeta {
    total: i32,
}

#[derive(Debug, Deserialize)]
struct GetSystemsResponse {
    data: Vec<System>,
    meta: GetSystemsResponseMeta
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetSystemsErrorData {
    retry_after: f32,
}
#[derive(Debug, Deserialize)]
struct GetSystemsError {
    data: GetSystemsErrorData
}
#[derive(Debug, Deserialize)]
struct GetSystemsErrorResponse {
    error: GetSystemsError
}

async fn get_system_page(client: &reqwest::Client, page: i32, token: &str) -> Result<GetSystemsResponse, Option<String>> {
    let url = format!("https://api.spacetraders.io/v2/systems?limit=20&page={}", page);

    let result =
        client
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|err| format!("Failed to fetch: {err}"))?;

    let response_status = result.status();

    let response_text =
        result
            .text()
            .await
            .map_err(|err| format!("Failed to parse response: {err}"))?;

    if response_status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let error_response =
            serde_json::from_str::<GetSystemsErrorResponse>(&response_text)
                .map_err(|err| format!("Failed to parse error response: {err}"))?;

        let duration = std::time::Duration::from_secs_f32(error_response.error.data.retry_after);
        tokio::time::sleep(duration).await;
        return Err(None);
    }

    let response =
        serde_json::from_str::<GetSystemsResponse>(&response_text)
            .map_err(|err| format!("Failed to parse response: {err}"))?;

    return Ok(response);
}

pub async fn get_systems(token: &str) -> Result<Vec<System>, String> {
    let client = reqwest::Client::new();

    let first_page =
        get_system_page(&client, 1, token)
            .await
            .map_err(|err| {
                let err = err.unwrap_or(String::from("Too any requests"));
                format!("Failed to retrieve first system page: {}", err)
            })?;

    let mut systems = first_page.data;
    let mut remaining_systems = first_page.meta.total - systems.len() as i32;
    let mut page = 2;

    while remaining_systems > 0 {
        let res = get_system_page(&client, page, token).await;

        match res {
            Err(Some(error)) => return Err(format!("Failed to retrieve system page: {error}")),
            Err(None) => {
                println!("Retrying system page fetch...");
                continue
            },
            Ok(mut response) => {
                let new_systems_count = response.data.len() as i32;

                systems.append(&mut response.data);
                remaining_systems -= new_systems_count as i32;
                page += 1;

                println!("Pulled {} systems. Remaining systems: {}, Total systems: {}", new_systems_count, remaining_systems, systems.len())
            }
        }
    }

    Ok(systems)
}
