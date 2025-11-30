//! UI snapshot tests for display features (sux.7.2)
//!
//! Tests for:
//! - sux.6.2: Enhanced match highlighting
//! - sux.6.3: Alternating color stripes
//! - Theme consistency across all presets

use assert_cmd::cargo::cargo_bin_cmd;
use coding_agent_search::ui::components::theme::{ThemePalette, ThemePreset};
use ratatui::style::{Color, Modifier};

#[test]
fn cli_shows_help() {
    let mut cmd = cargo_bin_cmd!("cass");
    cmd.arg("--help").assert().success();
}

// ============================================================
// sux.6.2: Enhanced Match Highlighting Tests
// ============================================================

#[test]
fn highlight_style_has_background_color() {
    // Test that highlight_style provides both fg and bg colors (sux.6.2)
    let palette = ThemePalette::dark();
    let style = palette.highlight_style();

    // Style should have background set (not None)
    assert!(
        style.bg.is_some(),
        "highlight_style should have background color for visibility"
    );
    assert!(
        style.fg.is_some(),
        "highlight_style should have foreground color"
    );
    assert!(
        style.add_modifier.contains(Modifier::BOLD),
        "highlight_style should be bold"
    );
}

#[test]
fn highlight_style_is_theme_aware() {
    // Test that different themes have different highlight colors (sux.6.2)
    let dark = ThemePalette::dark();
    let light = ThemePalette::light();

    let dark_style = dark.highlight_style();
    let light_style = light.highlight_style();

    // Dark and light themes should have different bg colors
    assert_ne!(
        dark_style.bg, light_style.bg,
        "Dark and light themes should have different highlight backgrounds"
    );
}

#[test]
fn all_themes_have_valid_highlight_style() {
    // Ensure all theme presets have valid highlight styles
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();
        let style = palette.highlight_style();

        assert!(
            style.bg.is_some(),
            "{:?} theme should have highlight background",
            preset
        );
        assert!(
            style.fg.is_some(),
            "{:?} theme should have highlight foreground",
            preset
        );
    }
}

// ============================================================
// sux.6.3: Alternating Color Stripes Tests
// ============================================================

#[test]
fn stripe_colors_are_distinct() {
    // Test that stripe_even and stripe_odd are different colors (sux.6.3)
    let palette = ThemePalette::dark();

    assert_ne!(
        palette.stripe_even, palette.stripe_odd,
        "Stripe colors should be distinct for zebra-striping effect"
    );
}

#[test]
fn stripe_even_matches_background() {
    // stripe_even should typically be same or very close to bg
    let dark = ThemePalette::dark();
    assert_eq!(
        dark.stripe_even, dark.bg,
        "Dark theme stripe_even should match background"
    );

    let light = ThemePalette::light();
    assert_eq!(
        light.stripe_even, light.bg,
        "Light theme stripe_even should match background"
    );
}

#[test]
fn all_themes_have_stripe_colors() {
    // Ensure all theme presets have stripe colors defined
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();

        // Stripes should be valid colors (not default)
        assert_ne!(
            palette.stripe_even,
            Color::Reset,
            "{:?} theme should have stripe_even color",
            preset
        );
        assert_ne!(
            palette.stripe_odd,
            Color::Reset,
            "{:?} theme should have stripe_odd color",
            preset
        );

        // Stripes should be distinct
        assert_ne!(
            palette.stripe_even, palette.stripe_odd,
            "{:?} theme should have distinct stripe colors",
            preset
        );
    }
}

#[test]
fn stripe_colors_have_subtle_contrast() {
    // Stripe colors should be similar but distinct - test RGB proximity
    let palette = ThemePalette::dark();

    if let (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) =
        (palette.stripe_even, palette.stripe_odd)
    {
        // Calculate approximate color distance
        let dr = (r1 as i32 - r2 as i32).abs();
        let dg = (g1 as i32 - g2 as i32).abs();
        let db = (b1 as i32 - b2 as i32).abs();
        let distance = dr + dg + db;

        // Should be subtle (not too far apart)
        assert!(
            distance < 100,
            "Stripe colors should be subtle (distance={distance}), not jarring"
        );
        // But should be visible (not identical)
        assert!(
            distance > 5,
            "Stripe colors should be visibly different (distance={distance})"
        );
    }
}

// ============================================================
// Theme Consistency Tests
// ============================================================

#[test]
fn theme_preset_cycle_is_complete() {
    // Test that cycling through themes covers all presets
    let mut current = ThemePreset::Dark;
    let mut visited = vec![current];

    for _ in 0..10 {
        current = current.next();
        if current == ThemePreset::Dark {
            break;
        }
        visited.push(current);
    }

    assert_eq!(
        visited.len(),
        ThemePreset::all().len(),
        "Theme cycle should visit all presets exactly once"
    );
}

