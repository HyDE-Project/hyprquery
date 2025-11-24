use std::{fs, io::Write, path::PathBuf};

fn create_test_config(name: &str, content: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir().join("hydequery_integration");
    let _ = fs::create_dir_all(&temp_dir);
    let path = temp_dir.join(name);
    let mut file = fs::File::create(&path).unwrap();
    write!(file, "{}", content).unwrap();
    path
}

fn cleanup_test_config(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_parse_theme_variables() {
    let content = r#"
$GTK_THEME = Gruvbox-Retro
$ICON_THEME = Gruvbox-Plus-Dark
$CURSOR_THEME = Gruvbox-Retro
$CURSOR_SIZE = 20
$COLOR_SCHEME = prefer-dark
"#;
    let path = create_test_config("theme.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    assert_eq!(config.get_variable("GTK_THEME").unwrap(), "Gruvbox-Retro");
    assert_eq!(
        config.get_variable("ICON_THEME").unwrap(),
        "Gruvbox-Plus-Dark"
    );
    assert_eq!(
        config.get_variable("CURSOR_THEME").unwrap(),
        "Gruvbox-Retro"
    );
    assert_eq!(config.get_variable("CURSOR_SIZE").unwrap(), "20");
    assert_eq!(config.get_variable("COLOR_SCHEME").unwrap(), "prefer-dark");

    cleanup_test_config(&path);
}

#[test]
fn test_parse_general_settings() {
    let content = r#"
general {
    gaps_in = 3
    gaps_out = 8
    border_size = 2
    layout = dwindle
    resize_on_border = true
}
"#;
    let path = create_test_config("general.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    let gaps_in = config.get("general:gaps_in").unwrap();
    assert!(matches!(gaps_in, hyprlang::ConfigValue::Int(3)));

    let gaps_out = config.get("general:gaps_out").unwrap();
    assert!(matches!(gaps_out, hyprlang::ConfigValue::Int(8)));

    let border_size = config.get("general:border_size").unwrap();
    assert!(matches!(border_size, hyprlang::ConfigValue::Int(2)));

    cleanup_test_config(&path);
}

#[test]
fn test_parse_decoration_settings() {
    let content = r#"
decoration {
    rounding = 3
    shadow:enabled = false

    blur {
        enabled = yes
        size = 4
        passes = 2
    }
}
"#;
    let path = create_test_config("decoration.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    let rounding = config.get("decoration:rounding").unwrap();
    assert!(matches!(rounding, hyprlang::ConfigValue::Int(3)));

    cleanup_test_config(&path);
}

#[test]
fn test_parse_group_settings() {
    let content = r#"
group {
    col.border_active = rgba(90ceaaff) rgba(ecd3a0ff) 45deg
    col.border_inactive = rgba(1e8b50d9) rgba(50b050d9) 45deg
}
"#;
    let path = create_test_config("group.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    cleanup_test_config(&path);
}

#[test]
fn test_source_with_relative_path() {
    let temp_dir = std::env::temp_dir().join("hydequery_source_integration");
    let _ = fs::create_dir_all(&temp_dir);

    let theme_path = temp_dir.join("theme.conf");
    let mut theme_file = fs::File::create(&theme_path).unwrap();
    writeln!(theme_file, "$GTK_THEME = Wallbash-Gtk").unwrap();

    let main_path = temp_dir.join("main.conf");
    let mut main_file = fs::File::create(&main_path).unwrap();
    writeln!(main_file, "source = ./theme.conf").unwrap();

    let mut config = hyprlang::Config::with_options(hyprlang::ConfigOptions {
        base_dir: Some(temp_dir.clone()),
        ..Default::default()
    });
    config.parse_file(&main_path).unwrap();

    let _ = fs::remove_file(theme_path);
    let _ = fs::remove_file(main_path);
    let _ = fs::remove_dir(temp_dir);
}

#[test]
fn test_parse_input_settings() {
    let content = r#"
input {
    kb_layout = us
    follow_mouse = 1
    sensitivity = 0
    touchpad {
        natural_scroll = false
    }
}
"#;
    let path = create_test_config("input.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    cleanup_test_config(&path);
}

#[test]
fn test_parse_animations() {
    let content = r#"
animations {
    enabled = yes
    bezier = wind, 0.05, 0.9, 0.1, 1.05
}
"#;
    let path = create_test_config("animations.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    cleanup_test_config(&path);
}

#[test]
fn test_multiple_variables_same_file() {
    let content = r#"
$terminal = kitty
$fileManager = dolphin
$menu = rofi -show drun

general {
    gaps_in = 5
    gaps_out = 10
}
"#;
    let path = create_test_config("userprefs.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    assert_eq!(config.get_variable("terminal").unwrap(), "kitty");
    assert_eq!(config.get_variable("fileManager").unwrap(), "dolphin");

    cleanup_test_config(&path);
}

#[test]
fn test_hyde_marker_variable() {
    let content = r#"
$HYDE_HYPRLAND=set
"#;
    let path = create_test_config("hyprland.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    assert_eq!(config.get_variable("HYDE_HYPRLAND").unwrap(), "set");

    cleanup_test_config(&path);
}

#[test]
fn test_layerrule_directive() {
    let content = r#"
layerrule = blur,waybar
layerrule = blur,rofi
"#;
    let path = create_test_config("layerrules.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    cleanup_test_config(&path);
}

#[test]
fn test_windowrule_directive() {
    let content = r#"
windowrulev2 = float,class:^(pavucontrol)$
windowrulev2 = float,class:^(blueman-manager)$
"#;
    let path = create_test_config("windowrules.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    cleanup_test_config(&path);
}

#[test]
fn test_monitor_configuration() {
    let content = r#"
monitor = ,preferred,auto,1
"#;
    let path = create_test_config("monitors.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    cleanup_test_config(&path);
}

#[test]
fn test_empty_config() {
    let content = "";
    let path = create_test_config("empty.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    cleanup_test_config(&path);
}

#[test]
fn test_comments_only_config() {
    let content = r#"
# This is a comment
# Another comment

# Comment with empty lines above
"#;
    let path = create_test_config("comments.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    cleanup_test_config(&path);
}

#[test]
fn test_mixed_content() {
    let content = r#"
# Theme configuration
$GTK_THEME = Gruvbox-Retro

general {
    # Gap settings
    gaps_in = 3
    gaps_out = 8
    border_size = 2
}

# Decoration settings
decoration {
    rounding = 3
}
"#;
    let path = create_test_config("mixed.conf", content);

    let mut config = hyprlang::Config::default();
    config.parse_file(&path).unwrap();

    assert_eq!(config.get_variable("GTK_THEME").unwrap(), "Gruvbox-Retro");

    let gaps_in = config.get("general:gaps_in").unwrap();
    assert!(matches!(gaps_in, hyprlang::ConfigValue::Int(3)));

    cleanup_test_config(&path);
}

#[test]
fn test_binary_help_flag() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--release", "--", "-h"])
        .output()
        .expect("Failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE") || stdout.contains("hydequery"));
}

#[test]
fn test_binary_query_theme_variable() {
    let content = "$GTK_THEME = Gruvbox-Retro";
    let path = create_test_config("binary_theme.conf", content);

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            path.to_str().unwrap(),
            "-Q",
            "$GTK_THEME"
        ])
        .output()
        .expect("Failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Gruvbox-Retro"));

    cleanup_test_config(&path);
}

#[test]
fn test_binary_query_nested_key() {
    let content = r#"
general {
    border_size = 2
}
"#;
    let path = create_test_config("binary_nested.conf", content);

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            path.to_str().unwrap(),
            "-Q",
            "general:border_size"
        ])
        .output()
        .expect("Failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2"));

    cleanup_test_config(&path);
}

#[test]
fn test_binary_json_export() {
    let content = "$CURSOR_SIZE = 24";
    let path = create_test_config("binary_json.conf", content);

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            path.to_str().unwrap(),
            "-Q",
            "$CURSOR_SIZE",
            "--export",
            "json"
        ])
        .output()
        .expect("Failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"key\""));
    assert!(stdout.contains("\"value\""));
    assert!(stdout.contains("24"));

    cleanup_test_config(&path);
}

#[test]
fn test_binary_env_export() {
    let content = "$COLOR_SCHEME = prefer-dark";
    let path = create_test_config("binary_env.conf", content);

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            path.to_str().unwrap(),
            "-Q",
            "$COLOR_SCHEME",
            "--export",
            "env"
        ])
        .output()
        .expect("Failed to run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("COLOR_SCHEME="));
    assert!(stdout.contains("prefer-dark"));

    cleanup_test_config(&path);
}

#[test]
fn test_binary_missing_variable_exit_code() {
    let content = "$GTK_THEME = Gruvbox-Retro";
    let path = create_test_config("binary_missing.conf", content);

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            path.to_str().unwrap(),
            "-Q",
            "$NONEXISTENT"
        ])
        .output()
        .expect("Failed to run binary");

    assert!(!output.status.success());

    cleanup_test_config(&path);
}

#[test]
fn test_binary_allow_missing_flag() {
    let content = "$GTK_THEME = Gruvbox-Retro";
    let path = create_test_config("binary_allow.conf", content);

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            path.to_str().unwrap(),
            "-Q",
            "$NONEXISTENT",
            "--allow-missing"
        ])
        .output()
        .expect("Failed to run binary");

    assert!(output.status.success());

    cleanup_test_config(&path);
}

#[test]
fn test_binary_config_not_found() {
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            "/nonexistent/config.conf",
            "-Q",
            "$GTK_THEME"
        ])
        .output()
        .expect("Failed to run binary");

    assert!(!output.status.success());
}
