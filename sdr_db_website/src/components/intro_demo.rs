use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

fn now_secs() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() / 1000.0)
        .unwrap_or(0.0)
}

/// Renders an animated fake-spectrum demo strip into `area`.
pub fn render_intro_demo(frame: &mut Frame, area: ratatui::layout::Rect) {
    let t = now_secs();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("intro demo");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 4 || inner.height < 3 {
        return;
    }

    let n = inner.width as usize;
    let block_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let spectrum: String = (0..n)
        .map(|i| {
            let x = i as f64 / n as f64; // 0.0 .. 1.0

            // Noise floor: low-level ripple
            let noise = 0.06
                + 0.04 * (x * 137.0 + t * 0.7).sin().abs()
                + 0.03 * (x * 89.0 - t * 1.1).sin().abs();

            // Main carrier peak at x = 0.50 (146.520 MHz)
            let d0 = x - 0.50;
            let main = 0.85 * (-40.0 * d0 * d0).exp() * (1.0 + 0.05 * (t * 4.0).sin());

            // Weaker signal at x ≈ 0.30
            let d1 = x - 0.30;
            let sig1 = 0.38 * (-80.0 * d1 * d1).exp() * (1.0 + 0.08 * (t * 2.5 + 1.0).sin());

            // Another signal at x ≈ 0.70
            let d2 = x - 0.70;
            let sig2 = 0.28 * (-100.0 * d2 * d2).exp() * (1.0 + 0.06 * (t * 3.3 - 0.5).sin());

            let v = (noise + main + sig1 + sig2).clamp(0.0, 1.0);
            block_chars[(v * 7.0) as usize]
        })
        .collect();

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(spectrum, Style::default().fg(Color::Green))),
        Line::from(""),
        Line::from(Span::styled(
            "Welcome to SDR_DB! Made by 0x0f Softworks (nuts-rice)",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}
