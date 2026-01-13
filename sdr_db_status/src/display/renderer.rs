use super::layout::*;
use crate::metrics::ClusterMetrics;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_6X12, ascii::FONT_9X15_BOLD, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

/// Renders status display to embedded-graphics compatible displays
pub struct StatusRenderer {
    text_style: MonoTextStyle<'static, Rgb565>,
    title_style: MonoTextStyle<'static, Rgb565>,
    small_style: MonoTextStyle<'static, Rgb565>,
}

impl StatusRenderer {
    pub fn new() -> Self {
        Self {
            text_style: MonoTextStyle::new(&FONT_6X12, Rgb565::WHITE),
            title_style: MonoTextStyle::new(&FONT_9X15_BOLD, Rgb565::CYAN),
            small_style: MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
        }
    }

    /// Render the complete status screen
    pub fn render<D>(&self, display: &mut D, metrics: &ClusterMetrics) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Clear screen
        Rectangle::new(pt(0, 0), sz(DISPLAY_WIDTH, DISPLAY_HEIGHT))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)?;

        // Render all sections
        self.render_header(display, metrics)?;
        self.render_cluster_status(display, metrics)?;
        self.render_database_stats(display, metrics)?;
        self.render_api_stats(display, metrics)?;
        self.render_signal_stats(display, metrics)?;
        self.render_system_stats(display, metrics)?;

        Ok(())
    }

    fn render_header<D>(&self, display: &mut D, metrics: &ClusterMetrics) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Title
        Text::new(
            "SDR_DB CLUSTER",
            pt(MARGIN, HEADER_Y + 12),
            self.title_style,
        )
        .draw(display)?;

        // Timestamp (right-aligned)
        let time_str = metrics.timestamp.format("%H:%M:%S").to_string();
        Text::with_alignment(
            &time_str,
            pt(DISPLAY_WIDTH as i32 - MARGIN, HEADER_Y + 12),
            self.small_style,
            Alignment::Right,
        )
        .draw(display)?;

        Ok(())
    }

    fn render_cluster_status<D>(
        &self,
        display: &mut D,
        metrics: &ClusterMetrics,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Section label
        Text::new("NODES:", pt(MARGIN, CLUSTER_Y + 10), self.text_style).draw(display)?;

        // Node indicators (circles)
        for (i, node) in metrics.nodes.iter().enumerate() {
            let x = NODE_INDICATOR_X + (i as i32 * NODE_INDICATOR_SPACING);
            let color = if node.is_up {
                Rgb565::GREEN
            } else {
                Rgb565::RED
            };

            Circle::new(pt(x, CLUSTER_Y + 2), NODE_INDICATOR_RADIUS * 2)
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(display)?;
        }

        // Node count summary
        let up_count = metrics.nodes.iter().filter(|n| n.is_up).count();
        let total = metrics.nodes.len();
        let status_text = format!("{}/{} UP", up_count, total);

        Text::with_alignment(
            &status_text,
            pt(DISPLAY_WIDTH as i32 - MARGIN, CLUSTER_Y + 10),
            self.text_style,
            Alignment::Right,
        )
        .draw(display)?;

        Ok(())
    }

    fn render_database_stats<D>(
        &self,
        display: &mut D,
        metrics: &ClusterMetrics,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Section title
        Text::new("DATABASE", pt(MARGIN, DATABASE_Y + 10), self.text_style).draw(display)?;

        // Total logs with rate
        let logs_text = format!(
            "Logs: {}  (+{:.0}/min)",
            format_number(metrics.database.total_logs),
            metrics.database.log_rate
        );
        Text::new(
            &logs_text,
            pt(MARGIN, DATABASE_Y + LINE_HEIGHT + 10),
            self.small_style,
        )
        .draw(display)?;

        // Connection pool status
        let conn_text = format!(
            "Conn: {}/{}    Lat: {}ms",
            metrics.database.active_connections,
            metrics.database.max_connections,
            metrics.database.avg_latency_ms
        );
        Text::new(
            &conn_text,
            pt(MARGIN, DATABASE_Y + LINE_HEIGHT * 2 + 10),
            self.small_style,
        )
        .draw(display)?;

        Ok(())
    }

    fn render_api_stats<D>(&self, display: &mut D, metrics: &ClusterMetrics) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Only show first 3 nodes to fit on screen
        for (i, node) in metrics.nodes.iter().take(3).enumerate() {
            let y = API_Y + (i as i32 * LINE_HEIGHT * 2);

            // Node name and request rate
            let node_text = format!("API ({})  ↑ {:.0} req/s", node.name, node.request_rate);
            Text::new(&node_text, pt(MARGIN, y + 10), self.small_style).draw(display)?;

            // Error rate and uptime
            let uptime_hrs = node.uptime.as_secs() / 3600;
            let uptime_days = uptime_hrs / 24;
            let uptime_str = if uptime_days > 0 {
                format!("{}d {}h", uptime_days, uptime_hrs % 24)
            } else {
                format!("{}h", uptime_hrs)
            };

            let stats_text = format!("Errors: {:.2}%   Up: {}", node.error_rate, uptime_str);
            Text::new(
                &stats_text,
                pt(MARGIN, y + LINE_HEIGHT + 10),
                self.small_style,
            )
            .draw(display)?;
        }

        Ok(())
    }

    fn render_signal_stats<D>(
        &self,
        display: &mut D,
        metrics: &ClusterMetrics,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Text::new("RF SIGNALS", pt(MARGIN, SIGNALS_Y + 10), self.text_style).draw(display)?;

        // Frequency range
        let freq_text = format!(
            "Freq: {:.1}-{:.0} MHz",
            metrics.signals.freq_range.0, metrics.signals.freq_range.1
        );
        Text::new(
            &freq_text,
            pt(MARGIN, SIGNALS_Y + LINE_HEIGHT + 10),
            self.small_style,
        )
        .draw(display)?;

        // Mode distribution
        let mode_text = format!(
            "FM({:.0}%) AM({:.0}%) Other({:.0}%)",
            metrics.signals.mode_distribution.fm_percent(),
            metrics.signals.mode_distribution.am_percent(),
            metrics.signals.mode_distribution.other_percent()
        );
        Text::new(
            &mode_text,
            pt(MARGIN, SIGNALS_Y + LINE_HEIGHT * 2 + 10),
            self.small_style,
        )
        .draw(display)?;

        // Grid squares
        let grid_text = format!("Grids: {} unique", metrics.signals.unique_grids);
        Text::new(
            &grid_text,
            pt(MARGIN, SIGNALS_Y + LINE_HEIGHT * 3 + 10),
            self.small_style,
        )
        .draw(display)?;

        Ok(())
    }

    fn render_system_stats<D>(
        &self,
        display: &mut D,
        metrics: &ClusterMetrics,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Text::new("SYSTEM", pt(MARGIN, SYSTEM_Y + 10), self.text_style).draw(display)?;

        // CPU and Memory
        let sys_text = format!(
            "CPU: {:.0}%   Mem: {:.1}/{:.1}GB",
            metrics.system.cpu_percent,
            metrics.system.memory_used_gb,
            metrics.system.memory_total_gb
        );
        Text::new(
            &sys_text,
            pt(MARGIN, SYSTEM_Y + LINE_HEIGHT + 10),
            self.small_style,
        )
        .draw(display)?;

        Ok(())
    }
}

impl Default for StatusRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Format large numbers with commas
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
