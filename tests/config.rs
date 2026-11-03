//! `config` (SPEC §13, §23.9): discovery, `extend`, `--config`, every documented key, precedence, per-file globs and
//! the errors that name the file and key.
#![allow(clippy::unwrap_used, clippy::expect_used)] // test helpers may unwrap (architecture §8)

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{fixture, run_cli};
use sloperator::Error;
use sloperator::cli::ExitCode;
use sloperator::config::{CliOverrides, Resolver, Settings, WindowMode};
use sloperator::rules::registry::Registry;
use sloperator::rules::{RuleCode, Severity};
use sloperator::text::Floors;

fn codes(s: &Settings) -> Vec<u16> {
    s.rules.keys().map(|c| c.0).collect()
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn canon(p: &Path) -> PathBuf {
    p.canonicalize().unwrap()
}

/// The `Error::Config` of resolving with `--config <file>`, as (file, key, reason).
fn config_error(path: &Path) -> (PathBuf, String, String) {
    match Resolver::new(CliOverrides::default(), Some(path)) {
        Err(Error::Config { file, key, reason }) => (file, key, reason),
        Err(e) => panic!("{}: expected Error::Config, got {e}", path.display()),
        Ok(_) => panic!("{}: expected Error::Config, got Ok", path.display()),
    }
}

#[test]
fn nearest_file_wins_no_parent_merge() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        &root.join("sloperator.toml"),
        "select = [\"SLP002\", \"SLP014\"]\nmin_severity = \"error\"\nexclude = [\"vendor/**\"]\n",
    );
    write(&root.join("sub/sloperator.toml"), "select = [\"SLP001\"]\n");
    write(&root.join("sub/deep/doc.md"), "text\n");
    write(&root.join("top.md"), "text\n");

    let mut r = Resolver::new(CliOverrides::default(), None).unwrap();

    let deep = r.for_file(&root.join("sub/deep/doc.md")).unwrap();
    assert_eq!(deep.source, Some(canon(&root.join("sub/sloperator.toml"))));
    assert_eq!(
        codes(&deep),
        vec![1],
        "only the nearest file's select applies"
    );
    assert_eq!(
        deep.min_severity,
        Severity::Warning,
        "the parent's min_severity is not merged"
    );
    assert!(
        deep.exclude.is_empty(),
        "the parent's exclude is not merged"
    );

    let top = r.for_file(&root.join("top.md")).unwrap();
    assert_eq!(top.source, Some(canon(&root.join("sloperator.toml"))));
    assert_eq!(codes(&top), vec![2, 14]);
    assert_eq!(top.min_severity, Severity::Error);

    // A file named but not yet on disk (a stdin name) is resolved from its directory.
    let unborn = r.for_file(&root.join("sub/new.md")).unwrap();
    assert_eq!(codes(&unborn), vec![1]);
}

#[test]
fn extend_chain_and_cycle_error() {
    let leaf = fixture("config/chain/leaf.toml");
    let mut r = Resolver::new(CliOverrides::default(), Some(&leaf)).unwrap();
    let s = r.for_file(&fixture("clean.md")).unwrap();
    // root: select {1, 2}, min_severity error, SLP014 param; mid (relative to team/): + 14; leaf: - 2.
    assert_eq!(codes(&s), vec![1, 14]);
    assert_eq!(s.min_severity, Severity::Error);
    assert_eq!(s.rules[&RuleCode(14)].params["max_per_1000_words"], 9.0);
    assert_eq!(s.source, Some(canon(&leaf)));

    let (file, key, reason) = config_error(&fixture("config/cycle/a.toml"));
    assert_eq!(key, "extend");
    assert_eq!(file, canon(&fixture("config/cycle/b.toml")));
    let a = canon(&fixture("config/cycle/a.toml")).display().to_string();
    let b = canon(&fixture("config/cycle/b.toml")).display().to_string();
    assert!(reason.contains("cycle"), "{reason}");
    assert!(
        reason.contains(&a) && reason.contains(&b),
        "the cycle names both files: {reason}"
    );
}

