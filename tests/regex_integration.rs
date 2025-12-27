//! Integration tests for regex functionality
//!
//! These tests compile MicroPerl programs and run them through the Z80 emulator
//! to verify end-to-end regex behavior.

use std::process::Command;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn compile_and_run(code: &str) -> String {
    // Use unique temp files per test to avoid race conditions
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let temp_src = format!("/tmp/microperl_test_{}.pl", id);
    let temp_rom = format!("/tmp/microperl_test_{}.rom", id);

    fs::write(&temp_src, code).expect("Failed to write test file");

    // Compile
    let compile_output = Command::new("./target/release/microperl")
        .args(&[&temp_src, "--rom", &temp_rom])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run compiler");

    if !compile_output.status.success() {
        panic!(
            "Compilation failed: {}",
            String::from_utf8_lossy(&compile_output.stderr)
        );
    }

    // Run in emulator with timeout
    let run_output = Command::new("timeout")
        .args(&["2", "../emulator/retroshield", &temp_rom])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run emulator");

    // Cleanup temp files
    let _ = fs::remove_file(&temp_src);
    let _ = fs::remove_file(&temp_rom);

    String::from_utf8_lossy(&run_output.stdout).to_string()
}

// === Core regex functionality tests ===

#[test]
fn test_regex_basic_match() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s =~ /world/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Basic match should work");
}

#[test]
fn test_regex_no_match() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s =~ /xyz/) {
            print "FAIL";
        } else {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Non-matching should go to else");
}

#[test]
fn test_regex_not_match_operator() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s !~ /xyz/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "!~ should match when pattern not found");
}

#[test]
fn test_regex_not_match_fails() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s !~ /hello/) {
            print "FAIL";
        } else {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "!~ should fail when pattern found");
}

// === Wildcard tests ===

#[test]
fn test_regex_wildcard_single() {
    let output = compile_and_run(r#"
        my $s = "hello";
        if ($s =~ /h.llo/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Single wildcard should match");
}

#[test]
fn test_regex_wildcard_multiple() {
    let output = compile_and_run(r#"
        my $s = "hello";
        if ($s =~ /h...o/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Multiple wildcards should match");
}

#[test]
fn test_regex_wildcard_all() {
    let output = compile_and_run(r#"
        my $s = "hello";
        if ($s =~ /...../) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "All wildcards should match 5 chars");
}

#[test]
fn test_regex_wildcard_no_match() {
    let output = compile_and_run(r#"
        my $s = "hi";
        if ($s =~ /h..i/) {
            print "FAIL";
        } else {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Wildcard should not match if length wrong");
}

// === Position tests ===

#[test]
fn test_regex_substring_at_start() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s =~ /hello/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Match at start should work");
}

#[test]
fn test_regex_substring_at_end() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s =~ /world/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Match at end should work");
}

#[test]
fn test_regex_substring_in_middle() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s =~ /lo wo/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Match in middle should work");
}

// === Edge cases ===

#[test]
fn test_regex_empty_pattern() {
    let output = compile_and_run(r#"
        my $s = "hello";
        if ($s =~ //) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Empty pattern should match");
}

#[test]
fn test_regex_exact_match() {
    let output = compile_and_run(r#"
        my $s = "hello";
        if ($s =~ /hello/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Exact match should work");
}

#[test]
fn test_regex_pattern_longer_than_string() {
    let output = compile_and_run(r#"
        my $s = "hi";
        if ($s =~ /hello/) {
            print "FAIL";
        } else {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Pattern longer than string should not match");
}

#[test]
fn test_regex_single_char_match() {
    let output = compile_and_run(r#"
        my $s = "a";
        if ($s =~ /a/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Single char match should work");
}

#[test]
fn test_regex_single_wildcard_match() {
    let output = compile_and_run(r#"
        my $s = "x";
        if ($s =~ /./) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Single wildcard should match any char");
}

// === Combined logic tests ===

#[test]
fn test_regex_multiple_conditions_and() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s =~ /hello/ && $s =~ /world/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Multiple AND conditions should work");
}

#[test]
fn test_regex_multiple_conditions_or() {
    let output = compile_and_run(r#"
        my $s = "hello";
        if ($s =~ /xyz/ || $s =~ /hello/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Multiple OR conditions should work");
}

#[test]
fn test_regex_mixed_match_not_match() {
    let output = compile_and_run(r#"
        my $s = "hello world";
        if ($s =~ /hello/ && $s !~ /xyz/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Mixed =~ and !~ should work");
}

// === Loop tests ===

#[test]
fn test_regex_in_loop_condition() {
    // This would be an infinite loop if the match worked,
    // so we test that pattern NOT matching exits the loop
    let output = compile_and_run(r#"
        my $s = "done";
        my $count = 0;
        while ($s =~ /run/ && $count < 5) {
            $count = $count + 1;
        }
        print "PASS";
    "#);
    assert!(output.contains("PASS"), "Regex in while condition should work");
}

// === String literal tests ===

#[test]
fn test_regex_on_string_literal() {
    let output = compile_and_run(r#"
        if ("hello" =~ /ell/) {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Match on string literal should work");
}

// === Case sensitivity test ===

#[test]
fn test_regex_case_sensitive() {
    let output = compile_and_run(r#"
        my $s = "Hello";
        if ($s =~ /hello/) {
            print "FAIL";
        } else {
            print "PASS";
        }
    "#);
    assert!(output.contains("PASS"), "Match should be case sensitive");
}