#[test]
fn all_themes_have_role_colors() {
    // Test that all themes have distinct role colors
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();

        // User, agent, tool, system should be different colors
        assert_ne!(
            palette.user, palette.agent,
            "{:?}: user and agent colors should differ",
            preset
        );
        assert_ne!(
            palette.tool, palette.system,
            "{:?}: tool and system colors should differ",
            preset
        );
    }
}

#[test]
fn high_contrast_theme_has_pure_colors() {
    // High contrast should use extreme values for accessibility
    let hc = ThemePalette::high_contrast();

    // Background should be pure black
    assert_eq!(
        hc.bg,
        Color::Rgb(0, 0, 0),
        "High contrast background should be pure black"
    );

    // Foreground should be pure white
    assert_eq!(
        hc.fg,
        Color::Rgb(255, 255, 255),
        "High contrast foreground should be pure white"
    );

    // Stripes should also use high contrast
    assert_eq!(
        hc.stripe_even,
        Color::Rgb(0, 0, 0),
        "High contrast stripe_even should be pure black"
    );
}

// ============================================================
// 008: Role-Aware Theming Tests
// ============================================================

use coding_agent_search::ui::components::theme::{
    AdaptiveBorders, ContrastLevel, GradientShades, TerminalWidth, check_contrast, contrast_ratio,
};

#[test]
fn role_theme_returns_complete_styling() {
    // Test that role_theme provides all style components
    let palette = ThemePalette::dark();

    for role in &["user", "assistant", "tool", "system"] {
        let theme = palette.role_theme(role);

        // All fields should be valid colors (not Reset)
        assert_ne!(theme.fg, Color::Reset, "{role} should have fg color");
        assert_ne!(theme.bg, Color::Reset, "{role} should have bg color");
        assert_ne!(
            theme.border,
            Color::Reset,
            "{role} should have border color"
        );
        assert_ne!(theme.badge, Color::Reset, "{role} should have badge color");
    }
}

#[test]
fn role_theme_has_distinct_backgrounds() {
    // Each role should have a different background tint
    let palette = ThemePalette::dark();

    let user_bg = palette.role_theme("user").bg;
    let agent_bg = palette.role_theme("assistant").bg;
    let tool_bg = palette.role_theme("tool").bg;
    let system_bg = palette.role_theme("system").bg;

    // All backgrounds should be distinct
    assert_ne!(user_bg, agent_bg, "user and agent should have different bg");
    assert_ne!(
        tool_bg, system_bg,
        "tool and system should have different bg"
    );
    assert_ne!(user_bg, tool_bg, "user and tool should have different bg");
}

#[test]
fn gradient_shades_header_has_depth() {
    // Header gradient should have distinct shades for depth effect
    let gradient = GradientShades::header();

    // Dark, mid, and light should all be different
    assert_ne!(gradient.dark, gradient.mid, "dark and mid should differ");
    assert_ne!(gradient.mid, gradient.light, "mid and light should differ");
    assert_ne!(
        gradient.dark, gradient.light,
        "dark and light should differ"
    );
}

#[test]
fn gradient_shades_pill_creates_centered_effect() {
    // Pill gradient should have darker edges and lighter center
    let gradient = GradientShades::pill();

    // Left and right should be similar (darker edges)
    assert_eq!(gradient.dark, gradient.light, "pill edges should match");

    // Center (mid) should be different (lighter)
    assert_ne!(
        gradient.mid, gradient.dark,
        "pill center should differ from edges"
    );
}

#[test]
fn gradient_shades_styles_returns_three_styles() {
    let gradient = GradientShades::header();
    let (dark_style, mid_style, light_style) = gradient.styles();

    // Each style should have a background set
    assert!(dark_style.bg.is_some(), "dark style should have bg");
    assert!(mid_style.bg.is_some(), "mid style should have bg");
    assert!(light_style.bg.is_some(), "light style should have bg");
}

// ============================================================
// 008: Terminal Width Adaptive Styling Tests
// ============================================================

#[test]
fn terminal_width_classification() {
    assert_eq!(TerminalWidth::from_cols(60), TerminalWidth::Narrow);
    assert_eq!(TerminalWidth::from_cols(79), TerminalWidth::Narrow);
    assert_eq!(TerminalWidth::from_cols(80), TerminalWidth::Normal);
    assert_eq!(TerminalWidth::from_cols(100), TerminalWidth::Normal);
    assert_eq!(TerminalWidth::from_cols(120), TerminalWidth::Normal);
    assert_eq!(TerminalWidth::from_cols(121), TerminalWidth::Wide);
    assert_eq!(TerminalWidth::from_cols(200), TerminalWidth::Wide);
}

