use async_trait::async_trait;
use serde_json::Value;

pub type Context = Vec<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Pending,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Result {
    pub status: Status,
    pub value: Option<Value>,
    pub error: Option<String>,
}

#[async_trait]
pub trait Node: Send + Sync {
    async fn prep(&mut self, ctx: &Context) -> Result;
    async fn exec(&mut self, ctx: &Context) -> Result;
    async fn post(&mut self, ctx: &mut Context) -> Result;

    async fn run(&mut self, ctx: &mut Context) -> Result {
        let prep_result = self.prep(ctx).await;
        if prep_result.status == Status::Failed {
            return self.post(ctx).await;
        }

        let exec_result = self.exec(ctx).await;
        if exec_result.status == Status::Failed {
            return self.post(ctx).await;
        }

        self.post(ctx).await
    }
}
