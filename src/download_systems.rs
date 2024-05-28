use space_trader_api::apis as apis;
use space_trader_api::apis::configuration::Configuration;

const TOKEN: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiQ0FSTkU4IiwidmVyc2lvbiI6InYyLjIuMCIsInJlc2V0X2RhdGUiOiIyMDI0LTA1LTE5IiwiaWF0IjoxNzE2NTYwODYzLCJzdWIiOiJhZ2VudC10b2tlbiJ9.aDsB9OhPmg9Q6cN8MgyAOL5PRKVAuFzbVmNPwOjvrJ78OUkRA0oACTqXoVYm7yql1D_rDDhDSJvqb--qg5qcY73zYhE0-0qnJzO3UHaBCj9bhuSTu0-XkaydT8exgV_BlHA1tLo3mh9eg_16fawJuba7gq-PY8FE95P0SSOyJ67HBPh9DfbxyJu5E6FajBoCCe_cA954jpAM70zNa15mcIKbYw-6bLvIFPTvzDm6tHD3FaneOxTCoxv-Y8hP9e_bIuPVGBQLvv6wSg9mZN61kQSY_vtjM73GiPNpPG0te86UWhbvdBC6qpZfEMnxngXqpsrC0pLqtlZlfUVCAsImvw";

pub async fn download_systems() -> Result<(), String> {
    let config = Configuration::from_bearer_access_token(TOKEN.to_string());
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