#[test]
fn terminal_width_decorations() {
    assert!(!TerminalWidth::Narrow.show_decorations());
    assert!(TerminalWidth::Normal.show_decorations());
    assert!(TerminalWidth::Wide.show_decorations());

    assert!(!TerminalWidth::Narrow.show_extended_info());
    assert!(!TerminalWidth::Normal.show_extended_info());
    assert!(TerminalWidth::Wide.show_extended_info());
}

#[test]
fn adaptive_borders_for_different_widths() {
    let narrow = AdaptiveBorders::for_width(60);
    let normal = AdaptiveBorders::for_width(100);
    let wide = AdaptiveBorders::for_width(150);

    // Narrow should have minimal styling
    assert_eq!(narrow.width_class, TerminalWidth::Narrow);
    assert!(!narrow.use_double);
    assert!(!narrow.show_corners);

    // Normal should have standard styling
    assert_eq!(normal.width_class, TerminalWidth::Normal);
    assert!(!normal.use_double);
    assert!(normal.show_corners);

    // Wide should have enhanced styling
    assert_eq!(wide.width_class, TerminalWidth::Wide);
    assert!(wide.use_double);
    assert!(wide.show_corners);
}

#[test]
fn adaptive_borders_focused_has_focus_color() {
    use coding_agent_search::ui::components::theme::colors;

    let focused = AdaptiveBorders::focused(100);
    assert_eq!(focused.color, colors::BORDER_FOCUS);
}

// ============================================================
// 008: Contrast Compliance Tests
// ============================================================

#[test]
fn contrast_ratio_black_white() {
    // Black and white should have maximum contrast (21:1)
    let ratio = contrast_ratio(Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0));
    assert!(
        ratio > 20.0 && ratio <= 21.0,
        "black/white ratio should be ~21:1, got {ratio}"
    );
}

#[test]
fn contrast_ratio_same_color() {
    // Same color should have ratio of 1:1
    let ratio = contrast_ratio(Color::Rgb(128, 128, 128), Color::Rgb(128, 128, 128));
    assert!(
        (ratio - 1.0).abs() < 0.01,
        "same color ratio should be 1:1, got {ratio}"
    );
}

#[test]
fn contrast_level_classification() {
    assert_eq!(ContrastLevel::from_ratio(2.5), ContrastLevel::Fail);
    assert_eq!(ContrastLevel::from_ratio(3.0), ContrastLevel::AALarge);
    assert_eq!(ContrastLevel::from_ratio(4.0), ContrastLevel::AALarge);
    assert_eq!(ContrastLevel::from_ratio(4.5), ContrastLevel::AA);
    assert_eq!(ContrastLevel::from_ratio(6.5), ContrastLevel::AA);
    assert_eq!(ContrastLevel::from_ratio(7.0), ContrastLevel::AAA);
    assert_eq!(ContrastLevel::from_ratio(10.0), ContrastLevel::AAA);
}

#[test]
fn contrast_level_meets_requirement() {
    let aaa = ContrastLevel::AAA;
    let aa = ContrastLevel::AA;
    let fail = ContrastLevel::Fail;

    assert!(aaa.meets(ContrastLevel::AA), "AAA should meet AA");
    assert!(aaa.meets(ContrastLevel::AAA), "AAA should meet AAA");
    assert!(aa.meets(ContrastLevel::AA), "AA should meet AA");
    assert!(!aa.meets(ContrastLevel::AAA), "AA should not meet AAA");
    assert!(!fail.meets(ContrastLevel::AA), "Fail should not meet AA");
}

#[test]
fn high_contrast_theme_meets_wcag_aaa() {
    // High contrast theme should meet WCAG AAA standards
    let hc = ThemePalette::high_contrast();
    let level = check_contrast(hc.fg, hc.bg);
    assert!(
        level.meets(ContrastLevel::AAA),
        "High contrast theme should meet WCAG AAA, got {:?}",
        level
    );
}

#[test]
fn all_themes_meet_wcag_aa_for_text() {
    // All themes should meet at least WCAG AA for primary text
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();
        let level = check_contrast(palette.fg, palette.bg);
        assert!(
            level.meets(ContrastLevel::AA),
            "{:?} theme should meet WCAG AA for fg/bg contrast, got {:?}",
            preset,
            level
        );
    }
}

// ============================================================
// pmb.2: In-Detail Highlighting Tests
// ============================================================

