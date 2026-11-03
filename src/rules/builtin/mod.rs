//! The built-in rule catalogue (SPEC §10). Each rule story adds its file and one line to `all()`, in code order.
//! SLP018–SLP021 share cached paragraph metrics and configured sample floors; all are provisional, default-off
//! suggestions with no fix or model window.

use super::Rule;

mod slp001_ai_vocabulary_replaceable;
mod slp002_ai_vocabulary;
mod slp003_not_x_but_y;
mod slp004_not_just_but_also;
mod slp005_rule_of_three;
mod slp006_copula_avoidance;
mod slp007_significance_puffery;
mod slp008_hedging_preamble;
mod slp009_stacked_hedge;
mod slp010_bullet_spam;
mod slp011_over_bolding;
mod slp012_chatbot_closer;
mod slp013_chatbot_opener;
mod slp014_em_dash_density;
mod slp015_ai_disclaimer;
mod slp016_summary_signpost;
mod slp017_decorative_emoji;
mod slp018_grade_level;
mod slp019_sentence_length_variation;
mod slp020_long_sentences;
mod slp021_lexical_diversity;

pub fn all() -> Vec<Box<dyn Rule>> {
    vec![
        slp001_ai_vocabulary_replaceable::rule(),
        slp002_ai_vocabulary::rule(),
        slp003_not_x_but_y::rule(),
        slp004_not_just_but_also::rule(),
        slp005_rule_of_three::rule(),
        slp006_copula_avoidance::rule(),
        slp007_significance_puffery::rule(),
        slp008_hedging_preamble::rule(),
        slp009_stacked_hedge::rule(),
        slp010_bullet_spam::rule(),
        slp011_over_bolding::rule(),
        slp012_chatbot_closer::rule(),
        slp013_chatbot_opener::rule(),
        slp014_em_dash_density::rule(),
        slp015_ai_disclaimer::rule(),
        slp016_summary_signpost::rule(),
        slp017_decorative_emoji::rule(),
        slp018_grade_level::rule(),
        slp019_sentence_length_variation::rule(),
        slp020_long_sentences::rule(),
        slp021_lexical_diversity::rule(),
    ]
}
