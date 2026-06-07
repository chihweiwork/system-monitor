// Reusable UI widgets for displaying system metrics

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::collectors::{cpu::CpuStats, memory::MemoryStats, io::IoStats};
use super::theme::Theme;

pub struct CpuWidget {
    history: Vec<f64>,
    max_history: usize,
    scroll_offset: usize,
}

impl CpuWidget {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            max_history: 60, // Keep 60 samples
            scroll_offset: 0,
        }
    }

    pub fn update(&mut self, stats: &CpuStats) {
        self.history.push(stats.usage_percent);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, stats: &CpuStats, theme: &Theme, is_detail: bool) {
        let title = if is_detail {
            format!(" CPU ({} cores) [Detail] ", stats.core_count)
        } else {
            format!(" CPU ({} cores) ", stats.core_count)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(theme.cpu_box);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Overall CPU usage gauge
        let gauge_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        };

        let color = theme.cpu_color(stats.usage_percent);
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .percent(stats.usage_percent.min(100.0) as u16)
            .label(format!("Total: {:.1}%", stats.usage_percent));

        frame.render_widget(gauge, gauge_area);

        // Render detail or grid view
        if inner.height > 3 && !stats.cores.is_empty() {
            if is_detail {
                self.render_detail_view(frame, inner, stats, theme);
            } else {
                self.render_core_grid(frame, inner, stats, theme);
            }
        }
    }

    fn render_core_grid(
        &self,
        frame: &mut Frame,
        area: Rect,
        stats: &CpuStats,
        theme: &Theme,
    ) {
        let available_height = area.height.saturating_sub(2) as usize;
        let available_width = area.width as usize;

        // Determine grid columns (2-3 based on terminal width)
        let columns = if available_width >= 90 {
            3
        } else if available_width >= 60 {
            2
        } else {
            1
        };

        let rows = (stats.core_count + columns - 1) / columns;

        // Check space availability
        if rows > available_height {
            let msg = Paragraph::new(format!(
                "{} cores (terminal too small for grid)",
                stats.core_count
            ))
            .style(Style::default().fg(Color::Yellow));

            let text_area = Rect {
                x: area.x,
                y: area.y + 2,
                width: area.width,
                height: area.height.saturating_sub(2),
            };
            frame.render_widget(msg, text_area);
            return;
        }

        // Render grid
        let core_width = available_width / columns;
        let mut lines = Vec::new();

        for row in 0..rows {
            let mut row_spans = Vec::new();

            for col in 0..columns {
                let idx = row * columns + col;
                if idx >= stats.cores.len() {
                    break;
                }

                let core = &stats.cores[idx];
                let color = theme.cpu_color(core.usage_percent);

                let text = format!("cpu{:>2}: {:>5.1}%", core.core_id, core.usage_percent);
                let padded = format!("{:<width$}", text, width = core_width);

                row_spans.push(Span::styled(padded, Style::default().fg(color)));
            }

            lines.push(Line::from(row_spans));
        }

        let text_area = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width,
            height: area.height.saturating_sub(2),
        };

        frame.render_widget(Paragraph::new(lines), text_area);
    }

    fn render_detail_view(
        &self,
        frame: &mut Frame,
        area: Rect,
        stats: &CpuStats,
        theme: &Theme,
    ) {
        let available_height = area.height.saturating_sub(2) as usize;

        if available_height == 0 {
            return;
        }

        // Calculate visible range
        let visible_cores = stats.cores.len().min(available_height);
        let max_scroll = stats.cores.len().saturating_sub(available_height);
        let scroll_offset = self.scroll_offset.min(max_scroll);
        let end_index = (scroll_offset + visible_cores).min(stats.cores.len());
        let visible = &stats.cores[scroll_offset..end_index];

        let mut lines = Vec::new();

        // Header line
        let header = Line::from(vec![
            Span::styled("Core ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Total ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("User ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Sys  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("I/O  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("IRQ  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Soft ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Steal", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]);
        lines.push(header);

        // Per-core breakdown
        for core in visible {
            let total_time = core.user_time + core.system_time + core.idle_time +
                           core.iowait_time + core.irq_time + core.softirq_time + core.steal_time;

            let (user_pct, sys_pct, io_pct, irq_pct, soft_pct, steal_pct) = if total_time > 0 {
                (
                    (core.user_time as f64 / total_time as f64) * 100.0,
                    (core.system_time as f64 / total_time as f64) * 100.0,
                    (core.iowait_time as f64 / total_time as f64) * 100.0,
                    (core.irq_time as f64 / total_time as f64) * 100.0,
                    (core.softirq_time as f64 / total_time as f64) * 100.0,
                    (core.steal_time as f64 / total_time as f64) * 100.0,
                )
            } else {
                (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
            };

            let usage_color = theme.cpu_color(core.usage_percent);

            let line = Line::from(vec![
                Span::styled(format!("{:>4} ", core.core_id), Style::default().fg(Color::Cyan)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>5.1}%", core.usage_percent), Style::default().fg(usage_color)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>4.1}%", user_pct), Style::default().fg(Color::Green)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>4.1}%", sys_pct), Style::default().fg(Color::Red)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>4.1}%", io_pct), Style::default().fg(Color::Magenta)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>4.1}%", irq_pct), Style::default().fg(Color::Yellow)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>4.1}%", soft_pct), Style::default().fg(Color::Cyan)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>4.1}%", steal_pct), Style::default().fg(Color::Blue)),
            ]);
            lines.push(line);
        }

        let text_area = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width,
            height: area.height.saturating_sub(2),
        };

        frame.render_widget(Paragraph::new(lines), text_area);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize, max: usize) {
        let max_scroll = max.saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
    }
}

