//! Live dashboard with real-time updates

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render the live dashboard
pub fn render_live_dashboard(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(10),         // Task list
            Constraint::Length(10),      // Selected task output
            Constraint::Length(3),       // Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_task_list(f, app, chunks[1]);
    render_task_output(f, app, chunks[2]);
    render_footer(f, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let graph = app.scheduler.graph();
    
    let title = if app.workspace_mode {
        format!("🌐 Workspace ({} projects) - GidTerm", app.project_names.len())
    } else if let Some(metadata) = &graph.metadata {
        format!("📊 {} - GidTerm (Live)", metadata.project)
    } else {
        "📊 GidTerm (Live)".to_string()
    };

    // Count task statuses
    let total = graph.all_tasks().len();
    let running = app.scheduler.get_running().len();
    let done = graph.all_tasks().values()
        .filter(|t| t.status == "done")
        .count();
    let failed = graph.all_tasks().values()
        .filter(|t| t.status == "failed")
        .count();

    let status_text = format!(
        "{} | Running: {} | Done: {} | Failed: {} | Total: {}",
        title, running, done, failed, total
    );

    let header = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(header, area);
}

fn render_task_list(f: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_idx = 0;

    if app.workspace_mode {
        // Group tasks by project
        let tasks_by_project = app.get_tasks_by_project();
        
        for project_name in &app.project_names {
            // Project header
            let project_header = Line::from(vec![
                Span::styled(
                    format!("📁 {}", project_name),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                ),
            ]);
            items.push(ListItem::new(project_header));
            current_idx += 1;

            // Tasks for this project
            if let Some(task_ids) = tasks_by_project.get(project_name) {
                for task_id in task_ids {
                    let item = render_task_item(app, task_id, current_idx);
                    items.push(item);
                    current_idx += 1;
                }
            }

            // Empty line between projects
            items.push(ListItem::new(Line::from("")));
            current_idx += 1;
        }
    } else {
        // Single project mode - flat list
        let task_ids = app.get_task_ids();
        for (idx, task_id) in task_ids.iter().enumerate() {
            let item = render_task_item(app, task_id, idx);
            items.push(item);
        }
    }

    let task_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Tasks (↑↓ to select)")
    );

    f.render_widget(task_list, area);
}

fn render_task_item<'a>(app: &'a App, task_id: &str, idx: usize) -> ListItem<'a> {
    let task = app.scheduler.graph().get_task(task_id).unwrap();
    
    let status_icon = match task.status.as_str() {
        "done" => "✓",
        "in-progress" => "⚙",
        "failed" => "✗",
        _ => "□",
    };

    let status_color = match task.status.as_str() {
        "done" => Color::Green,
        "in-progress" => Color::Yellow,
        "failed" => Color::Red,
        _ => Color::Gray,
    };

    let priority_badge = task.priority.as_ref()
        .map(|p| match p.as_str() {
            "critical" => "🔴",
            "high" => "🟡",
            "medium" => "🔵",
            _ => "⚪",
        })
        .unwrap_or("");

    // Show output line count if any
    let output_count = app.task_outputs.get(task_id)
        .map(|lines| format!(" ({}L)", lines.len()))
        .unwrap_or_default();

    // In workspace mode, show only the task name (without project prefix)
    let display_name = if app.workspace_mode {
        task_id.split(':').nth(1).unwrap_or(task_id)
    } else {
        task_id
    };

    // Highlight selected task
    let style = if idx == app.selected_task {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    let line = Line::from(vec![
        Span::raw("  "),  // Indent for project grouping
        Span::raw(format!("{} ", status_icon)),
        Span::styled(
            display_name.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        ),
        Span::raw(format!(" {}", priority_badge)),
        Span::styled(
            format!(" [{}]", task.status),
            Style::default().fg(status_color),
        ),
        Span::styled(output_count, Style::default().fg(Color::Cyan)),
    ]);

    ListItem::new(line).style(style)
}

fn render_task_output(f: &mut Frame, app: &App, area: Rect) {
    let task_ids = app.get_task_ids();
    
    if task_ids.is_empty() || app.selected_task >= task_ids.len() {
        let empty = Paragraph::new("No task selected")
            .block(Block::default().borders(Borders::ALL).title("Output"));
        f.render_widget(empty, area);
        return;
    }

    let task_id = &task_ids[app.selected_task];
    let output_lines = app.get_task_output(task_id, 8);

    let text = if output_lines.is_empty() {
        "(no output yet)".to_string()
    } else {
        output_lines.join("\n")
    };

    let output = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Output: {}", task_id))
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));

    f.render_widget(output, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let help_text = "q: Quit | r: Refresh | ↑↓: Select task";
    
    let footer = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(footer, area);
}