#[test]
fn detail_highlight_style_has_required_attributes() {
    // Detail-find highlighting must be visible: bg + fg + bold (pmb.2)
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();
        let style = palette.highlight_style();

        // Must have all three: background, foreground, and bold
        assert!(
            style.bg.is_some(),
            "{:?}: detail highlight needs bg for visibility",
            preset
        );
        assert!(
            style.fg.is_some(),
            "{:?}: detail highlight needs fg for readability",
            preset
        );
        assert!(
            style.add_modifier.contains(Modifier::BOLD),
            "{:?}: detail highlight should be bold",
            preset
        );
    }
}

#[test]
fn detail_highlight_contrasts_with_background() {
    // Highlight style must be visible against the theme background (pmb.2)
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();
        let style = palette.highlight_style();

        if let Some(highlight_bg) = style.bg {
            // Highlight background should differ from regular background
            assert_ne!(
                highlight_bg, palette.bg,
                "{:?}: highlight bg should differ from regular bg",
                preset
            );
        }
    }
}

#[test]
fn detail_highlight_uses_themed_accent() {
    // Highlight may use accent/brand color for consistency (pmb.2)
    // This test verifies the highlight styling is intentional, not accidental
    let palette = ThemePalette::dark();
    let highlight_style = palette.highlight_style();

    // Highlight should have a defined background (may be accent or dedicated color)
    assert!(
        highlight_style.bg.is_some(),
        "Highlight should have explicit background color"
    );

    // The foreground should be dark (readable on highlight bg)
    if let Some(Color::Rgb(r, g, b)) = highlight_style.fg {
        // Dark fg (black or near-black) for readability on colored bg
        let luminance = r as u32 + g as u32 + b as u32;
        assert!(
            luminance < 200,
            "Highlight fg should be dark for readability (got luminance {luminance})"
        );
    }
}

#[test]
fn all_themes_have_consistent_highlight_fg() {
    // All themes should use a readable fg color on the highlight bg
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();
        let style = palette.highlight_style();

        // Check that fg is set and not reset
        assert!(
            style.fg.is_some() && style.fg != Some(Color::Reset),
            "{:?}: highlight fg should be explicitly set, not Reset",
            preset
        );
    }
}

#[test]
fn highlight_style_bg_uses_accent_color() {
    // Highlight background should use the theme's accent color for brand consistency
    // (not necessarily yellow - depends on theme design)
    let dark = ThemePalette::dark();
    let dark_style = dark.highlight_style();

    // Verify highlight bg matches accent
    assert_eq!(
        dark_style.bg,
        Some(dark.accent),
        "Dark theme highlight bg should use accent color"
    );

    // Accent should be a saturated, visible color (not too dark/light)
    if let Some(Color::Rgb(r, g, b)) = dark_style.bg {
        let max_channel = r.max(g).max(b);
        let min_channel = r.min(g).min(b);
        let saturation_proxy = max_channel.saturating_sub(min_channel);

        // Should have some color saturation (not gray)
        assert!(
            saturation_proxy > 50,
            "Highlight bg should be saturated, not gray (got r={r}, g={g}, b={b})"
        );
    }
}

#[test]
fn detail_highlight_meets_aa_large_contrast() {
    // For accessibility, highlight fg on highlight bg should be readable (pmb.2)
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();
        let style = palette.highlight_style();

        if let (Some(fg), Some(bg)) = (style.fg, style.bg) {
            let level = check_contrast(fg, bg);
            // At minimum, should meet AA for large text (3:1 ratio)
            assert!(
                level.meets(ContrastLevel::AALarge),
                "{:?}: highlight fg/bg should meet at least AA-large contrast, got {:?}",
                preset,
                level
            );
        }
    }
}

#[test]
fn role_themes_support_highlight_overlay() {
    // Role backgrounds (user/agent/tool/system) should contrast with highlight (pmb.2)
    let palette = ThemePalette::dark();
    let highlight_style = palette.highlight_style();

    for role in &["user", "assistant", "tool", "system"] {
        let role_theme = palette.role_theme(role);

        if let Some(highlight_bg) = highlight_style.bg {
            // Highlight bg should be distinct from role bg
            assert_ne!(
                highlight_bg, role_theme.bg,
                "Highlight should be visible on {} role background",
                role
            );
        }
    }
}

#[test]
fn stripe_colors_allow_highlight_visibility() {
    // Highlight should be visible on both stripe colors (pmb.2)
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();
        let highlight_style = palette.highlight_style();

        if let Some(highlight_bg) = highlight_style.bg {
            // Highlight bg should differ from both stripe colors
            assert_ne!(
                highlight_bg, palette.stripe_even,
                "{:?}: highlight should be visible on stripe_even",
                preset
            );
            assert_ne!(
                highlight_bg, palette.stripe_odd,
                "{:?}: highlight should be visible on stripe_odd",
                preset
            );
        }
    }
}