pub struct MemoryWidget {
    scroll_offset: usize,
}

impl MemoryWidget {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, stats: &MemoryStats, theme: &Theme, is_detail: bool, processes: &[ProcessStats]) {
        let title = if is_detail {
            " Memory [Detail] "
        } else {
            " Memory "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(theme.mem_box);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Memory usage gauge
        let usage_percent = if stats.total > 0 {
            (stats.used as f64 / stats.total as f64) * 100.0
        } else {
            0.0
        };

        let gauge_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        };

        let color = theme.mem_color(usage_percent);
        let used_gb = stats.used as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_gb = stats.total as f64 / 1024.0 / 1024.0 / 1024.0;

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .percent(usage_percent.min(100.0) as u16)
            .label(format!("{:.1}% ({:.2}/{:.2} GB)", usage_percent, used_gb, total_gb));

        frame.render_widget(gauge, gauge_area);

        if is_detail {
            self.render_detail_view(frame, inner, processes, theme);
        } else {
            self.render_compact_view(frame, inner, stats, theme);
        }
    }

    fn render_compact_view(&self, frame: &mut Frame, inner: Rect, stats: &MemoryStats, theme: &Theme) {
        // Detailed stats
        if inner.height > 3 {
            let text_area = Rect {
                x: inner.x,
                y: inner.y + 2,
                width: inner.width,
                height: inner.height - 2,
            };

            let avail_gb = stats.available as f64 / 1024.0 / 1024.0 / 1024.0;
            let cached_gb = stats.cached as f64 / 1024.0 / 1024.0 / 1024.0;
            let buffers_gb = stats.buffers as f64 / 1024.0 / 1024.0 / 1024.0;

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Available: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{:.2} GB", avail_gb),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Cached:    ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{:.2} GB", cached_gb),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Buffers:   ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{:.2} GB", buffers_gb),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
            ];

            // Add swap info if swap is configured
            if stats.swap_total > 0 {
                let swap_used_gb = stats.swap_used as f64 / 1024.0 / 1024.0 / 1024.0;
                let swap_total_gb = stats.swap_total as f64 / 1024.0 / 1024.0 / 1024.0;
                let swap_percent = (stats.swap_used as f64 / stats.swap_total as f64) * 100.0;

                lines.push(Line::from(vec![
                    Span::styled("Swap:      ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{:.2}/{:.2} GB ({:.1}%)", swap_used_gb, swap_total_gb, swap_percent),
                        Style::default().fg(Color::Magenta),
                    ),
                ]));
            }

            let paragraph = Paragraph::new(lines);
            frame.render_widget(paragraph, text_area);
        }
    }

    fn render_detail_view(&self, frame: &mut Frame, inner: Rect, processes: &[ProcessStats], theme: &Theme) {
        if inner.height <= 3 {
            return;
        }

        let text_area = Rect {
            x: inner.x,
            y: inner.y + 2,
            width: inner.width,
            height: inner.height - 2,
        };

        // Sort processes by memory usage (descending)
        let mut sorted_processes: Vec<&ProcessStats> = processes.iter().collect();
        sorted_processes.sort_by(|a, b| b.memory_percent.partial_cmp(&a.memory_percent).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate visible range
        let visible_height = text_area.height as usize;
        let end_index = (self.scroll_offset + visible_height).min(sorted_processes.len());
        let visible_processes = &sorted_processes[self.scroll_offset..end_index];

        // Header line
        let mut lines = vec![
            Line::from(vec![
                Span::styled(format!("{:<8}", "PID"), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>8}", "MEM%"), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>12}", "SIZE"), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  {}", "NAME"), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ])
        ];

        // Process lines
        for process in visible_processes {
            let memory_mb = process.memory_kb as f64 / 1024.0;
            let size_str = if memory_mb >= 1024.0 {
                format!("{:.2} GB", memory_mb / 1024.0)
            } else {
                format!("{:.1} MB", memory_mb)
            };

            let mem_color = if process.memory_percent > 10.0 {
                Color::Red
            } else if process.memory_percent > 5.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{:<8}", process.pid), Style::default().fg(Color::Gray)),
                Span::styled(format!("{:>7.1}%", process.memory_percent), Style::default().fg(mem_color)),
                Span::styled(format!("{:>12}", size_str), Style::default().fg(Color::White)),
                Span::styled(format!("  {}", process.name), Style::default().fg(Color::Gray)),
            ]));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, text_area);
    }

    pub fn scroll_up(&mut self, count: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(count);
    }

    pub fn scroll_down(&mut self, count: usize, max: usize) {
        if max > 0 {
            let max_scroll = max.saturating_sub(1);
            self.scroll_offset = (self.scroll_offset + count).min(max_scroll);
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_up(page_size);
    }

    pub fn page_down(&mut self, page_size: usize, max: usize) {
        self.scroll_down(page_size, max);
    }

    pub fn home(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn end(&mut self, max: usize) {
        if max > 0 {
            self.scroll_offset = max.saturating_sub(1);
        }
    }
}

use crate::collectors::process::ProcessStats;

pub struct ProcessWidget {
    scroll_offset: usize,
    selected_index: usize,
}

impl ProcessWidget {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            selected_index: 0,
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        processes: &[ProcessStats],
        theme: &Theme,
        show_full_command: bool,
    ) {
        let block = Block::default()
            .title(" Processes ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if processes.is_empty() {
            let text = Paragraph::new("No processes found");
            frame.render_widget(text, inner);
            return;
        }

        // Header
        let header_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        };

        let header_name = if show_full_command { "Command" } else { "Name" };
        let header = Line::from(vec![
            Span::styled("USER     ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("    PID ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("CPU%  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("MEM%  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled(header_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]);

        frame.render_widget(Paragraph::new(header), header_area);

        // Process list
        let list_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };

        let visible_count = list_area.height as usize;
        let end_index = (self.scroll_offset + visible_count).min(processes.len());
        let visible_processes = &processes[self.scroll_offset..end_index];

        let mut lines = Vec::new();
        for (idx, process) in visible_processes.iter().enumerate() {
            let global_idx = self.scroll_offset + idx;
            let is_selected = global_idx == self.selected_index;

            let bg_color = if is_selected {
                Color::DarkGray
            } else {
                Color::Reset
            };

            let style = Style::default().bg(bg_color);

            let cpu_color = if process.cpu_percent > 80.0 {
                Color::Red
            } else if process.cpu_percent > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let mem_color = if process.memory_percent > 80.0 {
                Color::Red
            } else if process.memory_percent > 50.0 {
                Color::Yellow
            } else {
                Color::Cyan
            };

            // Calculate available width for name/command column
            // USER(9) + PID(9) + CPU(8) + MEM(8) + separators(~8) = 42
            let name_width = list_area.width.saturating_sub(42) as usize;

            // Choose display text based on mode and truncate if needed
            let display_text = if show_full_command {
                // Check if cmdline is empty or just "[PID]" format (kernel threads)
                let text = if process.cmdline.is_empty()
                    || process.cmdline.starts_with('[') && process.cmdline.ends_with(']') && process.cmdline[1..process.cmdline.len()-1].parse::<u32>().is_ok() {
                    &process.name
                } else {
                    &process.cmdline
                };

                if text.len() > name_width {
                    let truncate_at = name_width.saturating_sub(3);
                    let mut end = truncate_at;
                    while end > 0 && !text.is_char_boundary(end) {
                        end -= 1;
                    }
                    format!("{}...", &text[..end])
                } else {
                    text.to_string()
                }
            } else {
                if process.name.len() > name_width {
                    let truncate_at = name_width.saturating_sub(3);
                    let mut end = truncate_at;
                    while end > 0 && !process.name.is_char_boundary(end) {
                        end -= 1;
                    }
                    format!("{}...", &process.name[..end])
                } else {
                    process.name.clone()
                }
            };

            // Truncate username if too long
            let user_display = if process.user.len() > 8 {
                format!("{}+", &process.user[..7])
            } else {
                format!("{:<8}", process.user)
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", user_display), style.fg(Color::Cyan)),
                Span::styled("│ ", style.fg(Color::DarkGray)),
                Span::styled(format!("{:>7} ", process.pid), style.fg(Color::White)),
                Span::styled("│ ", style.fg(Color::DarkGray)),
                Span::styled(format!("{:>5.1} ", process.cpu_percent), style.fg(cpu_color)),
                Span::styled("│ ", style.fg(Color::DarkGray)),
                Span::styled(format!("{:>5.1} ", process.memory_percent), style.fg(mem_color)),
                Span::styled("│ ", style.fg(Color::DarkGray)),
                Span::styled(
                    display_text,
                    style.fg(if is_selected { Color::White } else { Color::Gray }),
                ),
            ]);

            lines.push(line);
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, list_area);
    }

    pub fn scroll_up(&mut self, count: usize) {
        self.selected_index = self.selected_index.saturating_sub(count);
    }

    pub fn scroll_down(&mut self, count: usize, max: usize) {
        if max > 0 {
            self.selected_index = (self.selected_index + count).min(max - 1);
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_up(page_size);
    }

    pub fn page_down(&mut self, page_size: usize, max: usize) {
        self.scroll_down(page_size, max);
    }

    pub fn home(&mut self) {
        self.selected_index = 0;
    }

    pub fn end(&mut self, max: usize) {
        if max > 0 {
            self.selected_index = max - 1;
        }
    }

    pub fn adjust_scroll(&mut self, visible_height: usize) {
        // Adjust scroll offset to keep selected item visible
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn set_selected_index(&mut self, index: usize) {
        self.selected_index = index;
    }
}

use crate::collectors::network::NetworkStats;

pub struct NetworkWidget {
    scroll_offset: usize,
}

impl NetworkWidget {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
        }
    }

    fn format_bytes_per_sec(bytes: f64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        if bytes >= GB {
            format!("{:.2} GB/s", bytes / GB)
        } else if bytes >= MB {
            format!("{:.1} MB/s", bytes / MB)
        } else if bytes >= KB {
            format!("{:.1} KB/s", bytes / KB)
        } else {
            format!("{:.0} B/s", bytes)
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        stats: &[NetworkStats],
        theme: &Theme,
        is_detail: bool,
        processes: &[ProcessStats],
    ) {
        let title = if is_detail {
            " Network [Detail] "
        } else {
            " Network "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if stats.is_empty() {
            let text = Paragraph::new("No active network interfaces");
            frame.render_widget(text, inner);
            return;
        }

        let mut lines = Vec::new();

        // Apply scroll offset for scrolling through network interfaces
        let visible_interfaces: Vec<&NetworkStats> = stats
            .iter()
            .skip(self.scroll_offset)
            .collect();

        for (idx, iface) in visible_interfaces.iter().enumerate() {
            // Check if we've exceeded visible height
            if lines.len() >= inner.height as usize - 1 {
                break;
            }

            if idx > 0 {
                lines.push(Line::from(""));
            }

            // Interface name
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}", iface.interface),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]));

            // Download (RX) gauge
            let rx_display = Self::format_bytes_per_sec(iface.rx_bytes_per_sec);
            let rx_color = if iface.rx_bytes_per_sec > 100_000_000.0 {
                // > 100 MB/s
                Color::Red
            } else if iface.rx_bytes_per_sec > 10_000_000.0 {
                // > 10 MB/s
                Color::Yellow
            } else {
                Color::Green
            };

            lines.push(Line::from(vec![
                Span::styled("  ↓ ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:>12}", rx_display),
                    Style::default().fg(rx_color),
                ),
            ]));

            // Upload (TX) gauge
            let tx_display = Self::format_bytes_per_sec(iface.tx_bytes_per_sec);
            let tx_color = if iface.tx_bytes_per_sec > 100_000_000.0 {
                Color::Red
            } else if iface.tx_bytes_per_sec > 10_000_000.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            lines.push(Line::from(vec![
                Span::styled("  ↑ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:>12}", tx_display),
                    Style::default().fg(tx_color),
                ),
            ]));

            // Detail mode: Show packet statistics and cumulative totals
            if is_detail {
                // Packet statistics
                lines.push(Line::from(vec![
                    Span::styled("    Packets: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("↓", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("{}/s ", iface.rx_packets),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled("↑", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("{}/s", iface.tx_packets),
                        Style::default().fg(Color::Green),
                    ),
                ]));

                // Cumulative totals since boot
                let rx_total_gb = iface.rx_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let tx_total_gb = iface.tx_bytes as f64 / 1024.0 / 1024.0 / 1024.0;

                lines.push(Line::from(vec![
                    Span::styled("    Total: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("↓{:.2} GB  ↑{:.2} GB", rx_total_gb, tx_total_gb),
                        Style::default().fg(Color::Gray),
                    ),
                ]));
            } else {
                // Total statistics (if there's space)
                if inner.height > (stats.len() * 4 + 2) as u16 {
                    let rx_total_mb = iface.rx_bytes as f64 / 1024.0 / 1024.0;
                    let tx_total_mb = iface.tx_bytes as f64 / 1024.0 / 1024.0;

                    lines.push(Line::from(vec![
                        Span::styled("    Total: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("↓{:.1}MB ↑{:.1}MB", rx_total_mb, tx_total_mb),
                            Style::default().fg(Color::Gray),
                        ),
                    ]));
                }
            }
        }

        // Add process attribution in detail mode
        if is_detail && lines.len() < inner.height as usize - 3 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "Note: Per-process network tracking requires eBPF (future enhancement)",
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "Active processes (by CPU, not network):",
                    Style::default().fg(Color::Cyan),
                ),
            ]));

            // Show top CPU processes as proxy for network activity
            let mut sorted_processes: Vec<&ProcessStats> = processes.iter().collect();
            sorted_processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));

            for process in sorted_processes.iter().take(5) {
                if process.cpu_percent > 1.0 && lines.len() < inner.height as usize - 1 {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("{} - {:.1}% CPU",
                                if process.name.len() > 35 {
                                    format!("{}...", &process.name[..32])
                                } else {
                                    process.name.clone()
                                }
                            ,process.cpu_percent),
                            Style::default().fg(Color::Gray),
                        ),
                    ]));
                }
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    pub fn scroll_up(&mut self, count: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(count);
    }

    pub fn scroll_down(&mut self, count: usize, max: usize) {
        if max > 0 {
            let max_scroll = max.saturating_sub(1);
            self.scroll_offset = (self.scroll_offset + count).min(max_scroll);
        }
    }
}

