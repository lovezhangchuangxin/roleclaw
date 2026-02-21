use crate::domain::{DialogueOption, TurnResult};

pub struct TurnGenerationContext {
    pub location_id: String,
    pub player_role: String,
    pub selected_action: String,
    pub turn: u32,
    pub model: String,
}

pub trait NarrativeProvider {
    fn generate_turn(&self, ctx: &TurnGenerationContext) -> TurnResult;
}

pub struct OpenAiProvider;
pub struct ClaudeProvider;

fn default_options() -> Vec<DialogueOption> {
    vec![
        DialogueOption {
            id: "opt_plot_1".to_string(),
            kind: "plot".to_string(),
            text: "追问眼前线索的源头".to_string(),
        },
        DialogueOption {
            id: "opt_emotion_1".to_string(),
            kind: "emotion".to_string(),
            text: "先建立信任再推进话题".to_string(),
        },
        DialogueOption {
            id: "opt_risk_1".to_string(),
            kind: "risk".to_string(),
            text: "冒险试探禁区情报".to_string(),
        },
    ]
}

impl NarrativeProvider for OpenAiProvider {
    fn generate_turn(&self, ctx: &TurnGenerationContext) -> TurnResult {
        TurnResult {
            narration: format!(
                "你在「{}」执行了动作：{}。基于 {} 的演算结果，局势出现新线索，作为{}你感到局面正在朝可控方向变化。",
                ctx.location_id, ctx.selected_action, ctx.model, ctx.player_role
            ),
            options: default_options(),
            state_changes_preview: vec![
                format!("回合推进到 {}", ctx.turn + 1),
                "主线推进度小幅提升".to_string(),
            ],
            event_hints: vec!["可能触发：地点关联事件".to_string()],
        }
    }
}

impl NarrativeProvider for ClaudeProvider {
    fn generate_turn(&self, ctx: &TurnGenerationContext) -> TurnResult {
        TurnResult {
            narration: format!(
                "在「{}」，你做出了“{}”这一选择。{} 给出的叙事推演显示，关键人物开始重新评估你的立场，暗线正在浮现。",
                ctx.location_id, ctx.selected_action, ctx.model
            ),
            options: default_options(),
            state_changes_preview: vec![
                format!("回合推进到 {}", ctx.turn + 1),
                "关系网张力提升".to_string(),
            ],
            event_hints: vec!["可能触发：人物关系阈值事件".to_string()],
        }
    }
}

pub fn generate_turn(provider: &str, ctx: &TurnGenerationContext) -> Result<TurnResult, String> {
    match provider {
        "openai" => Ok(OpenAiProvider.generate_turn(ctx)),
        "claude" => Ok(ClaudeProvider.generate_turn(ctx)),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}
