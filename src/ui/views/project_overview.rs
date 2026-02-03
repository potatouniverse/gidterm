//! Project Overview View - Unified dashboard showing all projects at a glance
//!
//! Shows:
//! - Project name, port, agent status
//! - Task pipeline summary (done/running/pending)
//! - Recent events
//!
//! Phase 2: Agent Integration
//! - Shows agent status with emoji indicators:
//!   🤖 running, 💭 thinking, ⏳ waiting, ✅ done, ❌ error

use crate::agents::AgentRuntimeStatus;
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render the project overview (unified dashboard)
pub fn render_project_overview(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(10),     // Project list
            Constraint::Length(8),   // Recent events
            Constraint::Length(3),   // Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_project_list(f, app, chunks[1]);
    render_recent_events(f, app, chunks[2]);
    render_footer(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let summaries = app.get_project_summaries();
    let total_projects = summaries.len();
    
    // Count agents by status (Phase 2)
    let mut agents_running = 0;
    let mut agents_thinking = 0;
    let mut agents_waiting = 0;
    
    for summary in &summaries {
        let status = app.get_agent_status(&summary.name);
        match status {
            AgentRuntimeStatus::Running => agents_running += 1,
            AgentRuntimeStatus::Thinking => agents_thinking += 1,
            AgentRuntimeStatus::WaitingInput => agents_waiting += 1,
            _ => {}
        }
    }
    
    let completed = summaries.iter().filter(|s| s.tasks_done == s.task_count && s.task_count > 0).count();
    let errors = summaries.iter().filter(|s| s.tasks_failed > 0).count();
    
    let search_indicator = if app.is_search_mode() {
        format!(" | Search: {}_", app.get_search_query())
    } else {
        String::new()
    };
    
    // Build status string with agent indicators
    let agent_status = if agents_running + agents_thinking + agents_waiting > 0 {
        format!(
            " | 🤖{} 💭{} ⏳{}",
            agents_running, agents_thinking, agents_waiting
        )
    } else {
        String::new()
    };
    
    let title = format!(
        "🌐 gidterm workspace ({} projects) | ✅{} ❌{}{}{}",
        total_projects, completed, errors, agent_status, search_indicator
    );
    
    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    
    f.render_widget(header, area);
}

fn render_project_list(f: &mut Frame, app: &App, area: Rect) {
    let summaries = app.get_project_summaries();
    let mut items: Vec<ListItem> = Vec::new();
    
    for (idx, summary) in summaries.iter().enumerate() {
        let is_selected = idx == app.selected_project;
        
        // Port display
        let port_str = summary.port
            .map(|p| format!(":{}", p))
            .unwrap_or_else(|| "    ".to_string());
        
        // Get agent runtime status for more detailed emoji (Phase 2)
        let agent_runtime = app.get_agent_status(&summary.name);
        let (status_emoji, status_color, status_text) = match agent_runtime {
            AgentRuntimeStatus::Running => ("🤖", Color::Green, "running"),
            AgentRuntimeStatus::Thinking => ("💭", Color::Yellow, "thinking"),
            AgentRuntimeStatus::WaitingInput => ("⏳", Color::Blue, "waiting"),
            AgentRuntimeStatus::Completed => ("✅", Color::Gray, "done"),
            AgentRuntimeStatus::Error => ("❌", Color::Red, "error"),
            AgentRuntimeStatus::NotRunning => {
                // Fall back to task-based display
                let emoji = summary.agent_status.emoji();
                let color = summary.agent_status.color();
                (emoji, color, "idle")
            }
        };
        
        // Task pipeline: [done] → [running] → [pending]
        let pipeline = format!(
            "✓{} ⚙{} □{}",
            summary.tasks_done,
            summary.tasks_running,
            summary.task_count.saturating_sub(summary.tasks_done + summary.tasks_running + summary.tasks_failed)
        );
        
        // Progress percentage
        let progress_pct = if summary.task_count > 0 {
            (summary.tasks_done as f32 / summary.task_count as f32 * 100.0) as u8
        } else {
            0
        };
        
        // Build the line
        let line = Line::from(vec![
            // Selection indicator and project number
            Span::styled(
                format!(" {} ", if is_selected { "▶" } else { " " }),
                Style::default().fg(if is_selected { Color::Yellow } else { Color::DarkGray }),
            ),
            Span::styled(
                format!("[{}] ", idx + 1),
                Style::default().fg(Color::DarkGray),
            ),
            // Project icon and name
            Span::raw("📁 "),
            Span::styled(
                format!("{:<16}", summary.name),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
            ),
            // Port
            Span::styled(
                format!("{:<6}", port_str),
                Style::default().fg(Color::Green),
            ),
            // Agent Status (Phase 2: detailed status)
            Span::styled(
                format!("{} ", status_emoji),
                Style::default().fg(status_color),
            ),
            Span::styled(
                format!("{:<8}", status_text),
                Style::default().fg(status_color),
            ),
            // Pipeline
            Span::styled(
                format!("{:<12}", pipeline),
                Style::default().fg(Color::Gray),
            ),
            // Progress
            Span::styled(
                format!(" {:>3}%", progress_pct),
                Style::default().fg(if progress_pct == 100 { Color::Green } else { Color::Yellow }),
            ),
        ]);
        
        // Recent event (second line)
        let event_line = if let Some(event) = &summary.recent_event {
            Line::from(vec![
                Span::raw("      └─ "),
                Span::styled(
                    truncate(event, 60),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        } else {
            Line::from("")
        };
        
        let style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };
        
        items.push(ListItem::new(vec![line, event_line]).style(style));
    }
    
    let block_title = format!(
        "Projects (1-{} quick switch, / search, Enter focus)",
        summaries.len().min(9)
    );
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(block_title));
    
    f.render_widget(list, area);
}

fn render_recent_events(f: &mut Frame, app: &App, area: Rect) {
    let events = app.get_recent_events(5);
    
    let text = if events.is_empty() {
        "No recent events".to_string()
    } else {
        events
            .iter()
            .map(|(project, msg)| format!("[{}] {}", project, msg))
            .collect::<Vec<_>>()
            .join("\n")
    };
    
    let events_widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Recent Events"))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Gray));
    
    f.render_widget(events_widget, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let help = if app.is_search_mode() {
        "Type to search │ Enter: Jump │ Esc: Cancel".to_string()
    } else {
        // Include agent status legend (Phase 2)
        "1-9: Switch │ /: Search │ Enter: Focus │ Tab: Views │ q: Quit │ 🤖running 💭thinking ⏳waiting ✅done ❌error".to_string()
    };
    
    let footer = Paragraph::new(help)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));
    
    f.render_widget(footer, area);
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
