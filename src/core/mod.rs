use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

pub type Context = Vec<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Continue,
    Stop,
    CallTool(String),
}

impl Default for Action {
    fn default() -> Self {
        Action::Continue
    }
}

#[derive(Debug, Clone)]
pub struct Output {
    pub action: Action,
    pub value: Option<Value>,
}

#[async_trait]
pub trait Node: Send + Sync {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>>;
    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>>;
    async fn post(&mut self, prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<()>;

    async fn run(&mut self, ctx: &mut Context) -> Result<()> {
        let prep_res = self.prep(ctx).await?;
        let exec_res = self.exec(prep_res.clone()).await?;
        self.post(prep_res, exec_res, ctx).await
    }
}

pub async fn exec_with_retry<N: Node>(
    node: &mut N,
    prep_res: Option<Value>,
    max_retries: usize,
    wait_secs: u64,
) -> Result<Option<Value>> {
    let mut retries = 0;
    loop {
        match node.exec(prep_res.clone()).await {
            Ok(res) => return Ok(res),
            Err(e) if retries >= max_retries => return Err(e),
            Err(_) => {
                sleep(Duration::from_secs(wait_secs)).await;
                retries += 1;
            }
        }
    }
}

pub async fn exec_batch<N: Node>(
    node: &mut N,
    values: &[Value],
    batch_size: usize,
) -> Result<Vec<Value>> {
    let mut results = Vec::new();
    for chunk in values.chunks(batch_size) {
        for val in chunk {
            if let Some(v) = node.exec(Some(val.clone())).await? {
                results.push(v);
            }
        }
    }
    Ok(results)
}