#[test]
fn explicit_config_overrides_discovery() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(&root.join("sloperator.toml"), "select = [\"SLP002\"]\n");
    write(
        &root.join("elsewhere/custom.toml"),
        "select = [\"SLP003\"]\n",
    );
    write(&root.join("doc.md"), "text\n");

    let mut found = Resolver::new(CliOverrides::default(), None).unwrap();
    assert_eq!(
        codes(&found.for_file(&root.join("doc.md")).unwrap()),
        vec![2]
    );

    let explicit = root.join("elsewhere/custom.toml");
    let mut r = Resolver::new(CliOverrides::default(), Some(&explicit)).unwrap();
    let s = r.for_file(&root.join("doc.md")).unwrap();
    assert_eq!(codes(&s), vec![3], "--config beats the discovered file");
    assert_eq!(s.source, Some(canon(&explicit)));

    // A missing --config is an error, never a silent fall back to discovery.
    let missing = Resolver::new(CliOverrides::default(), Some(&root.join("nope.toml")));
    assert!(matches!(missing, Err(Error::Io { .. })));
}

#[test]
fn all_documented_keys_parse() {
    let project = fixture("config/example/project");
    let mut r = Resolver::new(CliOverrides::default(), None).unwrap();
    let s = r.for_file(&project.join("README.md")).unwrap();

    assert_eq!(s.source, Some(canon(&project.join("sloperator.toml"))));
    assert_eq!(
        codes(&s),
        vec![1, 3, 5],
        "select, then extend_select, minus ignore"
    );
    assert_eq!(
        s.min_severity,
        Severity::Warning,
        "the file beats the parent it extends"
    );
    assert_eq!(s.include, vec!["**/*.md", "**/*.txt"]);
    assert_eq!(s.exclude, vec!["CHANGELOG.md", "vendor/**"]);
    assert!(s.respect_gitignore);
    assert_eq!(s.protect.patterns, vec![r"\bv\d+\.\d+\.\d+\b"]);
    assert_eq!(s.protect.terms, vec!["Acme Cloud", "Dr. Okafor"]);
    assert!(s.protect.front_matter && s.protect.math && s.protect.html);
    assert!(
        s.protect.compiled.is_some(),
        "patterns are compiled at load"
    );
    assert_eq!(s.model.name, "qwen3-4b");
    assert_eq!(s.model.path, Some(PathBuf::from("/models/my.gguf")));
    assert_eq!(s.model.profile.as_deref(), Some("qwen3"));
    assert_eq!(
        s.model.cache_dir,
        Some(PathBuf::from("~/.cache/sloperator/models"))
    );
    assert_eq!(s.model.window, WindowMode::Paragraph);
    assert_eq!(s.model.max_window_chars, 4000);
    assert_eq!(s.model.max_tokens, 0);
    assert_eq!(s.model.seed, 42);

    // [rules.SLP014] applies to SLP014 once it is enabled.
    let cli = CliOverrides {
        extend_select: vec!["SLP014".into()],
        ..Default::default()
    };
    let mut r = Resolver::new(cli, None).unwrap();
    let s = r.for_file(&project.join("README.md")).unwrap();
    let slp014 = &s.rules[&RuleCode(14)];
    assert_eq!(slp014.severity, Severity::Error);
    assert_eq!(slp014.params["max_per_1000_words"], 5.0);

    // `enabled` hands rules out in registry order with their resolved severity and params.
    let enabled = s.enabled(Registry::builtin());
    let order: Vec<u16> = enabled.iter().map(|e| e.rule.meta().code.0).collect();
    assert_eq!(order, vec![1, 3, 5, 14]);
    assert_eq!(enabled[3].severity, Severity::Error);
}

