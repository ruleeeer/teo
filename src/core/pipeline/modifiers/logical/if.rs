use async_trait::async_trait;
use crate::core::pipeline::modifier::Modifier;
use crate::core::pipeline::Pipeline;
use crate::core::pipeline::context::Context;
use crate::core::pipeline::context::stage::Stage::{ConditionFalse, ConditionTrue};

#[derive(Debug, Clone)]
pub struct IfModifier {
    pipeline: Pipeline
}

impl IfModifier {
    pub fn new(pipeline: Pipeline) -> Self {
        return IfModifier {
            pipeline
        };
    }
}

#[async_trait]
impl Modifier for IfModifier {

    fn name(&self) -> &'static str {
        "if"
    }

    async fn call<'a>(&self, ctx: Context<'a>) -> Context<'a> {
        if self.pipeline.process(ctx.clone()).await.is_valid() {
            ctx.alter_stage(ConditionTrue)
        } else {
            ctx.alter_stage(ConditionFalse)
        }
    }
}
