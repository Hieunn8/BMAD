#[derive(Debug, Default)]
pub struct AudioPolicyService;

impl AudioPolicyService {
    pub fn should_replace_audio(policy: &str) -> bool {
        matches!(policy, "ReplaceAll" | "Replace all spoken tracks with channel voiceover")
    }
}