#[test]
fn cli_beats_file_beats_default() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(&root.join("plain/doc.md"), "text\n");
    write(
        &root.join("conf/sloperator.toml"),
        "ignore = [\"SLP001\"]\nmin_severity = \"error\"\n[model]\nseed = 7\nwindow = \"sentence\"\n",
    );
    write(&root.join("conf/doc.md"), "text\n");
    let defaults = Registry::builtin().default_set();
    let default_codes: Vec<u16> = defaults.iter().map(|c| c.0).collect();

    // Default: no file anywhere up the temp tree.
    let mut r = Resolver::new(CliOverrides::default(), None).unwrap();
    let d = r.for_file(&root.join("plain/doc.md")).unwrap();
    assert_eq!(d.source, None);
    assert_eq!(codes(&d), default_codes);
    assert_eq!(d.min_severity, Severity::Warning);
    assert_eq!(d.model.seed, 42);
    assert_eq!(d.model.window, WindowMode::Paragraph);

    // File beats default.
    let f = r.for_file(&root.join("conf/doc.md")).unwrap();
    assert!(!f.rules.contains_key(&RuleCode(1)));
    assert_eq!(f.min_severity, Severity::Error);
    assert_eq!(f.model.seed, 7);
    assert_eq!(f.model.window, WindowMode::Sentence);

    // CLI beats file (FR-2.11: --select / --extend-select / --ignore override the matching keys).
    let cli = CliOverrides {
        select: vec!["SLP001".into(), "SLP002".into()],
        extend_select: vec!["SLP014".into()],
        ignore: vec!["SLP002".into()],
        min_severity: Some(Severity::Suggestion),
        seed: Some(99),
    };
    let mut r = Resolver::new(cli, None).unwrap();
    let c = r.for_file(&root.join("conf/doc.md")).unwrap();
    assert_eq!(codes(&c), vec![1, 14]);
    assert_eq!(c.min_severity, Severity::Suggestion);
    assert_eq!(c.model.seed, 99);
    assert_eq!(
        c.model.window,
        WindowMode::Sentence,
        "the file still beats the default where the CLI is silent"
    );

    // CLI codes get the same checks as file codes, naming the command line.
    let bad = CliOverrides {
        ignore: vec!["SLP901".into()],
        ..Default::default()
    };
    match Resolver::new(bad, None) {
        Err(Error::Config { file, key, .. }) => {
            assert_eq!(file, PathBuf::from("<command line>"));
            assert_eq!(key, "ignore");
        }
        other => panic!("expected Error::Config, got {:?}", other.err()),
    }
}

#[test]
fn per_file_glob_overrides() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        &root.join("sloperator.toml"),
        concat!(
            "select = [\"SLP001\", \"SLP008\", \"SLP014\"]\n",
            "[[per_file]]\nglob = \"docs/legal/**\"\nignore = [\"SLP008\"]\nmin_severity = \"error\"\n",
            "[[per_file]]\nglob = \"docs/**\"\nextend_select = [\"SLP003\"]\n",
            "[per_file.rules.SLP014]\nmax_per_1000_words = 1.0\n",
        ),
    );
    write(&root.join("docs/legal/terms.md"), "text\n");
    write(&root.join("docs/guide.md"), "text\n");
    write(&root.join("README.md"), "text\n");
    write(&root.join("sub/docs/legal/x.md"), "text\n");

    let mut r = Resolver::new(CliOverrides::default(), None).unwrap();

    let legal = r.for_file(&root.join("docs/legal/terms.md")).unwrap();
    assert_eq!(
        codes(&legal),
        vec![1, 3, 14],
        "both entries match, in file order"
    );
    assert_eq!(legal.min_severity, Severity::Error);
    assert_eq!(legal.rules[&RuleCode(14)].params["max_per_1000_words"], 1.0);

    let guide = r.for_file(&root.join("docs/guide.md")).unwrap();
    assert_eq!(codes(&guide), vec![1, 3, 8, 14]);
    assert_eq!(guide.min_severity, Severity::Warning);

    let readme = r.for_file(&root.join("README.md")).unwrap();
    assert_eq!(codes(&readme), vec![1, 8, 14]);
    assert_eq!(
        readme.rules[&RuleCode(14)].params["max_per_1000_words"],
        3.0,
        "the rule's default"
    );

    // Globs match relative to the declaring file's directory, so sub/docs/legal is not docs/legal.
    let nested = r.for_file(&root.join("sub/docs/legal/x.md")).unwrap();
    assert_eq!(codes(&nested), vec![1, 8, 14]);

    // The CLI still beats a per-file entry.
    let cli = CliOverrides {
        min_severity: Some(Severity::Suggestion),
        ..Default::default()
    };
    let mut r = Resolver::new(cli, None).unwrap();
    let legal = r.for_file(&root.join("docs/legal/terms.md")).unwrap();
    assert_eq!(legal.min_severity, Severity::Suggestion);
}

