use space_trader_api::apis as apis;
use space_trader_api::apis::configuration::Configuration;

const TOKEN_FLAG: &str = "--token=";

pub async fn download_systems(token: &str) -> Result<(), String> {
    let config = Configuration::from_bearer_access_token(token.to_string());
    let agent = apis::agents_api::get_my_agent(&config)
        .await
        .map_err(|err| format!("Failed to get agent: {err}"))?;

    println!("{:?}", agent.data);

    let systems = apis::systems_api::get_systems(&config)
        .await
        .map_err(|err| format!("Failed to get systems: {err}"))?;

    let systems_json = serde_json::to_string(&systems).unwrap();
    std::fs::write("./systems.json", systems_json).unwrap();

    Ok(())
}

pub async fn download_systems_if_needed(args: Vec<String>) -> Result<(), String> {
    if args.iter().find(|&arg| arg == "--download-systems").is_some() { // Download if specified
        let token_arg = args
            .iter().find(|&arg| arg.starts_with(TOKEN_FLAG))
            .expect("No token argument");

        let token = &token_arg[TOKEN_FLAG.len()..];
        download_systems(token).await

    } else {
        if std::fs::exists("./systems.json").unwrap() { // Download if needed (when systems.json doesn't exist)
            return Ok(())
        };

        let token_arg = args
            .iter().find(|&arg| arg.starts_with(TOKEN_FLAG))
            .expect("No token argument");
        download_systems(&token_arg[TOKEN_FLAG.len()..]).await
    }
}