use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;



#[allow(dead_code)]
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


pub async fn get_entries(token: &str, client: &Client, page: i32, page_size: i32) -> Vec<Entry>{
    let resp_tr = client
                .get(format!("http://10.70.2.42:2461/webmonitor/webmnt?page={page}&pageSize={page_size}"))
                .header("Authorization", "token: ".to_string() + token)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0")
                .header("Accept", "application/json, text/plain")
                .send().await;
    
    let resp = match resp_tr {
        Ok(resp) => resp,
        Err(e) => panic!("Error: {}", e),
    };

    let pages : Page = match resp.json().await {
        Ok(pages) => pages,
        Err(e) => panic!("Error: {}", e),
    };

    pages.items
                
}

#[derive(Deserialize)]
struct AuthResponse {
    token: String,
}

#[derive(Serialize)]
pub struct AuthRequest {
    pub login: String,
    pub password: String,
    pub env: String,
}


pub async fn get_token(config : &Config, client: &Client) -> String {
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
        Err(e) => panic!("Error: {}", e),
    };

    let auth : AuthResponse = match resp.json().await {
        Ok(auth) => auth,
        Err(e) => panic!("Error: {}", e),
    };

    auth.token
}


pub async fn delete_connection(config : &Config,id: &str, token: &str, client: &Client){
    match client
        .delete(format!("http://{}:{}/webmonitor/webmnt/{}",config.ip,config.porta,id))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0")
        .header("Accept", "application/json, text/plain")
        .header("Authorization", "token: ".to_string() + token)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => panic!("Error: {}", e),
    };
        
    
}


pub async fn send_message(config : &Config,id: &str, message: &str,token: &str, client: &Client) -> MessageResponse{
    let id_param = serde_json::to_string(&vec![id]).unwrap();
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
        Err(e) => panic!("Error: {}", e),
    };

    let resp_msg = match resp.await {
        Ok(resp_msg) => resp_msg,
        Err(e) => panic!("Error: {}", e),
    };
    resp_msg
    
}

