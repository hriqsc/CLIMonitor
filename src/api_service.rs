use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::errors::APIError;



#[derive(Default,Deserialize)]
pub struct Entry {
    pub id: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "machineName")]
    pub machine_name: String,
    #[serde(rename = "threadID")]
    pub thread_id: i32,
    pub server: String,
    pub function: String,
    pub environment: String,
    #[serde(rename = "dateTime")]
    pub date_time: String,
    #[serde(rename = "timeUp")]
    pub time_up: String,
    pub instructions: i64,
    #[serde(rename = "instructionsPS")]
    pub instructions_ps: i32,
    pub comments: String,
    pub memory: i32,
    #[serde(rename = "sID")]
    pub s_id: String,
    #[serde(rename = "idCTREE")]
    pub id_ctree: i32,
    #[serde(rename = "threadType")]
    pub thread_type: String,
    #[serde(rename = "inactiveTime")]
    pub inactive_time: String,
}


#[allow(dead_code)]
#[derive(Deserialize)]
struct Page{
    items: Vec<Entry>,
    #[serde(rename = "hasNext")]
    has_next: bool
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct MessageResponse{
    pub level : i32,
    pub message: Option<String>,
    pub data: Option<String>,
}



#[derive(Serialize)]
pub struct AuthRequest {
    pub login: String,
    pub password: String,
    pub env: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct AuthError{
    pub message: String,

}


/// Makes a GET request to the API to get a page of entries.
///
/// # Arguments
///
/// * `token` - The authorization token for the API request.
/// * `client` - The HTTP client used to make the API request.
/// * `page` - The page number to request.
/// * `page_size` - The number of entries per page to request.
///
/// # Errors
///
/// If the request fails, the function will panic with the error message.
///
/// # Returns
///
/// Returns a vector of `Entry`s.
pub async fn get_entries(config : &Config,token: &str, client: &Client, page: i32, page_size: i32) -> Result<Vec<Entry>,APIError>{
    let ip = &config.ip;
    let porta = &config.porta;
    let resp_tr = client
                .get(format!("http://{ip}:{porta}/webmonitor/webmnt?page={page}&pageSize={page_size}"))
                .header("Authorization", "token: ".to_string() + token)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0")
                .header("Accept", "application/json, text/plain")
                .send().await;
    
    let resp = match resp_tr {
        Ok(resp) => resp,
        Err(e) => return Err(APIError::RequestError(e.to_string())),
    };

    let pages : Page = match resp.json().await {
        Ok(pages) => pages,
        Err(e) => return Err(APIError::ParsingError(e.to_string())),
    };

    Ok(pages.items)
                
}

/// Makes a request to the api to get a token.
///
/// This function uses the `login`, `password`, and `enviorment` fields from the given `Config` to
/// make a POST request to the api to get a token. The token is then returned as a `String`.
///
/// # Errors
///
/// If the request fails, the function will panic with the error message.
pub async fn get_token(config : &Config, client: &Client) -> Result<String,APIError> {
    let request = AuthRequest {
        login: config.login.clone(),
        password: config.password.clone(),
        env: config.enviorment.clone(),
    };
    let resp = match client
                .post(format!("http://{}:{}/webmonitor/webmnt/auth",config.ip,config.porta))
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0")
                .header("Accept", "application/json, text/plain")
                .json(&request)
                .send()
                .await
    {
        Ok(resp) => resp,
        Err(e) => return Err(APIError::RequestError(e.to_string())),
    };

    if !resp.status().is_success(){
        let text : AuthError = match resp.json().await {
            Ok(auth) => auth,
            Err(e) => return Err(APIError::ParsingError(e.to_string())),
        };
        return Err(APIError::AuthFail(text.message));
    }

    let auth : AuthResponse = match resp.json().await {
        Ok(auth) => auth,
        Err(e) => return Err(APIError::ParsingError(e.to_string())),
    };

    Ok(auth.token)
}



/// Sends a DELETE request to remove connections based on entry IDs.
///
/// This function constructs a DELETE request to the given API endpoint to remove
/// connections identified by the provided entry IDs. It uses the authorization token
/// and client configuration details for the request.
///
/// # Arguments
///
/// * `config` - Configuration containing IP and port for the API endpoint.
/// * `id` - A vector of entry IDs to be deleted.
/// * `token` - The authorization token for the API request.
/// * `client` - The HTTP client used to make the API request.
///

pub async fn delete_connections(config : &Config,id: &Vec<String>, token: &str, client: &Client) -> Option<APIError>{

    match client
        .delete(format!("http://{}:{}/webmonitor/webmnt/{}",config.ip,config.porta,id.join(",")))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0")
        .header("Accept", "application/json, text/plain")
        .header("Authorization", "token: ".to_string() + token)
        .send()
        .await
    {
        Ok(_) => return None,
        Err(e) => return Some(APIError::RequestError(e.to_string())),
    };   
}


/// Sends a message to the given IDs.
///
/// This function constructs a GET request to the given API endpoint to send
/// a message to the given IDs. It uses the authorization token
/// and client configuration details for the request.
///
/// # Arguments
///
/// * `config` - Configuration containing IP and port for the API endpoint.
/// * `ids` - A vector of entry IDs to be sent the message.
/// * `message` - The message to be sent.
/// * `token` - The authorization token for the API request.
/// * `client` - The HTTP client used to make the API request.
///
/// # Returns
///
/// Returns a `MessageResponse` containing the status of the request and the
/// message that was sent.
pub async fn send_messages(config : &Config,ids: &Vec<String>, message: &str,token: &str, client: &Client) -> Result<MessageResponse,APIError>{
    let id_param = serde_json::to_string(&ids).map_err(|e| APIError::ParsingError(e.to_string()))?;
    let url = format!("http://{}:{}/webmonitor/webmnt/msg?msg={}&id={}",config.ip,config.porta ,message, id_param);
    let resp = match client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0")
        .header("Accept", "application/json, text/plain")
        .header("Authorization", "token: ".to_string() + token)
        .send()
        .await
    {
        Ok(resp) => resp.json(),
        Err(e) => return Err(APIError::RequestError(e.to_string())),
    };

    let resp_msg = match resp.await {
        Ok(resp_msg) => resp_msg,
        Err(e) => return Err(APIError::AsyncError(e.to_string())),
    };
    Ok(resp_msg)
    
}

