use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::widgets::panel_block;

pub fn render_armoury<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let is_narrow = area.width < 100;

    let chunks = if is_narrow {
        vec![area]
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area)
            .to_vec()
    };

    // -------------------------------------------------------------------------
    // Left Pane: Grouped Attributes
    // -------------------------------------------------------------------------
    let attr_block = panel_block(" Firmware Attributes ", true, theme);
    let attr_inner = attr_block.inner(chunks[0]);
    f.render_widget(attr_block, chunks[0]);

    let mut lines = Vec::new();
    let categories = ["Display", "GPU", "Power limits", "Battery", "General"];

    let unavailable_count = app.armoury_attributes.iter().filter(|a| !a.is_available).count();

    for cat in categories {
        let items: Vec<(usize, &crate::backend::types::ArmouryAttribute)> = app
            .armoury_attributes
            .iter()
            .enumerate()
            .filter(|(_, a)| a.category() == cat && (a.is_available || app.show_unavailable_armoury))
            .collect();

        if items.is_empty() {
            continue;
        }

        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            format!("  -- {cat} --"),
            Style::default().fg(theme.dim),
        )));

        for (idx, attr) in items {
            let is_selected = app.armoury_selected_idx == idx;
            let display_val = attr.display_value();
            let na_tag = if !attr.is_available { " [N/A]" } else { "" };

            let row_str = format!("  {: <28}: {display_val}{na_tag}", attr.display_name);
            let w = attr_inner.width as usize;
            let padded = if row_str.len() < w {
                format!("{}{}", row_str, " ".repeat(w - row_str.len()))
            } else {
                row_str
            };

            if is_selected {
                lines.push(Line::from(Span::styled(padded, theme.sel_style())));
            } else if !attr.is_available {
                lines.push(Line::from(Span::styled(padded, Style::default().fg(theme.dim))));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {: <28}: ", attr.display_name), Style::default().fg(theme.dim)),
                    Span::styled(display_val, Style::default().fg(theme.fg)),
                ]));
            }
        }
    }

    if !app.show_unavailable_armoury && unavailable_count > 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  + {unavailable_count} unavailable attributes (press 'u' to expand)"),
            Style::default().fg(theme.dim),
        )));
    }

    f.render_widget(Paragraph::new(lines), attr_inner);

    // -------------------------------------------------------------------------
    // Right Pane: Details & Warnings
    // -------------------------------------------------------------------------
    if !is_narrow {
        let details_block = panel_block(" Attribute Details ", false, theme);
        let details_inner = details_block.inner(chunks[1]);
        f.render_widget(details_block, chunks[1]);

        if let Some(attr) = app.armoury_attributes.get(app.armoury_selected_idx) {
            let (desc, reboot_warn) = attr.details();
            let mut right_lines = vec![
                Line::from(Span::styled(
                    format!("  {}", attr.display_name),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Category   : ", Style::default().fg(theme.dim)),
                    Span::styled(attr.category(), Style::default().fg(theme.fg)),
                ]),
                Line::from(vec![
                    Span::styled("  Sysfs Node : ", Style::default().fg(theme.dim)),
                    Span::styled(&attr.node_name, Style::default().fg(theme.fg)),
                ]),
                Line::from(vec![
                    Span::styled("  Current    : ", Style::default().fg(theme.dim)),
                    Span::styled(attr.display_value(), Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("  Default    : ", Style::default().fg(theme.dim)),
                    Span::styled(format!("{}", attr.default_value), Style::default().fg(theme.dim)),
                ]),
            ];

            if attr.min_value != attr.max_value {
                right_lines.push(Line::from(vec![
                    Span::styled("  Range      : ", Style::default().fg(theme.dim)),
                    Span::styled(format!("{} ..= {}", attr.min_value, attr.max_value), Style::default().fg(theme.dim)),
                ]));
            }

            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled("  Description:", Style::default().fg(theme.dim))));
            for line in desc.lines() {
                right_lines.push(Line::from(Span::styled(format!("  {line}"), Style::default().fg(theme.fg))));
            }

            if let Some(warn) = reboot_warn {
                right_lines.push(Line::from(""));
                right_lines.push(Line::from(Span::styled(
                    "  [ ! ] Hardware Notice",
                    Style::default().fg(theme.warn).add_modifier(Modifier::BOLD),
                )));
                for line in warn.lines() {
                    right_lines.push(Line::from(Span::styled(format!("  {line}"), Style::default().fg(theme.warn))));
                }
            }

            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                "  Press 'h' / 'l' or Enter to adjust value.",
                Style::default().fg(theme.dim),
            )));
            right_lines.push(Line::from(Span::styled(
                "  Press 'r' to restore factory default.",
                Style::default().fg(theme.dim),
            )));

            f.render_widget(Paragraph::new(right_lines), details_inner);
        }
    }
}
