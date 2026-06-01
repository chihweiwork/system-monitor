mod core;
mod collectors;
mod gpu;
mod ui;
mod storage;

use core::Result;
use collectors::{
    Collector,
    cpu::CpuCollector,
    memory::MemoryCollector,
    process::{ProcessCollector, ProcessStats},
    network::NetworkCollector,
    io::IoCollector,
    disk::DiskCollector,
};
use gpu::GpuCollector;
use ui::{
    Ui, CpuWidget, MemoryWidget, ProcessWidget,
    NetworkWidget, DiskIoWidget, DiskWidget, GpuWidget,
    AppState, ViewMode, SortField, SortOrder,
    MultiPanelLayout,
};
use std::time::Duration;
use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind, MouseButton};
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run_app().await {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

async fn run_app() -> Result<()> {
    // Initialize UI
    let mut ui = Ui::new()?;

    // Initialize collectors
    let mut cpu_collector = CpuCollector::new();
    let mut mem_collector = MemoryCollector::new();
    let mut process_collector = ProcessCollector::new();
    let mut network_collector = NetworkCollector::new();
    let mut io_collector = IoCollector::new();
    let mut disk_collector = DiskCollector::new();
    let mut gpu_collector = GpuCollector::new().await;

    // Initialize widgets
    let mut cpu_widget = CpuWidget::new();
    let mut mem_widget = MemoryWidget::new();
    let mut process_widget = ProcessWidget::new();
    let mut network_widget = NetworkWidget::new();
    let mut disk_io_widget = DiskIoWidget::new();
    let mut disk_widget = DiskWidget::new();
    let mut gpu_widget = GpuWidget::new();

    // Application state
    let mut app_state = AppState::new();

    // Initial data collection
    let mut cpu_stats = cpu_collector.collect().await?;
    let mut mem_stats = mem_collector.collect().await?;
    let mut processes = process_collector.collect().await?;
    let mut network_stats = network_collector.collect().await?;
    let mut io_stats = io_collector.collect().await?;
    let mut disk_stats = disk_collector.collect().await?;
    let mut gpu_stats = gpu_collector.collect().await.unwrap_or_default();

    // Update interval
    let update_interval = Duration::from_millis(1000);
    let mut last_update = tokio::time::Instant::now();

    // Store panel_rects for mouse event handling
    let mut panel_rects: Vec<ui::PanelRect> = Vec::new();
    let mut pending_mouse_event: Option<MouseEvent> = None;

    loop {
        // Handle input events
        if let Some(event) = ui.poll_event(Duration::from_millis(50))? {
            match event {
                Event::Key(key_event) => {
                    // Priority: handle detail popup events first
                    if app_state.is_detail_popup_open() {
                        handle_detail_popup_input(key_event.code, &mut app_state);
                        continue;
                    }

                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') if !app_state.filter_active => break,
                        KeyCode::Esc => {
                            if app_state.help_visible {
                                app_state.toggle_help();
                            } else if app_state.modal_active {
                                app_state.toggle_modal();
                            } else if app_state.filter_active {
                                app_state.toggle_filter();
                            } else {
                                break;
                            }
                        }

                        // Panel toggling
                        KeyCode::Tab if !app_state.filter_active => app_state.next_view(),
                        KeyCode::BackTab if !app_state.filter_active => app_state.prev_view(),
                        KeyCode::Char('1') if !app_state.filter_active => { app_state.toggle_panel(ViewMode::Cpu); },
                        KeyCode::Char('2') if !app_state.filter_active => { app_state.toggle_panel(ViewMode::Memory); },
                        KeyCode::Char('3') if !app_state.filter_active => { app_state.toggle_panel(ViewMode::Processes); },
                        KeyCode::Char('4') if !app_state.filter_active => { app_state.toggle_panel(ViewMode::Network); },
                        KeyCode::Char('5') if !app_state.filter_active => { app_state.toggle_panel(ViewMode::DiskIo); },
                        KeyCode::Char('6') if !app_state.filter_active => { app_state.toggle_panel(ViewMode::DiskUsage); },
                        KeyCode::Char('7') if !app_state.filter_active => { app_state.toggle_panel(ViewMode::Gpu); },

                        // Detail mode toggle
                        KeyCode::Char('d') if !app_state.filter_active && !app_state.modal_active && !app_state.is_detail_popup_open() => {
                            // Most panels use popup window
                            match app_state.active_panel {
                                ViewMode::Cpu => {
                                    app_state.open_detail_popup(ui::DetailPopupType::Cpu);
                                }
                                ViewMode::Memory => {
                                    app_state.open_detail_popup(ui::DetailPopupType::Memory);
                                }
                                ViewMode::DiskIo => {
                                    app_state.open_detail_popup(ui::DetailPopupType::DiskIo);
                                }
                                ViewMode::Network => {
                                    app_state.open_detail_popup(ui::DetailPopupType::Network);
                                }
                                ViewMode::DiskUsage => {
                                    app_state.open_detail_popup(ui::DetailPopupType::DiskUsage);
                                }
                                ViewMode::Gpu => {
                                    app_state.open_detail_popup(ui::DetailPopupType::Gpu);
                                }
                                _ => {
                                    // Fallback to inline detail mode
                                    app_state.toggle_detail_mode(app_state.active_panel);
                                }
                            }
                        }

                        // Help
                        KeyCode::Char('?') | KeyCode::Char('h') if !app_state.filter_active => {
                            app_state.toggle_help();
                        }

                        // CPU detail view controls
                        _ if app_state.active_panel == ViewMode::Cpu
                            && app_state.is_detail_mode(ViewMode::Cpu)
                            && !app_state.help_visible => {
                            handle_cpu_detail_input(key_event.code, &mut cpu_widget, &cpu_stats);
                        }

                        // Process view controls
                        _ if app_state.active_panel == ViewMode::Processes && !app_state.help_visible => {
                            handle_process_view_input(
                                key_event.code,
                                &mut app_state,
                                &mut process_widget,
                                &processes,
                            );
                        }

                        // Memory detail view scrolling
                        _ if app_state.active_panel == ViewMode::Memory
                            && app_state.is_detail_mode(ViewMode::Memory)
                            && !app_state.help_visible
                            && !app_state.filter_active
                            && !app_state.modal_active => {
                            match key_event.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    mem_widget.scroll_down(1, processes.len());
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    mem_widget.scroll_up(1);
                                }
                                KeyCode::PageDown => {
                                    mem_widget.page_down(10, processes.len());
                                }
                                KeyCode::PageUp => {
                                    mem_widget.page_up(10);
                                }
                                KeyCode::Home | KeyCode::Char('g') => {
                                    mem_widget.home();
                                }
                                KeyCode::End | KeyCode::Char('G') => {
                                    mem_widget.end(processes.len());
                                }
                                _ => {}
                            }
                        }

                        // Network scrolling (always available when panel is active)
                        _ if app_state.active_panel == ViewMode::Network
                            && !app_state.help_visible
                            && !app_state.filter_active
                            && !app_state.modal_active => {
                            match key_event.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    network_widget.scroll_down(1, network_stats.len());
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    network_widget.scroll_up(1);
                                }
                                _ => {}
                            }
                        }

                        // DiskIO scrolling
                        _ if app_state.active_panel == ViewMode::DiskIo
                            && !app_state.help_visible
                            && !app_state.filter_active
                            && !app_state.modal_active => {
                            match key_event.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    disk_io_widget.scroll_down(1, io_stats.len());
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    disk_io_widget.scroll_up(1);
                                }
                                _ => {}
                            }
                        }

                        // DiskUsage scrolling
                        _ if app_state.active_panel == ViewMode::DiskUsage
                            && !app_state.help_visible
                            && !app_state.filter_active
                            && !app_state.modal_active => {
                            match key_event.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    disk_widget.scroll_down(disk_stats.len(), 10);
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    disk_widget.scroll_up();
                                }
                                _ => {}
                            }
                        }

                        // GPU scrolling
                        _ if app_state.active_panel == ViewMode::Gpu
                            && !app_state.help_visible
                            && !app_state.filter_active
                            && !app_state.modal_active => {
                            match key_event.code {
                                KeyCode::Char('j') | KeyCode::Down => {
                                    gpu_widget.scroll_down(gpu_stats.len());
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    gpu_widget.scroll_up();
                                }
                                _ => {}
                            }
                        }

                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => {
                    // Store mouse event to process after displayed_processes is ready
                    pending_mouse_event = Some(mouse_event);
                }
                _ => {}
            }
        }

        // Update data if interval has passed
        let now = tokio::time::Instant::now();
        if now.duration_since(last_update) >= update_interval {
            cpu_stats = cpu_collector.collect().await?;
            mem_stats = mem_collector.collect().await?;
            processes = process_collector.collect().await?;
            network_stats = network_collector.collect().await?;
            io_stats = io_collector.collect().await?;
            disk_stats = disk_collector.collect().await?;
            gpu_stats = gpu_collector.collect().await.unwrap_or_default();
            cpu_widget.update(&cpu_stats);
            last_update = now;
        }

        // Sort and filter processes
        let mut displayed_processes = if app_state.filter_text.is_empty() {
            processes.clone()
        } else {
            processes
                .iter()
                .filter(|p| {
                    p.name
                        .to_lowercase()
                        .contains(&app_state.filter_text.to_lowercase())
                })
                .cloned()
                .collect()
        };

        sort_processes(&mut displayed_processes, app_state.sort_field, app_state.sort_order);

        // Handle pending mouse event after displayed_processes is ready
        if let Some(mouse_event) = pending_mouse_event.take() {
            handle_mouse_event(mouse_event, &panel_rects, &mut app_state, &mut process_widget, &displayed_processes);
        }

        // Render UI
        let cpu_stats_clone = cpu_stats.clone();
        let mem_stats_clone = mem_stats.clone();
        let network_stats_clone = network_stats.clone();
        let io_stats_clone = io_stats.clone();
        let disk_stats_clone = disk_stats.clone();
        let gpu_stats_clone = gpu_stats.clone();
        let theme = ui.theme().clone();
        let state = app_state.clone();
        let processes_clone = displayed_processes.clone();

        // Calculate panel_rects before rendering for mouse event handling
        let (width, height) = ui.size()?;
        let size = Rect { x: 0, y: 0, width, height };

        if !state.help_visible {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(if state.filter_active { 1 } else { 0 }),
                ])
                .split(size);

            let layout = MultiPanelLayout::new();
            if layout.validate_minimum_size(chunks[1]).is_ok() {
                panel_rects = layout.calculate(chunks[1], &state.visible_panels);
            } else {
                panel_rects.clear();
            }
        } else {
            panel_rects.clear();
        }

        ui.render(|frame| {
            let size = frame.area();

            if state.help_visible {
                render_help_screen(frame, size);
                return;
            }

            // Title bar
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(if state.filter_active { 1 } else { 0 }),
                ])
                .split(size);

            render_title_bar(frame, chunks[0], &state);

            // Multi-panel layout
            let layout = MultiPanelLayout::new();

            if let Err(msg) = layout.validate_minimum_size(chunks[1]) {
                let error = Paragraph::new(msg).style(Style::default().fg(Color::Red));
                frame.render_widget(error, chunks[1]);
                return;
            }

            let panel_rects_local = layout.calculate(chunks[1], &state.visible_panels);

            // Render each visible panel
            for panel_rect in panel_rects_local {
                match panel_rect.panel {
                    ViewMode::Cpu => {
                        cpu_widget.render(frame, panel_rect.rect, &cpu_stats_clone, &theme, state.is_detail_mode(ViewMode::Cpu));
                    }
                    ViewMode::Memory => {
                        mem_widget.render(frame, panel_rect.rect, &mem_stats_clone, &theme, state.is_detail_mode(ViewMode::Memory), &processes_clone);
                    }
                    ViewMode::Processes => {
                        process_widget.render(frame, panel_rect.rect, &processes_clone, &theme, state.show_full_command);
                    }
                    ViewMode::Network => {
                        network_widget.render(frame, panel_rect.rect, &network_stats_clone, &theme, state.is_detail_mode(ViewMode::Network), &processes_clone);
                    }
                    ViewMode::DiskIo => {
                        disk_io_widget.render(frame, panel_rect.rect, &io_stats_clone, &theme, state.is_detail_mode(ViewMode::DiskIo), &processes_clone);
                    }
                    ViewMode::DiskUsage => {
                        disk_widget.render(frame, panel_rect.rect, &disk_stats_clone, &theme, state.is_detail_mode(ViewMode::DiskUsage));
                    }
                    ViewMode::Gpu => {
                        gpu_widget.render(frame, panel_rect.rect, &gpu_stats_clone, &theme, state.is_detail_mode(ViewMode::Gpu));
                    }
                }
            }

            // Filter bar
            if state.filter_active {
                render_filter_bar(frame, chunks[2], &state.filter_text);
            }

            // Modal
            if state.modal_active && state.active_panel == ViewMode::Processes {
                if let Some(idx) = process_widget.selected_index().checked_sub(0) {
                    if idx < processes_clone.len() {
                        render_process_modal(frame, size, &processes_clone[idx], &theme);
                    }
                }
            }

            // Detail popup
            if state.is_detail_popup_open() {
                if let Some(popup_state) = &state.detail_popup {
                    ui::render_detail_popup(
                        frame,
                        size,
                        popup_state,
                        state.detail_popup_type,
                        &processes_clone,
                        Some(&cpu_stats_clone),
                        Some(&disk_stats_clone),
                        Some(&gpu_stats_clone),
                        &theme,
                    );
                }
            }
        })?;

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}