pub struct DiskIoWidget {
    max_devices: usize,
    scroll_offset: usize,
}

impl DiskIoWidget {
    pub fn new() -> Self {
        Self {
            max_devices: 10, // Show top 10 most active devices
            scroll_offset: 0,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, io_stats: &[IoStats], theme: &Theme, is_detail: bool, processes: &[crate::collectors::process::ProcessStats]) {
        let block = Block::default()
            .title(" Disk I/O ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if io_stats.is_empty() {
            let text = Paragraph::new("No disk activity");
            frame.render_widget(text, inner);
            return;
        }

        // Filter only devices with activity
        let active_devices: Vec<_> = io_stats
            .iter()
            .filter(|stat| {
                stat.read_bytes_per_sec > 0.0 || stat.write_bytes_per_sec > 0.0
            })
            .collect();

        // If no active devices, show all devices
        let all_devices: Vec<_> = if active_devices.is_empty() {
            io_stats.iter().collect()
        } else {
            active_devices
        };

        // Apply scroll offset for scrolling through devices
        let devices_to_show: Vec<_> = all_devices
            .iter()
            .skip(self.scroll_offset)
            .collect();

        let mut lines = Vec::new();

        for stat in &devices_to_show {
            // Check if we've exceeded visible height
            if lines.len() >= inner.height as usize {
                break;
            }
            // Calculate MB/s
            let read_mb = stat.read_bytes_per_sec / 1024.0 / 1024.0;
            let write_mb = stat.write_bytes_per_sec / 1024.0 / 1024.0;

            // Determine color intensity based on I/O rate
            let read_color = if read_mb > 100.0 {
                Color::Red
            } else if read_mb > 10.0 {
                Color::Yellow
            } else if read_mb > 0.1 {
                Color::Green
            } else {
                Color::DarkGray
            };

            let write_color = if write_mb > 100.0 {
                Color::Red
            } else if write_mb > 10.0 {
                Color::Yellow
            } else if write_mb > 0.1 {
                Color::Magenta
            } else {
                Color::DarkGray
            };

            // Device name line (more compact format)
            let device_line = Line::from(vec![
                Span::styled(
                    format!("{:<10}", stat.device),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" R:", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:>7.2}", read_mb),
                    Style::default().fg(read_color),
                ),
                Span::styled(" W:", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:>7.2} MB/s", write_mb),
                    Style::default().fg(write_color),
                ),
            ]);

            lines.push(device_line);

            // Add progress bars if there's space
            if lines.len() + 3 < inner.height as usize {
                // Read bar
                let read_percent = (read_mb / 200.0 * 100.0).min(100.0); // Scale to 200 MB/s max
                let read_bar_width = (inner.width as f64 * 0.7 * read_percent / 100.0) as usize;
                let read_bar = "█".repeat(read_bar_width);

                let read_bar_line = Line::from(vec![
                    Span::styled("  R ", Style::default().fg(Color::DarkGray)),
                    Span::styled(read_bar, Style::default().fg(read_color)),
                ]);
                lines.push(read_bar_line);

                // Write bar
                let write_percent = (write_mb / 200.0 * 100.0).min(100.0); // Scale to 200 MB/s max
                let write_bar_width = (inner.width as f64 * 0.7 * write_percent / 100.0) as usize;
                let write_bar = "█".repeat(write_bar_width);

                let write_bar_line = Line::from(vec![
                    Span::styled("  W ", Style::default().fg(Color::DarkGray)),
                    Span::styled(write_bar, Style::default().fg(write_color)),
                ]);
                lines.push(write_bar_line);

                // Add spacing between devices
                if lines.len() < inner.height as usize {
                    lines.push(Line::from(""));
                }
            }

            // Stop if we've filled the available space
            if lines.len() >= inner.height as usize {
                break;
            }
        }

