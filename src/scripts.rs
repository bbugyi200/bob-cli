#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptKind {
    Bash,
    Python,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptAsset {
    pub command: &'static str,
    pub path: &'static str,
    pub kind: ScriptKind,
}

pub const SCRIPT_ASSETS: &[ScriptAsset] = &[
    ScriptAsset {
        command: "bob_pomodoro",
        path: "scripts/bob_pomodoro",
        kind: ScriptKind::Bash,
    },
    ScriptAsset {
        command: "bob_pomodoro_runtimes",
        path: "scripts/bob_pomodoro_runtimes",
        kind: ScriptKind::Python,
    },
    ScriptAsset {
        command: "bob_notify",
        path: "scripts/bob_notify",
        kind: ScriptKind::Bash,
    },
    ScriptAsset {
        command: "bob_sync",
        path: "scripts/bob_sync",
        kind: ScriptKind::Bash,
    },
    ScriptAsset {
        command: "tmux_bob_pomodoro",
        path: "scripts/tmux_bob_pomodoro",
        kind: ScriptKind::Bash,
    },
];

pub fn script_names() -> impl Iterator<Item = &'static str> {
    SCRIPT_ASSETS.iter().map(|asset| asset.command)
}