fn handle_cpu_detail_input(
    key: KeyCode,
    widget: &mut CpuWidget,
    stats: &collectors::cpu::CpuStats,
) {
    match key {
        KeyCode::Char('j') | KeyCode::Down => {
            widget.scroll_down(1, stats.cores.len());
        }
        KeyCode::Char('k') | KeyCode::Up => {
            widget.scroll_up(1);
        }
        KeyCode::PageDown => {
            widget.scroll_down(10, stats.cores.len());
        }
        KeyCode::PageUp => {
            widget.scroll_up(10);
        }
        _ => {}
    }
}

fn handle_detail_popup_input(
    key: KeyCode,
    app_state: &mut AppState,
) {
    let popup_type = app_state.detail_popup_type;
    if let Some(popup_state) = &mut app_state.detail_popup {
        if popup_state.search_mode {
            // Search mode key handling
            match key {
                KeyCode::Esc => {
                    popup_state.toggle_search();
                }
                KeyCode::Enter => {
                    popup_state.search_mode = false;
                    // Keep search text, continue filtering
                }
                KeyCode::Backspace => {
                    popup_state.search_text.pop();
                }
                KeyCode::Char(c) => {
                    popup_state.search_text.push(c);
                }
                _ => {}
            }
        } else {
            // Normal mode key handling
            match key {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app_state.close_detail_popup();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    popup_state.scroll_down(1, 10000);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    popup_state.scroll_up(1);
                }
                KeyCode::PageDown => {
                    popup_state.scroll_down(10, 10000);
                }
                KeyCode::PageUp => {
                    popup_state.scroll_up(10);
                }
                KeyCode::Home => {
                    popup_state.scroll_offset = 0;
                }
                KeyCode::End => {
                    popup_state.scroll_offset = 10000;
                }
                KeyCode::Char('/') => {
                    popup_state.toggle_search();
                }
                KeyCode::Char('s') => {
                    // Cycle through sort fields
                    popup_state.sort_field = popup_state.sort_field.next(popup_type);
                    popup_state.scroll_offset = 0;
                }
                KeyCode::Char('r') => {
                    // Reverse sort order
                    popup_state.sort_order = popup_state.sort_order.toggle();
                    popup_state.scroll_offset = 0;
                }
                _ => {}
            }
        }
    }
}

