use std::time::Duration;

pub async fn is_apache_server(url: &str, timeout_secs: u64) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(url).send().await {
        Ok(response) => {
            let server_header = response
                .headers()
                .get(reqwest::header::SERVER)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_lowercase();
            server_header.contains("apache") || server_header.contains("xampp")
        }
        Err(_) => false,
    }
}