// ============================================================
// yln.3: UI Interaction Mode Tests
// ============================================================

use coding_agent_search::model::types::MessageRole;
use coding_agent_search::ui::data::{role_style, InputMode};

#[test]
fn input_mode_has_detail_find_variant() {
    // Verify DetailFind mode exists for in-detail search (yln.3)
    let mode = InputMode::DetailFind;
    assert_eq!(mode, InputMode::DetailFind);

    // All modes should be distinct
    assert_ne!(InputMode::Query, InputMode::DetailFind);
    assert_ne!(InputMode::Agent, InputMode::DetailFind);
    assert_ne!(InputMode::Workspace, InputMode::DetailFind);
    assert_ne!(InputMode::PaneFilter, InputMode::DetailFind);
}

#[test]
fn input_mode_covers_all_filter_types() {
    // Verify all input modes for filtering exist (yln.3)
    let modes = [
        InputMode::Query,
        InputMode::Agent,
        InputMode::Workspace,
        InputMode::CreatedFrom,
        InputMode::CreatedTo,
        InputMode::PaneFilter,
        InputMode::DetailFind,
    ];

    // All should be distinct
    for (i, mode_a) in modes.iter().enumerate() {
        for (j, mode_b) in modes.iter().enumerate() {
            if i != j {
                assert_ne!(mode_a, mode_b, "Modes at {} and {} should differ", i, j);
            }
        }
    }
}

#[test]
fn role_style_returns_distinct_colors_for_roles() {
    // Each message role should have distinct styling (yln.3)
    let palette = ThemePalette::dark();

    let user_style = role_style(&MessageRole::User, palette);
    let agent_style = role_style(&MessageRole::Agent, palette);
    let tool_style = role_style(&MessageRole::Tool, palette);
    let system_style = role_style(&MessageRole::System, palette);

    // User and Agent should be distinct
    assert_ne!(
        user_style.fg, agent_style.fg,
        "User and Agent should have different colors"
    );

    // Tool and System should be distinct
    assert_ne!(
        tool_style.fg, system_style.fg,
        "Tool and System should have different colors"
    );
}

#[test]
fn role_style_is_theme_consistent() {
    // role_style should use theme palette colors (yln.3)
    let palette = ThemePalette::dark();

    let user_style = role_style(&MessageRole::User, palette);
    let agent_style = role_style(&MessageRole::Agent, palette);

    // User style should match palette.user
    assert_eq!(
        user_style.fg,
        Some(palette.user),
        "User role should use palette.user color"
    );

    // Agent style should match palette.agent
    assert_eq!(
        agent_style.fg,
        Some(palette.agent),
        "Agent role should use palette.agent color"
    );
}

#[test]
fn role_style_handles_other_role() {
    // Other/unknown roles should get hint styling (yln.3)
    let palette = ThemePalette::dark();

    let other_style = role_style(&MessageRole::Other("custom".into()), palette);

    // Should use hint color (not crash, not be empty)
    assert_eq!(
        other_style.fg,
        Some(palette.hint),
        "Other role should use palette.hint color"
    );
}

#[test]
fn role_style_all_themes_provide_valid_colors() {
    // All theme presets should provide valid role colors (yln.3)
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();

        for role in &[
            MessageRole::User,
            MessageRole::Agent,
            MessageRole::Tool,
            MessageRole::System,
        ] {
            let style = role_style(role, palette);
            assert!(
                style.fg.is_some(),
                "{:?} preset should provide fg color for {:?}",
                preset,
                role
            );

            // Color should not be Reset
            assert_ne!(
                style.fg,
                Some(Color::Reset),
                "{:?}: {:?} role should have explicit color",
                preset,
                role
            );
        }
    }
}

#[test]
fn role_colors_are_wcag_readable() {
    // Role colors should be readable against theme background (yln.3)
    for preset in ThemePreset::all() {
        let palette = preset.to_palette();

        for role in &[
            MessageRole::User,
            MessageRole::Agent,
            MessageRole::Tool,
            MessageRole::System,
        ] {
            let style = role_style(role, palette);
            if let Some(fg) = style.fg {
                let level = check_contrast(fg, palette.bg);
                assert!(
                    level.meets(ContrastLevel::AALarge),
                    "{:?}: {:?} role color should meet WCAG AA-large on bg, got {:?}",
                    preset,
                    role,
                    level
                );
            }
        }
    }
}
