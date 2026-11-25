//! CLI behaviour that does not touch a keyboard.

use assert_cmd::Command;

fn kontroll() -> Command {
    Command::cargo_bin("kontroll").unwrap()
}

#[test]
fn connect_commands_are_deprecated_no_ops() {
    for args in [&["connect", "-i", "1"][..], &["connect-any"], &["disconnect"]] {
        let output = kontroll().args(args).output().unwrap();
        assert!(output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("deprecated"), "{args:?}: {stderr}");
    }
}

#[test]
fn port_is_accepted_and_ignored() {
    let output = kontroll()
        .args(["-p", "50051", "connect-any"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--port is deprecated"), "{stderr}");
}

#[test]
fn an_invalid_color_fails_before_opening_a_keyboard() {
    let output = kontroll()
        .args(["set-rgb-all", "-c", "#12"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("not a valid hex color"), "{stderr}");
}

#[test]
fn keyboard_is_a_global_option() {
    kontroll()
        .args(["connect-any", "--keyboard", "2"])
        .assert()
        .success();
    kontroll()
        .args(["-k", "2", "connect-any"])
        .assert()
        .success();
}