#[test]
fn unknown_key_wrong_type_unknown_code_unknown_param_bad_regex_exit_two() {
    let cases: &[(&str, &str, &str)] = &[
        ("unknown_key", "model.windwo", "unknown key"),
        ("wrong_type", "respect_gitignore", "bool"),
        ("unknown_code", "extend_select", "SLP999"),
        (
            "unknown_param",
            "rules.SLP014.max_per_100_words",
            "unknown parameter",
        ),
        ("bad_regex", "protect.patterns[1]", "unclosed"),
    ];
    for (name, want_key, want_reason) in cases {
        let path = fixture(&format!("config/errors/{name}.toml"));
        let (file, key, reason) = config_error(&path);
        assert_eq!(file, canon(&path), "{name}: names the file");
        assert_eq!(&key, want_key, "{name}: names the key");
        assert!(
            reason.to_lowercase().contains(&want_reason.to_lowercase()),
            "{name}: reason `{reason}` should mention `{want_reason}`"
        );
        let shown = Resolver::new(CliOverrides::default(), Some(&path))
            .err()
            .unwrap()
            .to_string();
        assert!(
            shown.contains(&file.display().to_string()) && shown.contains(want_key),
            "{shown}"
        );
    }

    // The same checks run on a discovered file, and a non-numeric param and a system code are rejected too.
    let tmp = tempfile::tempdir().unwrap();
    let cases: &[(&str, &str)] = &[
        (
            "[rules.SLP014]\nmax_per_1000_words = \"five\"\n",
            "rules.SLP014.max_per_1000_words",
        ),
        ("select = [\"SLP902\"]\n", "select"),
        ("select = [\"nope\"]\n", "select"),
        ("[[per_file]]\nselect = [\"SLP001\"]\n", "per_file[0].glob"),
        ("[[per_file]]\nglob = \"a/[\"\n", "per_file[0].glob"),
        ("[model]\npath = \"m.gguf\"\n", "model.path"),
        ("min_severity = \"loud\"\n", "min_severity"),
        ("select = [\n", "(syntax)"),
    ];
    for (i, (text, want_key)) in cases.iter().enumerate() {
        let dir = tmp.path().join(format!("case{i}"));
        write(&dir.join("sloperator.toml"), text);
        let mut r = Resolver::new(CliOverrides::default(), None).unwrap();
        match r.for_file(&dir.join("doc.md")) {
            Err(Error::Config { file, key, .. }) => {
                assert_eq!(file, canon(&dir.join("sloperator.toml")), "case {i}");
                assert_eq!(&key, want_key, "case {i}: {text}");
            }
            other => panic!(
                "case {i} ({text}): expected Error::Config, got {:?}",
                other.err()
            ),
        }
    }
}