        // If in detail mode and there's space, show per-process I/O
        if is_detail && lines.len() < inner.height as usize {
            // Add separator
            if !lines.is_empty() && lines.len() + 2 < inner.height as usize {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Top Processes by Disk I/O:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(""));
            }

            // Sort processes by total I/O rate (read + write)
            let mut sorted_processes: Vec<_> = processes.iter()
                .filter(|p| p.io_read_rate > 0.0 || p.io_write_rate > 0.0)
                .collect();
            sorted_processes.sort_by(|a, b| {
                let a_total = a.io_read_rate + a.io_write_rate;
                let b_total = b.io_read_rate + b.io_write_rate;
                b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Display top 10 processes
            for process in sorted_processes.iter().take(10) {
                if lines.len() >= inner.height as usize {
                    break;
                }

                let read_mb = process.io_read_rate / 1024.0 / 1024.0;
                let write_mb = process.io_write_rate / 1024.0 / 1024.0;
                let total_mb = read_mb + write_mb;

                // Color code based on total I/O rate
                let io_color = if total_mb > 10.0 {
                    Color::Red
                } else if total_mb > 1.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                let process_name = if process.name.len() > 20 {
                    format!("{}...", &process.name[..17])
                } else {
                    process.name.clone()
                };

                let line = Line::from(vec![
                    Span::styled(format!("{:>6} ", process.pid), Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:<20} ", process_name), Style::default().fg(Color::Gray)),
                    Span::styled("R:", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:>7.2} ", read_mb), Style::default().fg(io_color)),
                    Span::styled("W:", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:>7.2} MB/s", write_mb), Style::default().fg(io_color)),
                ]);
                lines.push(line);
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    pub fn set_max_devices(&mut self, max: usize) {
        self.max_devices = max;
    }

    pub fn scroll_up(&mut self, count: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(count);
    }

    pub fn scroll_down(&mut self, count: usize, max: usize) {
        if max > 0 {
            let max_scroll = max.saturating_sub(1);
            self.scroll_offset = (self.scroll_offset + count).min(max_scroll);
        }
    }
}

use crate::collectors::disk::DiskStats;

pub struct DiskWidget {
    scroll_offset: usize,
}

impl DiskWidget {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        disks: &[DiskStats],
        theme: &Theme,
        is_detail: bool,
    ) {
        let block = Block::default()
            .title(" Disks ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if disks.is_empty() {
            let text = Paragraph::new("No disks found");
            frame.render_widget(text, inner);
            return;
        }

        let mut lines = Vec::new();

        for disk in disks.iter().skip(self.scroll_offset) {
            // Format sizes
            let total_gb = disk.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_gb = disk.used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
            let available_gb = disk.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0;

            // Color code based on usage
            let usage_color = if disk.usage_percent >= 90.0 {
                Color::Red
            } else if disk.usage_percent >= 70.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            // Mount point and device line
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<20}", Self::truncate_string(&disk.mount_point, 20)),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(
                    Self::truncate_string(&disk.device, 30),
                    Style::default().fg(Color::Gray),
                ),
            ]));

            // Usage bar
            let bar_width = inner.width.saturating_sub(25) as usize;
            let filled = ((disk.usage_percent / 100.0) * bar_width as f64) as usize;
            let empty = bar_width.saturating_sub(filled);

            let mut bar_spans = vec![Span::styled("[", Style::default().fg(Color::DarkGray))];

            if filled > 0 {
                bar_spans.push(Span::styled(
                    "=".repeat(filled.saturating_sub(1)),
                    Style::default().fg(usage_color),
                ));
                if filled > 0 {
                    bar_spans.push(Span::styled(">", Style::default().fg(usage_color)));
                }
            }

            if empty > 0 {
                bar_spans.push(Span::styled(" ".repeat(empty), Style::default()));
            }

            bar_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
            bar_spans.push(Span::styled(
                format!(" {:.1}%", disk.usage_percent),
                Style::default().fg(usage_color).add_modifier(Modifier::BOLD),
            ));

            lines.push(Line::from(bar_spans));

            // Size information
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{:.2} GB used", used_gb),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(" / ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2} GB total", total_gb),
                    Style::default().fg(Color::Blue),
                ),
                Span::styled("  (", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2} GB free", available_gb),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(")", Style::default().fg(Color::DarkGray)),
            ]));

            // Filesystem type
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("Type: ", Style::default().fg(Color::Gray)),
                Span::styled(&disk.fs_type, Style::default().fg(Color::White)),
            ]));

            // Spacer line
            if lines.len() < inner.height as usize {
                lines.push(Line::from(""));
            }

            // Stop if we've filled the area
            if lines.len() >= inner.height as usize {
                break;
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, max_disks: usize, visible_height: usize) {
        let max_scroll = max_disks.saturating_sub(visible_height / 5);
        self.scroll_offset = (self.scroll_offset + 1).min(max_scroll);
    }
}