fn handle_process_view_input(
    key: KeyCode,
    state: &mut AppState,
    widget: &mut ProcessWidget,
    processes: &[ProcessStats],
) {
    if state.help_visible {
        return;
    }

    if state.filter_active {
        match key {
            KeyCode::Char(c) => state.add_filter_char(c),
            KeyCode::Backspace => state.remove_filter_char(),
            KeyCode::Enter | KeyCode::Esc => state.toggle_filter(),
            _ => {}
        }
        return;
    }

    if state.modal_active {
        if matches!(key, KeyCode::Enter | KeyCode::Esc) {
            state.toggle_modal();
        }
        return;
    }

    // Command line toggle
    if matches!(key, KeyCode::Char('c') | KeyCode::Char('C')) {
        state.toggle_full_command();
        return;
    }

    let visible_height = 20; // Approximate, will be adjusted dynamically

    match key {
        // Navigation - vim style
        KeyCode::Char('j') | KeyCode::Down => {
            widget.scroll_down(1, processes.len());
            widget.adjust_scroll(visible_height);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            widget.scroll_up(1);
            widget.adjust_scroll(visible_height);
        }
        KeyCode::Char('g') => {
            widget.home();
            widget.adjust_scroll(visible_height);
        }
        KeyCode::Char('G') => {
            widget.end(processes.len());
            widget.adjust_scroll(visible_height);
        }
        KeyCode::PageDown => {
            widget.page_down(visible_height, processes.len());
            widget.adjust_scroll(visible_height);
        }
        KeyCode::PageUp => {
            widget.page_up(visible_height);
            widget.adjust_scroll(visible_height);
        }
        KeyCode::Home => {
            widget.home();
            widget.adjust_scroll(visible_height);
        }
        KeyCode::End => {
            widget.end(processes.len());
            widget.adjust_scroll(visible_height);
        }

        // Filter
        KeyCode::Char('/') | KeyCode::Char('f') => {
            state.toggle_filter();
        }

        // Sort
        KeyCode::Left => state.prev_sort_field(),
        KeyCode::Right => state.next_sort_field(),
        KeyCode::Char('s') | KeyCode::Char(' ') => state.toggle_sort_order(),

        // Modal
        KeyCode::Enter => {
            state.selected_process_index = Some(widget.selected_index());
            state.toggle_modal();
        }

        _ => {}
    }
}