/// FR-2.13, DQ-9: `config PATH` prints the settings that apply there as JSON whose keys are snake_case, like the file.
#[test]
fn config_command_prints_resolved_json() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        &root.join("sloperator.toml"),
        "select = [\"SLP001\", \"SLP014\"]\nmin_severity = \"error\"\nrespect_gitignore = false\n",
    );
    write(&root.join("doc.md"), "text\n");

    for target in [root.join("doc.md"), root.to_path_buf()] {
        let (code, out, err) = run_cli(&["config", &target.to_string_lossy()], "");
        assert_eq!(code, ExitCode::Clean, "stderr: {err}");
        let json: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            json["source"].as_str().unwrap(),
            canon(&root.join("sloperator.toml")).to_string_lossy()
        );
        assert_eq!(json["min_severity"], "error");
        assert_eq!(json["respect_gitignore"], false);
        let rules: Vec<&String> = json["rules"].as_object().unwrap().keys().collect();
        assert_eq!(rules, ["SLP001", "SLP014"]);
        assert!(json["model"]["max_window_chars"].is_u64(), "{out}");
        assert!(
            !out.contains("minSeverity") && !out.contains("maxWindowChars"),
            "{out}"
        );
    }

    // --config replaces discovery; a bad one exits 2 naming the key.
    let bad = fixture("config/errors/unknown_key.toml");
    let (code, _, err) = run_cli(
        &[
            "config",
            "--config",
            &bad.to_string_lossy(),
            &root.to_string_lossy(),
        ],
        "",
    );
    assert_eq!(code, ExitCode::Failure, "stderr: {err}");
    assert!(err.contains("model.windwo"), "stderr: {err}");
}

#[test]
fn metrics_floors_default_to_5_and_100() {
    let tmp = tempfile::tempdir().unwrap();
    let mut r = Resolver::new(CliOverrides::default(), None).unwrap();
    assert_eq!(
        r.for_file(&tmp.path().join("doc.md")).unwrap().metrics,
        Floors {
            sentences: 5,
            words: 100
        }
    );
    let config = tmp.path().join("sloperator.toml");
    write(&config, "select = [\"SLP001\"]\n");
    let mut r = Resolver::new(CliOverrides::default(), Some(&config)).unwrap();
    assert_eq!(
        r.for_file(&tmp.path().join("doc.md")).unwrap().metrics,
        Floors::default()
    );
}

#[test]
fn metrics_floors_load_from_table() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        &root.join("sloperator.toml"),
        "[metrics]\nmin_sentences = 10\nmin_words = 200\n",
    );
    let mut r = Resolver::new(CliOverrides::default(), None).unwrap();
    for path in ["doc.md", "sub/deep/doc.md"] {
        assert_eq!(
            r.for_file(&root.join(path)).unwrap().metrics,
            Floors {
                sentences: 10,
                words: 200
            }
        );
    }
    // Each explicitly set field overrides the chain; omitted fields inherit.
    let child = root.join("child.toml");
    write(
        &child,
        "extend = \"sloperator.toml\"\n[metrics]\nmin_words = 300\n[[per_file]]\nglob = \"**/*.md\"\nmin_severity = \"error\"\n",
    );
    let mut r = Resolver::new(CliOverrides::default(), Some(&child)).unwrap();
    write(&root.join("sub/doc.md"), "text\n");
    let s = r.for_file(&root.join("sub/doc.md")).unwrap();
    assert_eq!(
        s.metrics,
        Floors {
            sentences: 10,
            words: 300
        }
    );
    assert_eq!(s.min_severity, Severity::Error);
    let small = root.join("small.toml");
    write(&small, "[metrics]\nmin_sentences = 1\nmin_words = 1\n");
    let mut r = Resolver::new(CliOverrides::default(), Some(&small)).unwrap();
    assert_eq!(
        r.for_file(&root.join("doc.md")).unwrap().metrics,
        Floors {
            sentences: 1,
            words: 1
        }
    );
}

