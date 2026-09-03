use std::process::Command;

#[test]
fn placeholder_mode_generates_valid_ts() {
    let output = Command::new(env!("CARGO_BIN_EXE_fa-vendor"))
        .arg("tests/sample-icons.toml")
        .arg("--placeholders")
        .output()
        .expect("failed to run fa-vendor");

    assert!(
        output.status.success(),
        "fa-vendor failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("export const fa_house_solid ="));
    assert!(stdout.contains("export const fa_github_brands ="));
    assert!(stdout.contains("export const fa_star_solid_notdog ="));
    assert!(stdout.contains("export const fa_arrow_up_right_from_square_regular ="));
    assert!(stdout.contains("export const fa_500px_brands ="));
    assert!(stdout.contains("export type FaIconName ="));
    assert!(stdout.contains("export const faIcons: Record<FaIconName, string> ="));
}
