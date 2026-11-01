//! The model registry (SPEC §15, ADR-08): every model `sloperator` knows how to fetch and prompt.
//! The spike (epic 02) may change `DEFAULT_MODEL` or add entries, but not the shape.

/// How a model family is prompted and sampled.
#[derive(Debug)]
pub struct PromptProfile {
    pub name: &'static str,
    pub temp: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub presence_penalty: f32,
    /// Appended to the user turn, e.g. Qwen3's soft switch that turns thinking off.
    pub user_suffix: &'static str,
    /// Remove one leading empty `<think></think>` block from the output (FR-9.3).
    pub strip_empty_think: bool,
}

/// The Qwen3 card's non-thinking sampling settings (research inference-and-models.md §2).
pub const QWEN3: PromptProfile = PromptProfile {
    name: "qwen3",
    temp: 0.7,
    top_p: 0.8,
    top_k: 20,
    presence_penalty: 1.5,
    user_suffix: " /no_think",
    strip_empty_think: true,
};

/// Every profile, for `model.profile` lookups by name.
pub const PROFILES: &[&PromptProfile] = &[&QWEN3];

/// One downloadable GGUF file and how to prompt it.
#[derive(Debug)]
pub struct ModelSpec {
    pub name: &'static str,
    /// The Hugging Face repository, e.g. `Qwen/Qwen3-4B-GGUF`.
    pub repo: &'static str,
    pub file: &'static str,
    /// Lowercase hex sha256 of `file`; `pull` verifies it.
    pub sha256: &'static str,
    pub bytes: u64,
    pub licence: &'static str,
    pub profile: &'static PromptProfile,
}

/// sha256 and sizes read from the Hugging Face API (`/api/models/<repo>?blobs=true`) on 2026-10-30.
pub const REGISTRY: &[ModelSpec] = &[
    ModelSpec {
        name: "qwen3-4b",
        repo: "Qwen/Qwen3-4B-GGUF",
        file: "Qwen3-4B-Q4_K_M.gguf",
        sha256: "7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5",
        bytes: 2_497_280_256,
        licence: "apache-2.0",
        profile: &QWEN3,
    },
    ModelSpec {
        name: "qwen3-1.7b",
        repo: "Qwen/Qwen3-1.7B-GGUF",
        file: "Qwen3-1.7B-Q8_0.gguf",
        sha256: "061b54daade076b5d3362dac252678d17da8c68f07560be70818cace6590cb1a",
        bytes: 1_834_426_016,
        licence: "apache-2.0",
        profile: &QWEN3,
    },
];

/// The model used when `model.name` is not set (D-6 / Brad Q4 (a)).
pub const DEFAULT_MODEL: &str = "qwen3-4b";