// GPU monitoring widget with multi-vendor support
use crate::gpu::{GpuStats, GpuProcess};

pub struct GpuWidget {
    scroll_offset: usize,
}

impl GpuWidget {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        gpus: &[GpuStats],
        theme: &Theme,
        is_detail: bool,
    ) {
        let title = if is_detail {
            " GPU [Detail] "
        } else {
            " GPU "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if gpus.is_empty() {
            let text = Paragraph::new("No GPUs detected");
            frame.render_widget(text, inner);
            return;
        }

        let mut lines = Vec::new();

        for (idx, gpu) in gpus.iter().enumerate().skip(self.scroll_offset) {
            // Vendor color
            let vendor_color = match gpu.vendor.as_str() {
                "NVIDIA" => Color::Green,
                "AMD" => Color::Red,
                "Intel" => Color::Blue,
                _ => Color::White,
            };

            // GPU name header
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}. ", idx + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    &gpu.vendor,
                    Style::default().fg(vendor_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ", Style::default()),
                Span::styled(
                    &gpu.name,
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]));

            // Utilization gauge
            if inner.width > 25 {
                let gauge_width = inner.width.saturating_sub(10) as usize;
                let filled = ((gpu.utilization_percent / 100.0) * gauge_width as f64) as usize;
                let empty = gauge_width.saturating_sub(filled);

                let util_color = if gpu.utilization_percent >= 80.0 {
                    Color::Red
                } else if gpu.utilization_percent >= 50.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                let mut gauge_spans = vec![Span::styled("[", Style::default().fg(Color::DarkGray))];

                if filled > 0 {
                    gauge_spans.push(Span::styled(
                        "█".repeat(filled),
                        Style::default().fg(util_color),
                    ));
                }

                if empty > 0 {
                    gauge_spans.push(Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)));
                }

                gauge_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
                gauge_spans.push(Span::styled(
                    format!(" {:.1}%", gpu.utilization_percent),
                    Style::default().fg(util_color).add_modifier(Modifier::BOLD),
                ));

                lines.push(Line::from(gauge_spans));
            }

            // VRAM usage
            let memory_percent = if gpu.memory_total_mb > 0 {
                (gpu.memory_used_mb as f64 / gpu.memory_total_mb as f64) * 100.0
            } else {
                0.0
            };

            let memory_color = if memory_percent >= 90.0 {
                Color::Red
            } else if memory_percent >= 70.0 {
                Color::Yellow
            } else {
                Color::Cyan
            };

            lines.push(Line::from(vec![
                Span::styled("  VRAM: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} MB", gpu.memory_used_mb),
                    Style::default().fg(memory_color),
                ),
                Span::styled(" / ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} MB", gpu.memory_total_mb),
                    Style::default().fg(Color::Blue),
                ),
                Span::styled(
                    format!(" ({:.1}%)", memory_percent),
                    Style::default().fg(memory_color),
                ),
            ]));

            // Temperature
            let temp_color = if gpu.temperature_c >= 80.0 {
                Color::Red
            } else if gpu.temperature_c >= 60.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            lines.push(Line::from(vec![
                Span::styled("  Temp: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}°C", gpu.temperature_c),
                    Style::default().fg(temp_color).add_modifier(Modifier::BOLD),
                ),
            ]));

            // Power
            lines.push(Line::from(vec![
                Span::styled("  Power: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1} W", gpu.power_watts),
                    Style::default().fg(Color::Magenta),
                ),
            ]));

            // Detail mode: Show extended metrics note
            if is_detail {
                lines.push(Line::from(vec![
                    Span::styled(
                        "  Note: Clock speeds & PCIe info: future NVML integration",
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }

            // Spacer between GPUs
            if idx < gpus.len() - 1 {
                lines.push(Line::from(""));
            }

            // Stop if we've filled the area
            if lines.len() >= inner.height as usize {
                break;
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, max_gpus: usize) {
        if self.scroll_offset < max_gpus.saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }
}

pub struct GpuProcessesWidget {
    scroll_offset: usize,
}

impl GpuProcessesWidget {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        gpus: &[GpuStats],
        theme: &Theme,
    ) {
        let block = Block::default()
            .title(" GPU Processes ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Collect all GPU processes from all GPUs
        let all_processes: Vec<(&GpuProcess, u32)> = gpus
            .iter()
            .flat_map(|gpu| {
                gpu.processes.iter().map(move |proc| (proc, gpu.gpu_id))
            })
            .collect();

        if all_processes.is_empty() {
            let text = Paragraph::new("No GPU processes detected");
            frame.render_widget(text, inner);
            return;
        }

        let mut lines = Vec::new();

        // Header
        let header = Line::from(vec![
            Span::styled("GPU ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("   PID ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("GPU MB ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("GPU%  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Type    ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Name", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]);
        lines.push(header);

        // Calculate visible range
        let visible_height = (inner.height as usize).saturating_sub(1); // -1 for header
        let end_index = (self.scroll_offset + visible_height).min(all_processes.len());
        let visible_processes = &all_processes[self.scroll_offset..end_index];

        // Process rows
        for (process, gpu_id) in visible_processes {
            let mem_color = if process.gpu_memory_mb > 1024 {
                Color::Red
            } else if process.gpu_memory_mb > 512 {
                Color::Yellow
            } else {
                Color::Green
            };

            let util_color = if process.gpu_utilization > 80 {
                Color::Red
            } else if process.gpu_utilization > 50 {
                Color::Yellow
            } else {
                Color::Green
            };

            let type_str = match &process.process_type {
                crate::gpu::GpuProcessType::Graphics => "Graphics",
                crate::gpu::GpuProcessType::Compute => "Compute ",
                crate::gpu::GpuProcessType::Both => "Both    ",
            };

            let type_color = match &process.process_type {
                crate::gpu::GpuProcessType::Graphics => Color::Blue,
                crate::gpu::GpuProcessType::Compute => Color::Magenta,
                crate::gpu::GpuProcessType::Both => Color::Cyan,
            };

            // Calculate available width for process name
            // GPU(4) + PID(9) + GPU MB(9) + GPU%(8) + Type(10) + separators(~10) = 50
            let name_width = inner.width.saturating_sub(50) as usize;
            let display_name = if process.process_name.len() > name_width {
                let truncate_at = name_width.saturating_sub(3);
                let mut end = truncate_at;
                while end > 0 && !process.process_name.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}...", &process.process_name[..end])
            } else {
                process.process_name.clone()
            };

            let line = Line::from(vec![
                Span::styled(format!("{:>3} ", gpu_id), Style::default().fg(Color::Cyan)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>7} ", process.pid), Style::default().fg(Color::White)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>6} ", process.gpu_memory_mb), Style::default().fg(mem_color)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>4}% ", process.gpu_utilization), Style::default().fg(util_color)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} ", type_str), Style::default().fg(type_color)),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(display_name, Style::default().fg(Color::Gray)),
            ]);
            lines.push(line);
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    pub fn scroll_up(&mut self, count: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(count);
    }

    pub fn scroll_down(&mut self, count: usize, max: usize) {
        if max > 0 {
            let max_scroll = max.saturating_sub(1);
            self.scroll_offset = (self.scroll_offset + count).min(max_scroll);
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_up(page_size);
    }

    pub fn page_down(&mut self, page_size: usize, max: usize) {
        self.scroll_down(page_size, max);
    }

    pub fn home(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn end(&mut self, max: usize) {
        if max > 0 {
            self.scroll_offset = max.saturating_sub(1);
        }
    }
}
