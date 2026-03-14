use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinSkillsTarget {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_key: Option<String>,
    pub relative_path: String,
}

#[derive(Debug, Clone)]
pub struct AgentPathClaim {
    pub agent_label: String,
    pub path: String,
    pub priority: u32,
}

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
    builtin_skills_targets: Vec<BuiltinSkillsTarget>,
    global_path_claims: Vec<AgentPathClaim>,
    project_path_claims: Vec<AgentPathClaim>,
}

pub fn default_visible_skill_target_ids() -> Vec<String> {
    vec![
        "universal".into(),
        "codex".into(),
        "antigravity".into(),
        "claude-code".into(),
        "codebuddy".into(),
        "kiro-cli".into(),
        "openclaw".into(),
        "qoder".into(),
        "trae".into(),
        "windsurf".into(),
    ]
}

pub const VISIBLE_SKILLS_TARGETS_VERSION: u32 = 1;

impl AgentRegistry {
    pub fn new() -> Self {
        let builtin_skills_targets = vec![
            BuiltinSkillsTarget {
                id: "universal".into(),
                label: "Universal".into(),
                label_key: Some("skills.targetLabels.universal".into()),
                relative_path: ".agents/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "codex".into(),
                label: "Codex IDE".into(),
                label_key: Some("skills.targetLabels.codex".into()),
                relative_path: ".codex/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "antigravity".into(),
                label: "Antigravity".into(),
                label_key: None,
                relative_path: ".agent/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "augment".into(),
                label: "Augment".into(),
                label_key: None,
                relative_path: ".augment/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "claude-code".into(),
                label: "Claude Code".into(),
                label_key: None,
                relative_path: ".claude/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "openclaw".into(),
                label: "OpenClaw".into(),
                label_key: None,
                relative_path: "skills".into(),
            },
            BuiltinSkillsTarget {
                id: "codebuddy".into(),
                label: "CodeBuddy".into(),
                label_key: None,
                relative_path: ".codebuddy/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "command-code".into(),
                label: "Command Code".into(),
                label_key: None,
                relative_path: ".commandcode/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "continue".into(),
                label: "Continue".into(),
                label_key: None,
                relative_path: ".continue/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "cortex-code".into(),
                label: "Cortex Code".into(),
                label_key: None,
                relative_path: ".cortex/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "crush".into(),
                label: "Crush".into(),
                label_key: None,
                relative_path: ".crush/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "droid".into(),
                label: "Droid".into(),
                label_key: None,
                relative_path: ".factory/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "goose".into(),
                label: "Goose".into(),
                label_key: None,
                relative_path: ".goose/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "junie".into(),
                label: "Junie".into(),
                label_key: None,
                relative_path: ".junie/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "iflow-cli".into(),
                label: "iFlow CLI".into(),
                label_key: None,
                relative_path: ".iflow/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "kilo-code".into(),
                label: "Kilo Code".into(),
                label_key: None,
                relative_path: ".kilocode/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "kiro-cli".into(),
                label: "Kiro CLI".into(),
                label_key: None,
                relative_path: ".kiro/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "kode".into(),
                label: "Kode".into(),
                label_key: None,
                relative_path: ".kode/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "mcpjam".into(),
                label: "MCPJam".into(),
                label_key: None,
                relative_path: ".mcpjam/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "mistral-vibe".into(),
                label: "Mistral Vibe".into(),
                label_key: None,
                relative_path: ".vibe/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "mux".into(),
                label: "Mux".into(),
                label_key: None,
                relative_path: ".mux/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "openhands".into(),
                label: "OpenHands".into(),
                label_key: None,
                relative_path: ".openhands/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "pi".into(),
                label: "Pi".into(),
                label_key: None,
                relative_path: ".pi/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "qoder".into(),
                label: "Qoder".into(),
                label_key: None,
                relative_path: ".qoder/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "qwen-code".into(),
                label: "Qwen Code".into(),
                label_key: None,
                relative_path: ".qwen/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "roo-code".into(),
                label: "Roo Code".into(),
                label_key: None,
                relative_path: ".roo/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "trae".into(),
                label: "Trae".into(),
                label_key: None,
                relative_path: ".trae/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "trae-cn".into(),
                label: "Trae CN".into(),
                label_key: None,
                relative_path: ".trae/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "windsurf".into(),
                label: "Windsurf".into(),
                label_key: None,
                relative_path: ".windsurf/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "zencoder".into(),
                label: "Zencoder".into(),
                label_key: None,
                relative_path: ".zencoder/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "neovate".into(),
                label: "Neovate".into(),
                label_key: None,
                relative_path: ".neovate/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "pochi".into(),
                label: "Pochi".into(),
                label_key: None,
                relative_path: ".pochi/skills".into(),
            },
            BuiltinSkillsTarget {
                id: "adal".into(),
                label: "AdaL".into(),
                label_key: None,
                relative_path: ".adal/skills".into(),
            },
        ];
        let agents = vec![
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
        ];

        Self {
            builtin_skills_targets,
            global_path_claims: vec![
                AgentPathClaim {
                    agent_label: "Claude Code".into(),
                    path: ".claude/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "OpenAI Codex".into(),
                    path: ".codex/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "OpenAI Codex".into(),
                    path: ".agents/skills".into(),
                    priority: 90,
                },
                AgentPathClaim {
                    agent_label: "Cursor".into(),
                    path: ".cursor/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "GitHub Copilot / VS Code".into(),
                    path: ".copilot/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "Cursor".into(),
                    path: ".agents/skills".into(),
                    priority: 80,
                },
                AgentPathClaim {
                    agent_label: "GitHub Copilot / VS Code".into(),
                    path: ".agents/skills".into(),
                    priority: 70,
                },
            ],
            project_path_claims: vec![
                AgentPathClaim {
                    agent_label: "Claude Code".into(),
                    path: ".claude/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "OpenAI Codex".into(),
                    path: ".codex/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "OpenAI Codex".into(),
                    path: ".agents/skills".into(),
                    priority: 90,
                },
                AgentPathClaim {
                    agent_label: "Cursor".into(),
                    path: ".cursor/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "Cursor".into(),
                    path: ".agents/skills".into(),
                    priority: 80,
                },
                AgentPathClaim {
                    agent_label: "GitHub Copilot / VS Code".into(),
                    path: ".github/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "GitHub Copilot / VS Code".into(),
                    path: ".agents/skills".into(),
                    priority: 70,
                },
            ],
            agents,
        }
    }

