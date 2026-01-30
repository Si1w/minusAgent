use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::context::{Action, Context};

#[async_trait]
pub trait Node: Send + Sync {
    async fn prep(&mut self, ctx: &Context) -> Result<Option<Value>>;
    async fn exec(&mut self, prep_res: Option<Value>) -> Result<Option<Value>>;
    async fn post(&mut self, prep_res: Option<Value>, exec_res: Option<Value>, ctx: &mut Context) -> Result<Action>;

    async fn run(&mut self, ctx: &mut Context) -> Result<Action> {
        let prep_res = self.prep(ctx).await?;
        let exec_res = self.exec(prep_res.clone()).await?;
        self.post(prep_res, exec_res, ctx).await
    }
}