fn sort_processes(processes: &mut [ProcessStats], field: SortField, order: SortOrder) {
    processes.sort_by(|a, b| {
        let cmp = match field {
            SortField::Pid => a.pid.cmp(&b.pid),
            SortField::Name => a.name.cmp(&b.name),
            SortField::Cpu => a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal),
            SortField::Memory => a.memory_percent.partial_cmp(&b.memory_percent).unwrap_or(std::cmp::Ordering::Equal),
        };

        match order {
            SortOrder::Ascending => cmp,
            SortOrder::Descending => cmp.reverse(),
        }
    });
}

fn render_title_bar(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let tabs = [
        (ViewMode::Cpu, "1:CPU"),
        (ViewMode::Memory, "2:Mem"),
        (ViewMode::Processes, "3:Proc"),
        (ViewMode::Network, "4:Net"),
        (ViewMode::DiskIo, "5:I/O"),
        (ViewMode::DiskUsage, "6:Disk"),
        (ViewMode::Gpu, "7:GPU"),
    ];

    let mut spans = Vec::new();
    for (panel, label) in tabs.iter() {
        let is_visible = state.is_panel_visible(*panel);
        let is_active = state.active_panel == *panel;

        let style = if is_active {
            // Active: bold cyan on black
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_visible {
            // Visible: cyan text
            Style::default().fg(Color::Cyan)
        } else {
            // Hidden: dark gray
            Style::default().fg(Color::DarkGray)
        };

        spans.push(Span::styled(format!(" {} ", label), style));
        spans.push(Span::raw(" "));
    }

    // Sort indicator
    if state.active_panel == ViewMode::Processes {
        spans.push(Span::styled(
            format!("│ Sort: {} ", state.sort_field.name()),
            Style::default().fg(Color::Yellow),
        ));
        spans.push(Span::styled(
            match state.sort_order {
                SortOrder::Ascending => "▲",
                SortOrder::Descending => "▼",
            },
            Style::default().fg(Color::Yellow),
        ));
    }

    // Help hint
    spans.push(Span::styled(
        " │ 1-7: Toggle | Tab: Switch | ?: Help | q: Quit",
        Style::default().fg(Color::DarkGray),
    ));

    let title = Paragraph::new(Line::from(spans));
    frame.render_widget(title, area);
}

fn render_filter_bar(frame: &mut ratatui::Frame, area: Rect, filter_text: &str) {
    let text = format!("Filter: {} _", filter_text);
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}

fn render_process_modal(
    frame: &mut ratatui::Frame,
    area: Rect,
    process: &ProcessStats,
    theme: &ui::Theme,
) {
    // Center modal
    let modal_width = area.width.min(80);
    let modal_height = area.height.min(25);
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: modal_width,
        height: modal_height,
    };

    let block = Block::default()
        .title(format!(" Process Details: {} ", process.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let lines = vec![
        Line::from(vec![
            Span::styled("PID:          ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", process.pid), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Parent PID:   ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", process.ppid), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Status:       ", Style::default().fg(Color::Gray)),
            Span::styled(&process.status, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Threads:      ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", process.threads), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("CPU Usage:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.2}%", process.cpu_percent),
                Style::default().fg(theme.cpu_color(process.cpu_percent)),
            ),
        ]),
        Line::from(vec![
            Span::styled("Memory:       ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.2} MB ({:.2}%)", process.memory_kb as f64 / 1024.0, process.memory_percent),
                Style::default().fg(theme.mem_color(process.memory_percent)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Working Dir:  ", Style::default().fg(Color::Gray)),
            Span::styled(&process.cwd, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Open FDs:     ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{} files", process.fd_count), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Command:      ", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled(&process.cmdline, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Enter or ESC to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn handle_mouse_event(
    mouse_event: MouseEvent,
    panel_rects: &[ui::PanelRect],
    state: &mut AppState,
    process_widget: &mut ProcessWidget,
    processes: &[ProcessStats],
) {
    // Only handle left clicks
    if !matches!(mouse_event.kind, MouseEventKind::Down(MouseButton::Left)) {
        return;
    }

    let click_x = mouse_event.column;
    let click_y = mouse_event.row;

    // Find which panel was clicked
    for panel_rect in panel_rects {
        if panel_rect.panel == ViewMode::Processes {
            let rect = panel_rect.rect;

            // Check if click is within panel bounds
            if click_x >= rect.x && click_x < rect.x + rect.width
                && click_y >= rect.y && click_y < rect.y + rect.height {

                // Account for border (1) + title (1) + header (1) = 3 rows
                let content_start_y = rect.y + 3;

                if click_y >= content_start_y {
                    let clicked_row = (click_y - content_start_y) as usize;
                    let process_index = process_widget.scroll_offset() + clicked_row;

                    if process_index < processes.len() {
                        process_widget.set_selected_index(process_index);
                        state.active_panel = ViewMode::Processes;
                    }
                }

                break;
            }
        }
    }
}

fn render_help_screen(frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title(" Help - Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("General:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  q, ESC         - Quit application"),
        Line::from("  ?, h           - Show this help screen"),
        Line::from("  Tab            - Switch to next visible panel"),
        Line::from("  Shift+Tab      - Switch to previous visible panel"),
        Line::from("  1-7            - Toggle panels: CPU(1), Memory(2), Processes(3),"),
        Line::from("                   Network(4), Disk I/O(5), Disk Usage(6), GPU(7)"),
        Line::from("                   (At least one panel must remain visible)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Process View Navigation:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  j, ↓           - Move down"),
        Line::from("  k, ↑           - Move up"),
        Line::from("  g, Home        - Go to first process"),
        Line::from("  G, End         - Go to last process"),
        Line::from("  PageUp/Down    - Scroll page up/down"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Process View Actions:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Enter          - Show process details"),
        Line::from("  /, f           - Filter by name"),
        Line::from("  c              - Toggle command line display"),
        Line::from("  ←, →           - Change sort field"),
        Line::from("  s, Space       - Toggle sort order (ascending/descending)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press any key to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}