#[test]
fn metrics_floors_reject_invalid_values() {
    let tmp = tempfile::tempdir().unwrap();
    let config = tmp.path().join("bad.toml");
    for (line, expected) in [
        ("min_sentences = 0", "metrics.min_sentences"),
        ("min_words = -5", "metrics.min_words"),
        ("min_words = 1.5", "metrics.min_words"),
        ("max = 3", "metrics.max"),
        ("min_words = \"100\"", "metrics.min_words"),
        ("min_sentences = false", "metrics.min_sentences"),
    ] {
        write(&config, &format!("[metrics]\n{line}\n"));
        let (file, key, reason) = config_error(&config);
        assert_eq!(file, canon(&config));
        assert_eq!(key, expected, "{line}: {reason}");
        assert!(!reason.is_empty());
    }
    write(
        &config,
        "[[per_file]]\nglob = \"*.md\"\n[per_file.metrics]\nmin_words = 200\n",
    );
    assert_eq!(config_error(&config).1, "per_file[0].metrics");
}

#[test]
fn metrics_floors_run_root() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let mut defaults = Resolver::new(CliOverrides::default(), None).unwrap();
    assert_eq!(defaults.run_floors(root).unwrap(), Floors::default());
    write(
        &root.join("sloperator.toml"),
        "[metrics]\nmin_sentences = 10\nmin_words = 200\n",
    );
    write(
        &root.join("sub/sloperator.toml"),
        "[metrics]\nmin_words = 300\n",
    );
    fs::create_dir_all(root.join("sub/deep")).unwrap();
    let mut r = Resolver::new(CliOverrides::default(), None).unwrap();
    assert_eq!(
        r.run_floors(root).unwrap(),
        Floors {
            sentences: 10,
            words: 200
        }
    );
    assert_eq!(
        r.run_floors(&root.join("sub/deep")).unwrap(),
        Floors {
            sentences: 5,
            words: 300
        }
    );
    // Run floors come from cwd, regardless of a previously resolved input's config.
    r.for_file(&root.join("doc.md")).unwrap();
    assert_eq!(
        r.run_floors(&root.join("sub/deep")).unwrap(),
        Floors {
            sentences: 5,
            words: 300
        }
    );
    let explicit = root.join("explicit.toml");
    write(
        &explicit,
        "extend = \"sloperator.toml\"\n[metrics]\nmin_words = 400\n",
    );
    let mut r = Resolver::new(CliOverrides::default(), Some(&explicit)).unwrap();
    assert_eq!(
        r.run_floors(&root.join("sub/deep")).unwrap(),
        Floors {
            sentences: 10,
            words: 400
        }
    );
}

#[test]
fn rejects_negative_threshold() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sloperator.toml");
    for (value, reason) in [
        ("-1", "minimum is 0"),
        ("nan", "NaN"),
        ("inf", "maximum"),
        ("-inf", "minimum"),
    ] {
        write(
            &path,
            &format!("[rules.SLP014]\nmax_per_1000_words = {value}\n"),
        );
        let (file, key, message) = config_error(&path);
        assert_eq!(file, canon(&path));
        assert_eq!(key, "rules.SLP014.max_per_1000_words");
        assert!(message.contains(reason), "{message}");
    }
}

#[test]
fn existing_params_still_load() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sloperator.toml");
    for value in [0.0, 1.0, f64::MAX] {
        write(
            &path,
            &format!(
                "[rules.SLP005]\nmax_per_1000_words = {value:?}\n[ rules.SLP010 ]\nmin_items = {value:?}\nmax_words = {value:?}\n[rules.SLP011]\nmax_per_100_words = {value:?}\n[rules.SLP014]\nmax_per_1000_words = {value:?}\n"
            ),
        );
        let mut resolver = Resolver::new(CliOverrides::default(), Some(&path)).unwrap();
        let settings = resolver.for_file(&tmp.path().join("doc.md")).unwrap();
        for code in [5, 10, 11, 14] {
            assert!(
                settings.rules[&RuleCode(code)]
                    .params
                    .values()
                    .all(|v| *v == value)
            );
        }
    }
}

