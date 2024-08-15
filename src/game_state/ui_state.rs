#[derive(Debug, Clone)]
pub enum UIState {
    CharacterSelection,
    /// SourceState
    CharacterDetail(Box<UIState>),
    /// Index into BattleState.characters to indicate whos skills are being selected
    ActionSelection(usize),
    /// Index into BattleState.characters to indicate whos skills are being selected; Index into
    /// characters skill list
    TargetSelection(usize, usize),
}
