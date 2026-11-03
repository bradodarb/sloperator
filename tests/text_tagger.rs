//! Conservative certainty and the Phase 4 SLP006 subject spike (SPEC §26, §28).
use sloperator::{
    doc::{self, Document, Format, ProtectSettings},
    text::{Tag, Tagged, segment::segment, tagger::tag},
};

fn document(src: &str) -> Document {
    doc::parse(src.into(), Format::Markdown, &ProtectSettings::default())
}

fn tags(src: &str) -> Vec<Tagged> {
    let doc = document(src);
    let seg = segment(&doc);
    let out = tag(&doc, &seg);
    assert_eq!(out.len(), seg.words.len());
    out
}

#[test]
fn known_word_is_certain() {
    assert_eq!(
        tags("Library"),
        vec![Tagged {
            tag: Tag::Noun,
            certain: true,
            unknown: false
        }]
    );
}

#[test]
fn unknown_word_flagged_and_guessed() {
    assert_eq!(
        tags("zorbly"),
        vec![Tagged {
            tag: Tag::Adv,
            certain: false,
            unknown: true
        }]
    );
}

#[test]
fn ambiguous_word_not_certain() {
    let out = tags("tool");
    assert_eq!(out[0].tag, Tag::Noun);
    assert!(!out[0].certain && !out[0].unknown);
}

#[test]
fn rule_changes_tag_but_never_certainty() {
    let out = tags("the guide");
    assert_eq!(out[1].tag, Tag::Noun);
    assert!(!out[1].certain);
    // Start with the most frequent noun tag, then transform after an infinitive.
    let out = tags("to guide");
    assert_eq!(out[1].tag, Tag::Verb);
    assert!(!out[1].certain && !out[1].unknown);
    let out = tags("to zorb");
    assert_eq!(out[1].tag, Tag::Verb);
    assert!(!out[1].certain && out[1].unknown);
}

#[test]
fn rules_respect_sentence_boundaries_and_lexicon_options() {
    assert_eq!(tags("to. Guide")[1].tag, Tag::Noun);
    assert_eq!(tags("to library")[1].tag, Tag::Noun);
    assert!(tags("to library")[1].certain);
    assert_eq!(tags("fast careful")[0].tag, Tag::Adv);
    assert!(!tags("fast careful")[0].certain);
    assert_eq!(tags("careful ready")[0].tag, Tag::Adj);
}

#[test]
fn one_tag_per_prose_word_in_source_order() {
    let out = tags("The library. Zorbly `hidden` careful 42.");
    assert_eq!(
        out.iter().map(|x| x.tag).collect::<Vec<_>>(),
        [Tag::Det, Tag::Noun, Tag::Adv, Tag::Adj, Tag::Num]
    );
    assert!(tags("`hidden`").is_empty());
    assert_eq!(tags("hello Zorbing")[1].tag, Tag::ProperNoun);
    assert_eq!(tags("Zorbing")[0].tag, Tag::VerbPart);
    assert_eq!(tags("zorped")[0].tag, Tag::VerbPast);
}

/// Prototype the §28 gate only here: this story deliberately does not change the rule.
fn admitted_classes(src: &str) -> Vec<usize> {
    let doc = document(src);
    let seg = segment(&doc);
    let out = tag(&doc, &seg);
    let lower = |i: usize| doc.src[seg.words[i].range.clone()].to_lowercase();
    let certain = |i: usize, t: Tag| out[i].certain && out[i].tag == t;
    let joined = |i: usize| {
        let gap = seg.words[i].range.end..seg.words[i + 1].range.start;
        doc.src[gap.clone()].trim().is_empty() && !doc.is_protected(&gap)
    };
    let mut classes = Vec::new();
    for s in &seg.sentences {
        let start = s.words.start as usize;
        let end = s.words.end as usize;
        for v in start..end {
            if !["serves", "stands", "functions"].contains(&lower(v).as_str())
                || v + 2 >= end
                || lower(v + 1) != "as"
                || ![
                    "a", "an", "the", "this", "that", "these", "those", "my", "your", "our",
                    "their", "his", "her", "its",
                ]
                .contains(&lower(v + 2).as_str())
            {
                continue;
            }
            let shape = [Tag::Det, Tag::Noun];
            let longer = [Tag::Det, Tag::Adj, Tag::Noun];
            for (class, expected) in [shape.as_slice(), longer.as_slice()].iter().enumerate() {
                if v - start != expected.len()
                    || !expected
                        .iter()
                        .enumerate()
                        .all(|(j, t)| certain(start + j, *t) && joined(start + j))
                    || !seg.words[start].sentence_initial
                    || ![
                        "a", "an", "another", "each", "every", "her", "his", "its", "my", "our",
                        "that", "the", "their", "this", "your",
                    ]
                    .contains(&lower(start).as_str())
                {
                    continue;
                }
                let first = v + 2;
                if first + 1 < end
                    && ["a matter", "a result", "a rule", "a whole"]
                        .contains(&format!("{} {}", lower(first), lower(first + 1)).as_str())
                {
                    continue;
                }
                let mut verb = false;
                for (i, tagged) in out.iter().enumerate().take(end).skip(first) {
                    if i > first
                        && doc.src[seg.words[i - 1].range.end..seg.words[i].range.start]
                            .contains([',', ';', ':', '(', '—', '–'])
                    {
                        break;
                    }
                    if matches!(
                        tagged.tag,
                        Tag::Verb | Tag::Verb3s | Tag::VerbPast | Tag::Modal
                    ) {
                        verb = true;
                    }
                }
                if !verb {
                    classes.push(class);
                }
            }
        }
    }
    classes
}

#[test]
fn spike_slp006_classes() {
    for (class, src) in [
        (0, "The library serves as a guide."),
        (1, "The main library serves as a guide."),
    ] {
        assert_eq!(admitted_classes(src), [class], "{src}");
    }
    let fixture = document(include_str!("../fixtures/rules/SLP006/positive.md"));
    let seg = segment(&fixture);
    for sentence in &seg.sentences {
        let src = &fixture.src[sentence.range.clone()];
        let expected = match src {
            "The library serves as a guide." => vec![0],
            "The main library serves as a guide." => vec![1],
            _ => vec![],
        };
        assert_eq!(admitted_classes(src), expected, "{src}");
    }
    for src in [
        include_str!("../fixtures/rules/SLP006/negative.md"),
        "They served as mentors.",
        "Both serve as anchors.",
        "The tool serves as a bridge.",
        "Please export these functions as a module.",
        "We expose helper functions as a convenience.",
        "It functions as follows.",
        "It serves as well as the old one.",
        "This can serve as well as any.",
        "These files can serve as templates.",
        "Here it serves as",
        "It serves as `code` here.",
        "The tools can't serve as a bridge, so we stand as one.",
        "Export the library functions as a module.",
        "The library functions as a module are deprecated.",
        "The library serves as a whole.",
        "The fast library serves as a guide.",
        "The service serves as a guide.",
        "The zorb library serves as a guide.",
    ] {
        assert!(admitted_classes(src).is_empty(), "{src}");
    }
}