#[test]
fn rejects_param_below_minimum() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sloperator.toml");
    for (name, value) in [("min_sentences", "0"), ("max_grade", "-1")] {
        write(&path, &format!("[rules.SLP018]\n{name} = {value}\n"));
        let (file, key, reason) = config_error(&path);
        assert_eq!(file, canon(&path));
        assert_eq!(key, format!("rules.SLP018.{name}"));
        assert!(reason.contains("minimum"), "{reason}");
    }
}

#[test]
fn rejects_param_above_maximum() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sloperator.toml");
    write(&path, "[rules.SLP018]\nmax_grade = 31\n");
    let (_, key, reason) = config_error(&path);
    assert_eq!(key, "rules.SLP018.max_grade");
    assert!(reason.contains("30"), "{reason}");
    write(&path, "[rules.SLP021]\nmin_ttr = 2\n");
    let (_, key, reason) = config_error(&path);
    assert_eq!(key, "rules.SLP021.min_ttr");
    assert!(reason.contains("maximum is 1"), "{reason}");
}

#[test]
fn metric_rule_minimums_follow_report_floors() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sloperator.toml");
    let text = "select = [\"SLP018\"]\n[metrics]\nmin_sentences = 10\nmin_words = 200\n";
    write(&path, text);
    let mut r = Resolver::new(CliOverrides::default(), Some(&path)).unwrap();
    let s = r.for_file(&tmp.path().join("doc.md")).unwrap();
    assert_eq!(s.rules[&RuleCode(18)].params["min_sentences"], 10.0);
    assert_eq!(s.rules[&RuleCode(18)].params["min_words"], 200.0);
    for (name, value, floor) in [("min_sentences", 6, 10), ("min_words", 150, 200)] {
        write(&path, &format!("{text}[rules.SLP018]\n{name} = {value}\n"));
        let (_, key, reason) = config_error(&path);
        assert_eq!(key, format!("rules.SLP018.{name}"));
        assert!(reason.contains(&floor.to_string()), "{reason}");
    }
    // Valid per-file overrides apply after the report floors; lower ones fail when that file resolves.
    write(
        &path,
        &format!(
            "{text}[[per_file]]\nglob = \"doc.md\"\n[per_file.rules.SLP018]\nmin_sentences = 12\n"
        ),
    );
    let mut r = Resolver::new(CliOverrides::default(), Some(&path)).unwrap();
    assert_eq!(
        r.for_file(&tmp.path().join("doc.md")).unwrap().rules[&RuleCode(18)].params["min_sentences"],
        12.0
    );
    write(
        &path,
        &format!(
            "{text}[[per_file]]\nglob = \"doc.md\"\n[per_file.rules.SLP018]\nmin_sentences = 6\n"
        ),
    );
    let mut r = Resolver::new(CliOverrides::default(), Some(&path)).unwrap();
    match r.for_file(&tmp.path().join("doc.md")) {
        Err(Error::Config { file, key, reason }) => {
            assert_eq!(file, canon(&path));
            assert_eq!(key, "rules.SLP018.min_sentences");
            assert!(reason.contains("10"));
        }
        other => panic!("expected floor error, got {other:?}"),
    }
    // A later chain layer can raise the report floor; origin points to the earlier rule param.
    write(
        &path,
        "select = [\"SLP018\"]\n[rules.SLP018]\nmin_sentences = 6\n",
    );
    let child = tmp.path().join("child.toml");
    write(
        &child,
        "extend = \"sloperator.toml\"\n[metrics]\nmin_sentences = 10\n",
    );
    let (file, key, _) = config_error(&child);
    assert_eq!(file, canon(&path));
    assert_eq!(key, "rules.SLP018.min_sentences");
}
