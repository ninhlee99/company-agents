#![forbid(unsafe_code)]

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::{fmt,time::Duration};

#[derive(Debug,Clone)]
pub struct OutboundEmail { pub message_id:String,pub from:String,pub to:String,pub subject:String,pub html:String,pub idempotency_key:String,pub unsubscribe_url:Option<String> }
#[derive(Debug,Clone)]
pub struct ProviderReceipt { pub provider:String,pub provider_reference:String }
#[derive(Debug,Clone)] pub enum MessageError { Configuration(String), InvalidRequest(String), Unauthorized, RateLimited, Provider(String), Transport(String) }
impl fmt::Display for MessageError { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f,"email provider error") } }
#[async_trait]
pub trait OutboundMessageProvider: Send+Sync { async fn send_email(&self,email:&OutboundEmail)->Result<ProviderReceipt,MessageError>; }
#[derive(Clone)] pub struct ResendProvider { api_key:String, from:String, client:Client, api_base:String }
#[derive(Deserialize)] struct ResendResponse { id:Option<String> }
impl ResendProvider {
 pub fn from_env()->Result<Self,MessageError>{
  let api_key=std::env::var("RESEND_API_KEY").map_err(|_|MessageError::Configuration("RESEND_API_KEY is required".into()))?;
  let from=std::env::var("RESEND_FROM").map_err(|_|MessageError::Configuration("RESEND_FROM is required".into()))?;
  if api_key.trim().is_empty()||from.trim().is_empty(){return Err(MessageError::Configuration("empty Resend configuration".into()));}
  let client=Client::builder().connect_timeout(Duration::from_secs(5)).timeout(Duration::from_secs(20)).build().map_err(|e|MessageError::Configuration(e.to_string()))?;
  Ok(Self{api_key,from,client,api_base:std::env::var("RESEND_API_BASE").unwrap_or_else(|_|"https://api.resend.com".into())})
 }
}
#[async_trait] impl OutboundMessageProvider for ResendProvider {
 async fn send_email(&self,email:&OutboundEmail)->Result<ProviderReceipt,MessageError>{
  if email.message_id.trim().is_empty()||email.to.trim().is_empty()||email.subject.trim().is_empty()||email.html.trim().is_empty()||email.idempotency_key.trim().is_empty(){return Err(MessageError::InvalidRequest("required email fields missing".into()));}
  let mut payload=serde_json::json!({"from":self.from,"to":[email.to],"subject":email.subject,"html":email.html});
  if let Some(url)=&email.unsubscribe_url { payload["headers"]=serde_json::json!({"List-Unsubscribe":format!("<{url}>"),"List-Unsubscribe-Post":"List-Unsubscribe=One-Click"}); }
  let response=self.client.post(format!("{}/emails",self.api_base.trim_end_matches('/'))).bearer_auth(&self.api_key).header("Idempotency-Key",&email.idempotency_key).json(&payload).send().await.map_err(|e|MessageError::Transport(e.to_string()))?;
  if response.status()==StatusCode::UNAUTHORIZED||response.status()==StatusCode::FORBIDDEN{return Err(MessageError::Unauthorized);}
  if response.status()==StatusCode::TOO_MANY_REQUESTS{return Err(MessageError::RateLimited);}
  let status=response.status(); let body:ResendResponse=response.json().await.map_err(|e|MessageError::Provider(format!("invalid provider response: {e}")))?;
  if !status.is_success(){return Err(MessageError::Provider(format!("http {status}")));}
  Ok(ProviderReceipt{provider:"resend".into(),provider_reference:body.id.ok_or_else(||MessageError::Provider("provider reference missing".into()))?})
 }
}
