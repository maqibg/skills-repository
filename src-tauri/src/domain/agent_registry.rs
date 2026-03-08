use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapability {
    pub id: String,
    pub label: String,
    pub global_paths: Vec<String>,
    pub project_paths: Vec<String>,
    pub default_global_mode: String,
    pub default_project_mode: String,
}

#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: Vec<AgentCapability>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: vec![
                AgentCapability {
                    id: "claude-code".into(),
                    label: "Claude Code".into(),
                    global_paths: vec![".claude/skills".into()],
                    project_paths: vec![".claude/skills".into()],
                    default_global_mode: "symlink".into(),
                    default_project_mode: "copy".into(),
                },
                AgentCapability {
                    id: "codex".into(),
                    label: "OpenAI Codex".into(),
                    global_paths: vec![".agents/skills".into(), ".codex/skills".into()],
                    project_paths: vec![".agents/skills".into(), ".codex/skills".into()],
                    default_global_mode: "symlink".into(),
                    default_project_mode: "copy".into(),
                },
                AgentCapability {
                    id: "cursor".into(),
                    label: "Cursor".into(),
                    global_paths: vec![".cursor/skills".into()],
                    project_paths: vec![".cursor/skills".into(), ".agents/skills".into()],
                    default_global_mode: "symlink".into(),
                    default_project_mode: "copy".into(),
                },
                AgentCapability {
                    id: "github-copilot".into(),
                    label: "GitHub Copilot / VS Code".into(),
                    global_paths: vec![".copilot/skills".into(), ".agents/skills".into()],
                    project_paths: vec![".github/skills".into(), ".agents/skills".into()],
                    default_global_mode: "symlink".into(),
                    default_project_mode: "copy".into(),
                },
            ],
        }
    }

    pub fn agents(&self) -> &[AgentCapability] {
        &self.agents
    }
}
