//! The rewrite prompt (SPEC §16, D-12): written for this project, not taken from training data. The output contract
//! it asks for is enforced by `mask::propose` (FR-9.3).

use crate::config::ModelSettings;

pub const SYSTEM: &str = "You edit one passage of prose so it reads as written by a careful human. \
Keep every fact, name, number and claim. Keep every placeholder like ⟦1⟧ exactly once, unchanged. \
Remove filler, hype, hedging preambles, 'not X but Y' framing and lists of three. Use plain words. \
Return only the rewritten passage: no preamble, no quotes, no notes.";

/// (input, output) pairs shown before the window.
pub const FEW_SHOT: &[(&str, &str)] = &[
    (
        "It's important to note that ⟦1⟧ plays a crucial role in the build — it's not just a tool, it's a foundation.",
        "⟦1⟧ drives the build.",
    ),
    (
        "Our team is committed to delivering a seamless, robust, and scalable experience for 30% more users.",
        "We want the service to keep working as it grows to 30% more users.",
    ),
];

/// `model.max_tokens`, or 3/2 × (masked chars / 4) + 16 when that setting is 0 (architecture §8).
pub fn max_tokens(masked: &str, s: &ModelSettings) -> u32 {
    if s.max_tokens > 0 {
        return s.max_tokens;
    }
    let chars = u32::try_from(masked.chars().count()).unwrap_or(u32::MAX);
    (chars / 4).saturating_mul(3) / 2 + 16
}