    pub fn agents(&self) -> &[AgentCapability] {
        &self.agents
    }

    pub fn builtin_skills_targets(&self) -> &[BuiltinSkillsTarget] {
        &self.builtin_skills_targets
    }

    pub fn builtin_skills_target_by_id(&self, id: &str) -> Option<&BuiltinSkillsTarget> {
        self.builtin_skills_targets.iter().find(|target| target.id == id)
    }

    pub fn preferred_global_path_for(&self, label: &str) -> Option<&str> {
        self.global_path_claims
            .iter()
            .filter(|claim| claim.agent_label == label)
            .max_by_key(|claim| claim.priority)
            .map(|claim| claim.path.as_str())
    }

    pub fn preferred_project_path_for(&self, label: &str) -> Option<&str> {
        self.project_path_claims
            .iter()
            .filter(|claim| claim.agent_label == label)
            .max_by_key(|claim| claim.priority)
            .map(|claim| claim.path.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::AgentRegistry;

    #[test]
    fn includes_codex_in_builtin_targets_and_visible_defaults() {
        let registry = AgentRegistry::new();
        let codex_target = registry
            .builtin_skills_target_by_id("codex")
            .expect("codex target should exist");

        assert_eq!(codex_target.relative_path, ".codex/skills");
        assert!(super::default_visible_skill_target_ids()
            .iter()
            .any(|target_id| target_id == "codex"));
    }
}
