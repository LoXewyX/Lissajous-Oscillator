use std::f32::consts::{PI, TAU};
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use eframe::{App, Frame, NativeOptions};

use rodio::{source::SineWave, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

// =============================================================================
// Constants
// =============================================================================

const MIN_FREQUENCY_HZ: f64 = 20.0;
const MAX_FREQUENCY_HZ: f64 = 20_000.0;

// -----------------------------------------------------------------------------
// Default audio
// -----------------------------------------------------------------------------

const DEFAULT_FREQUENCY_HZ: f64 = 220.0;
const DEFAULT_VOLUME: f32 = 0.30;

const AUDIO_GAIN: f32 = 0.42;

// -----------------------------------------------------------------------------
// Lissajous ratio
// -----------------------------------------------------------------------------
//
// Mathematical relationship:
//
//     a / b = X / Y
//
// Examples:
//
//     220 / 220.0 = 1.000000
//     220 / 220.5 = 0.997732
//     220 / 440.0 = 0.500000
//     220 / 660.0 = 0.333333
//
// The visual curve is:
//
//     x(t) = sin(a t)
//     y(t) = sin(b t + phase)
//
// For normal rational ratios phase = 0.
//
// Very close to 1:1, a controlled phase is introduced so a tiny frequency
// difference becomes visibly elliptical rather than looking like a single
// thick line.
//
// Exactly 1:1 remains phase = 0, therefore:
//
//     x(t) = sin(t)
//     y(t) = sin(t)
//
// which is the diagonal line you requested.
// -----------------------------------------------------------------------------

const DEFAULT_LISSAJOUS_RATIO: f32 = 1.0;

const MIN_LISSAJOUS_RATIO: f32 = 0.01;
const MAX_LISSAJOUS_RATIO: f32 = 1.0;

// The near-1 region where a frequency difference becomes visually enhanced.
//
// Example:
//
//     220 / 220.5 = 0.997732
//
// Difference:
//
//     |1 - ratio| = 0.002268
//
// This is inside the enhancement region.
const NEAR_ONE_RATIO_RANGE: f32 = 0.010;

// -----------------------------------------------------------------------------
// Visual animation
// -----------------------------------------------------------------------------

const DEFAULT_VISUAL_CYCLES_PER_SECOND: f32 = 1.0;

const MIN_VISUAL_CYCLES_PER_SECOND: f32 = 0.01;
const MAX_VISUAL_CYCLES_PER_SECOND: f32 = 2.0;

// -----------------------------------------------------------------------------
// 2D zoom
// -----------------------------------------------------------------------------

const DEFAULT_ZOOM: f32 = 0.0;

const MIN_ZOOM: f32 = -100.0;
const MAX_ZOOM: f32 = 100.0;

// -----------------------------------------------------------------------------
// Curve sampling
// -----------------------------------------------------------------------------

const CURVE_STEPS: usize = 6000;

const MAX_PERIOD_DENOMINATOR: u32 = 128;

// -----------------------------------------------------------------------------
// Layout
// -----------------------------------------------------------------------------

const MAIN_MARGIN: f32 = 18.0;
const PANEL_GAP: f32 = 18.0;

// -----------------------------------------------------------------------------
// Window
// -----------------------------------------------------------------------------

const WINDOW_WIDTH: f32 = 1250.0;
const WINDOW_HEIGHT: f32 = 780.0;

const MIN_WINDOW_WIDTH: f32 = 850.0;
const MIN_WINDOW_HEIGHT: f32 = 600.0;

// =============================================================================
// Visual rate presets
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisualRatePreset {
    VerySlow,
    Slow,
    Normal,
}

// =============================================================================
// Audio
// =============================================================================

struct AudioState {
    device: MixerDeviceSink,
    player: Option<Player>,
}

impl AudioState {
    fn new() -> anyhow::Result<Self> {
        let mut device = DeviceSinkBuilder::open_default_sink()?;

        device.log_on_drop(false);

        Ok(Self {
            device,
            player: None,
        })
    }

    fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            player.stop();
        }
    }

    fn set_volume(&self, volume: f32) {
        if let Some(player) = &self.player {
            player.set_volume(volume.clamp(0.0, 1.0) * AUDIO_GAIN);
        }
    }

    fn play(&mut self, x_hz: f32, y_hz: f32, volume: f32) -> anyhow::Result<()> {
        self.stop();

        let player = Player::connect_new(self.device.mixer());

        let gain = volume.clamp(0.0, 1.0) * AUDIO_GAIN;

        let x_source = SineWave::new(x_hz.max(1.0)).amplify(gain);

        let y_source = SineWave::new(y_hz.max(1.0)).amplify(gain);

        player.append(x_source.mix(y_source));

        player.play();

        self.player = Some(player);

        Ok(())
    }
}

// =============================================================================
// Application
// =============================================================================

struct NoteApp {
    // -------------------------------------------------------------------------
    // Audio frequencies
    // -------------------------------------------------------------------------
    x_hz: f64,
    y_hz: f64,

    // -------------------------------------------------------------------------
    // Audio lock
    // -------------------------------------------------------------------------
    lock_frequency: bool,

    // -------------------------------------------------------------------------
    // Audio X:Y ratio input
    // -------------------------------------------------------------------------
    frequency_input: String,
    parsed_ok: bool,

    // -------------------------------------------------------------------------
    // Playback
    // -------------------------------------------------------------------------
    playing: bool,
    volume: f32,

    // -------------------------------------------------------------------------
    // Mathematical Lissajous ratio
    //
    //     a / b = X / Y
    // -------------------------------------------------------------------------
    lissajous_ratio: f32,

    // -------------------------------------------------------------------------
    // Visual animation
    // -------------------------------------------------------------------------
    visual_cycles_per_second: f32,
    visual_rate_preset: VisualRatePreset,

    visual_time: f32,
    last_update: Instant,

    // -------------------------------------------------------------------------
    // 2D zoom
    // -------------------------------------------------------------------------
    zoom: f32,

    // -------------------------------------------------------------------------
    // UI
    // -------------------------------------------------------------------------
    volume_presets_expanded: bool,
    note_presets_expanded: bool,

    // -------------------------------------------------------------------------
    // Audio
    // -------------------------------------------------------------------------
    audio: Option<AudioState>,
    audio_error: Option<String>,
}

// =============================================================================
// Default
// =============================================================================

impl Default for NoteApp {
    fn default() -> Self {
        let mut app = Self {
            x_hz: DEFAULT_FREQUENCY_HZ,

            y_hz: DEFAULT_FREQUENCY_HZ,

            lock_frequency: true,

            frequency_input: "1:1".to_owned(),

            parsed_ok: true,

            playing: false,

            volume: DEFAULT_VOLUME,

            lissajous_ratio: DEFAULT_LISSAJOUS_RATIO,

            visual_cycles_per_second: DEFAULT_VISUAL_CYCLES_PER_SECOND,

            visual_rate_preset: VisualRatePreset::Normal,

            visual_time: 0.0,

            last_update: Instant::now(),

            zoom: DEFAULT_ZOOM,

            volume_presets_expanded: false,

            note_presets_expanded: false,

            audio: None,

            audio_error: None,
        };

        app.sync_lissajous_ratio_from_audio();
        app.initialize_audio();

        app
    }
}

// =============================================================================
// Parsing
// =============================================================================

impl NoteApp {
    fn initialize_audio(&mut self) {
        match AudioState::new() {
            Ok(audio) => {
                self.audio = Some(audio);
            }

            Err(err) => {
                self.audio_error = Some(format!("Audio output unavailable: {}", err,));
            }
        }
    }

    fn parse_expr(input: &str) -> Result<f64, String> {
        let expression = input.trim().replace('×', "*").replace('÷', "/");

        if expression.is_empty() {
            return Err("empty expression".to_owned());
        }

        meval::eval_str(expression).map_err(|_| "invalid expression".to_owned())
    }

    fn parse_frequency(input: &str) -> Result<f64, String> {
        let normalized = input.trim().to_ascii_lowercase();

        let expression = normalized.strip_suffix("hz").unwrap_or(&normalized).trim();

        let hz = Self::parse_expr(expression)?;

        if hz.is_finite() && hz >= MIN_FREQUENCY_HZ && hz <= MAX_FREQUENCY_HZ {
            Ok(hz)
        } else {
            Err("frequency out of range".to_owned())
        }
    }

    // -------------------------------------------------------------------------
    // Audio X:Y ratio
    // -------------------------------------------------------------------------

    fn parse_ratio_and_apply(&mut self, left: &str, right: &str) {
        let left_lower = left.to_ascii_lowercase();

        let right_lower = right.to_ascii_lowercase();

        let contains_hz = left_lower.contains("hz") || right_lower.contains("hz");

        if contains_hz {
            match (Self::parse_frequency(left), Self::parse_frequency(right)) {
                (Ok(x), Ok(y)) => {
                    self.x_hz = x;

                    self.y_hz = y;

                    self.parsed_ok = true;

                    self.sync_lissajous_ratio_from_audio();
                    self.update_frequency_input_from_audio();

                    self.restart_audio_if_playing();
                }

                _ => {
                    self.parsed_ok = false;
                }
            }

            return;
        }

        match (Self::parse_expr(left), Self::parse_expr(right)) {
            (Ok(x_ratio), Ok(y_ratio))
                if x_ratio.is_finite() && y_ratio.is_finite() && x_ratio > 0.0 && y_ratio > 0.0 =>
            {
                let reference_x = self.x_hz;

                let new_y = reference_x * (y_ratio / x_ratio);

                if new_y.is_finite() && new_y >= MIN_FREQUENCY_HZ && new_y <= MAX_FREQUENCY_HZ {
                    self.y_hz = new_y;

                    self.parsed_ok = true;

                    self.sync_lissajous_ratio_from_audio();
                    self.update_frequency_input_from_audio();

                    self.restart_audio_if_playing();
                } else {
                    self.parsed_ok = false;
                }
            }

            _ => {
                self.parsed_ok = false;
            }
        }
    }

    fn recompute_ratio(&mut self) {
        if !self.lock_frequency {
            return;
        }

        let input = self.frequency_input.trim().to_owned();

        if input.is_empty() {
            self.parsed_ok = false;

            return;
        }

        if let Some((left, right)) = input.split_once(':') {
            self.parse_ratio_and_apply(left, right);
        } else {
            self.parsed_ok = false;
        }
    }

    // -------------------------------------------------------------------------
    // Frequency lock
    // -------------------------------------------------------------------------

    fn set_frequency_lock(&mut self, locked: bool) {
        self.lock_frequency = locked;

        if locked {
            self.y_hz = self.x_hz;

            self.frequency_input = "1:1".to_owned();

            self.parsed_ok = true;

            self.sync_lissajous_ratio_from_audio();

            self.restart_audio_if_playing();
        }
    }

    // -------------------------------------------------------------------------
    // X frequency
    // -------------------------------------------------------------------------

    fn set_x_frequency(&mut self, frequency: f64) {
        self.x_hz = frequency.clamp(MIN_FREQUENCY_HZ, MAX_FREQUENCY_HZ);

        if self.lock_frequency {
            self.y_hz = self.x_hz;

            self.apply_current_ratio_to_x();
        }

        self.sync_lissajous_ratio_from_audio();
        self.update_frequency_input_from_audio();

        self.restart_audio_if_playing();
    }

    fn apply_current_ratio_to_x(&mut self) {
        if !self.lock_frequency {
            return;
        }

        let input = self.frequency_input.trim().to_owned();

        if let Some((left, right)) = input.split_once(':') {
            let left_result = Self::parse_expr(left);

            let right_result = Self::parse_expr(right);

            if let (Ok(left_ratio), Ok(right_ratio)) = (left_result, right_result) {
                if left_ratio > 0.0
                    && right_ratio > 0.0
                    && left_ratio.is_finite()
                    && right_ratio.is_finite()
                {
                    let new_y = self.x_hz * (right_ratio / left_ratio);

                    if new_y.is_finite() && new_y >= MIN_FREQUENCY_HZ && new_y <= MAX_FREQUENCY_HZ {
                        self.y_hz = new_y;
                    }
                }
            }
        } else {
            self.y_hz = self.x_hz;
        }
    }
}

// =============================================================================
// Ratio synchronization
// =============================================================================

impl NoteApp {
    /// Synchronize:
    ///
    ///     a/b = X/Y
    fn sync_lissajous_ratio_from_audio(&mut self) {
        let ratio = self.x_hz / self.y_hz.max(MIN_FREQUENCY_HZ);

        self.lissajous_ratio =
            ratio.clamp(MIN_LISSAJOUS_RATIO as f64, MAX_LISSAJOUS_RATIO as f64) as f32;
    }

    /// Changing a/b changes Y audio.
    ///
    /// X remains reference:
    ///
    ///     Y = X / (a/b)
    fn set_lissajous_ratio_and_audio(&mut self, ratio: f32) {
        let ratio = ratio.clamp(MIN_LISSAJOUS_RATIO, MAX_LISSAJOUS_RATIO);

        let new_y = self.x_hz / ratio as f64;

        if !new_y.is_finite() || new_y < MIN_FREQUENCY_HZ || new_y > MAX_FREQUENCY_HZ {
            return;
        }

        self.lissajous_ratio = ratio;

        self.y_hz = new_y;

        self.update_frequency_input_from_audio();

        self.parsed_ok = true;

        self.restart_audio_if_playing();
    }

    fn update_frequency_input_from_audio(&mut self) {
        let ratio = self.y_hz / self.x_hz;

        self.frequency_input = format!("1:{}", pretty_num(ratio,),);
    }
}

// =============================================================================
// Audio
// =============================================================================

impl NoteApp {
    fn restart_audio_if_playing(&mut self) {
        if self.playing {
            self.restart_audio();
        }
    }

    fn restart_audio(&mut self) {
        let result = match &mut self.audio {
            Some(audio) => audio.play(self.x_hz as f32, self.y_hz as f32, self.volume),

            _ => Err(anyhow::anyhow!("audio output is unavailable",)),
        };

        match result {
            Ok(()) => {
                self.audio_error = None;
            }

            Err(err) => {
                self.audio_error = Some(format!("Audio: {}", err,));
            }
        }
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);

        if self.playing {
            if let Some(audio) = &self.audio {
                audio.set_volume(self.volume);
            }
        }
    }

    fn toggle_play(&mut self) {
        self.playing = !self.playing;

        if self.playing {
            self.visual_time = 0.0;

            self.last_update = Instant::now();

            self.restart_audio();
        } else if let Some(audio) = &mut self.audio {
            audio.stop();
        }
    }
}

// =============================================================================
// Lissajous mathematics
// =============================================================================

impl NoteApp {
    fn lissajous_frequencies(&self) -> (f32, f32) {
        (self.lissajous_ratio, 1.0)
    }

    fn visual_ratio(&self) -> f32 {
        self.lissajous_ratio
            .clamp(MIN_LISSAJOUS_RATIO, MAX_LISSAJOUS_RATIO)
    }

    // -------------------------------------------------------------------------
    // Near-1 phase enhancement
    // -------------------------------------------------------------------------
    //
    // Exactly 1:1:
    //
    //     phase = 0
    //
    //     x = sin(t)
    //     y = sin(t)
    //
    // -> straight diagonal line.
    //
    // At 220 / 220.5:
    //
    //     ratio ≈ 0.997732
    //
    // phase becomes non-zero, making the tiny frequency difference visible
    // as an ellipse.
    //
    // The transition is smooth.
    // -------------------------------------------------------------------------

    fn lissajous_phase(&self) -> f32 {
        let ratio = self.visual_ratio();

        let distance = (1.0 - ratio).abs();

        if distance <= f32::EPSILON {
            return 0.0;
        }

        let normalized = (distance / NEAR_ONE_RATIO_RANGE).clamp(0.0, 1.0);

        // Smoothstep.
        let smooth = normalized * normalized * (3.0 - 2.0 * normalized);

        // Keep the phase at zero exactly at 1:1.
        // As the ratio moves away from 1, gradually approach π/2.
        PI * 0.5 * smooth
    }

    /// Standard Lissajous equation with a near-1 visual phase adjustment.
    ///
    ///     x(t) = sin(a t)
    ///     y(t) = sin(b t + phase)
    fn lissajous_xy(&self, t: f32) -> (f32, f32) {
        let (a, b) = self.lissajous_frequencies();

        let phase = self.lissajous_phase();

        let x = (a * t).sin();

        let y = (b * t + phase).sin();

        (x, y)
    }

    // -------------------------------------------------------------------------
    // Complete mathematical period
    // -------------------------------------------------------------------------
    //
    // For ratios that are clearly rational:
    //
    //     a/b = p/q
    //
    // period:
    //
    //     T = 2πq
    //
    // Near 1:1, a tiny difference such as 220/220.5 has an enormous exact
    // beat period if represented as a rational number. That is not useful
    // for displaying the local ellipse, so near 1 we deliberately display
    // one normalized cycle.
    // -------------------------------------------------------------------------

    fn mathematical_period(&self) -> f32 {
        let ratio = self.visual_ratio();

        if ratio <= 0.000001 {
            return TAU;
        }

        // ---------------------------------------------------------------------
        // Near 1:1.
        //
        // Example:
        //
        //     220 / 220.5 ≈ 0.997732
        //
        // We want to inspect the local ellipse rather than require thousands
        // of cycles before an exact near-rational closure occurs.
        // ---------------------------------------------------------------------

        if (1.0 - ratio).abs() < NEAR_ONE_RATIO_RANGE {
            return TAU;
        }

        // ---------------------------------------------------------------------
        // Rational approximation for normal Lissajous figures.
        // ---------------------------------------------------------------------

        let mut best_denominator = 1u32;

        let mut best_error = f32::MAX;

        for denominator in 1..=MAX_PERIOD_DENOMINATOR {
            let numerator = (ratio * denominator as f32).round();

            if numerator <= 0.0 {
                continue;
            }

            let approximation = numerator / denominator as f32;

            let error = (approximation - ratio).abs();

            if error < best_error {
                best_error = error;

                best_denominator = denominator;
            }

            if error < 0.00001 {
                break;
            }
        }

        TAU * best_denominator as f32
    }

    fn current_math_time(&self) -> f32 {
        self.visual_time * self.mathematical_period()
    }
}

// =============================================================================
// Visual rate
// =============================================================================

impl NoteApp {
    fn target_visual_cycles_per_second(&self) -> f32 {
        match self.visual_rate_preset {
            VisualRatePreset::VerySlow => 0.10,

            VisualRatePreset::Slow => 0.50,

            VisualRatePreset::Normal => 1.00,
        }
    }

    fn set_visual_cycles_per_second(&mut self, value: f32) {
        self.visual_cycles_per_second =
            value.clamp(MIN_VISUAL_CYCLES_PER_SECOND, MAX_VISUAL_CYCLES_PER_SECOND);

        const EPSILON: f32 = 0.0005;

        if (self.visual_cycles_per_second - 0.10).abs() < EPSILON {
            self.visual_rate_preset = VisualRatePreset::VerySlow;
        } else if (self.visual_cycles_per_second - 0.50).abs() < EPSILON {
            self.visual_rate_preset = VisualRatePreset::Slow;
        } else if (self.visual_cycles_per_second - 1.00).abs() < EPSILON {
            self.visual_rate_preset = VisualRatePreset::Normal;
        }
    }

    fn apply_visual_rate_preset(&mut self, preset: VisualRatePreset) {
        self.visual_rate_preset = preset;

        self.visual_cycles_per_second = self.target_visual_cycles_per_second();
    }
}

// =============================================================================
// Animation
// =============================================================================

impl NoteApp {
    fn update_animation(&mut self) {
        let now = Instant::now();

        let dt = now.duration_since(self.last_update).as_secs_f32();

        self.last_update = now;

        if self.playing {
            self.visual_time += dt * self.visual_cycles_per_second;

            // visual_time is normalized:
            //
            //     0.0 = beginning of displayed curve
            //     1.0 = end of displayed curve
            //
            // Keep the dot inside exactly the same interval used by
            // draw_lissajous_curve().
            self.visual_time = self.visual_time.rem_euclid(1.0);
        }
    }
}

// =============================================================================
// Zoom
// =============================================================================

impl NoteApp {
    fn zoom_factor(&self) -> f32 {
        let normalized = self.zoom / MAX_ZOOM;

        10.0_f32.powf(-normalized * 2.0)
    }

    fn apply_zoom_delta(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta).clamp(MIN_ZOOM, MAX_ZOOM);
    }
}

// =============================================================================
// Visualizer
// =============================================================================

impl NoteApp {
    fn draw_visualizer(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 16.0, Color32::from_rgb(7, 7, 7));

        // ---------------------------------------------------------------------
        // Mouse wheel zoom.
        //
        // There is no drag-to-change-a/b interaction.
        // ---------------------------------------------------------------------

        let response = ui.interact(
            rect,
            ui.id().with("lissajous_visualizer"),
            egui::Sense::hover(),
        );

        if response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);

            if scroll.abs() > 0.001 {
                self.apply_zoom_delta(-scroll * 0.08);
            }
        }

        let center = rect.center();

        let zoom = self.zoom_factor();

        let base_radius = rect.width().min(rect.height()) * 0.34;

        let radius = (base_radius * zoom).clamp(8.0, base_radius * 100.0);

        self.draw_2d_grid(&painter, center, radius);

        self.draw_lissajous_curve(&painter, center, radius);

        self.draw_visualizer_labels(&painter, rect);
    }

    // -------------------------------------------------------------------------
    // 2D grid
    // -------------------------------------------------------------------------

    fn draw_2d_grid(&self, painter: &egui::Painter, center: Pos2, radius: f32) {
        let grid_stroke = Stroke::new(0.5, Color32::from_gray(28));

        let grid_size = 1.2_f32;

        let divisions = 8usize;

        // Vertical.
        for i in 0..=divisions {
            let f = -grid_size + (2.0 * grid_size) * i as f32 / divisions as f32;

            let x = center.x + f * radius;

            painter.line_segment(
                [
                    Pos2::new(x, center.y - radius),
                    Pos2::new(x, center.y + radius),
                ],
                grid_stroke,
            );
        }

        // Horizontal.
        for i in 0..=divisions {
            let f = -grid_size + (2.0 * grid_size) * i as f32 / divisions as f32;

            let y = center.y - f * radius;

            painter.line_segment(
                [
                    Pos2::new(center.x - radius, y),
                    Pos2::new(center.x + radius, y),
                ],
                grid_stroke,
            );
        }

        let axis_stroke = Stroke::new(1.0, Color32::from_gray(90));

        // X axis.
        painter.line_segment(
            [
                Pos2::new(center.x - radius, center.y),
                Pos2::new(center.x + radius, center.y),
            ],
            axis_stroke,
        );

        // Y axis.
        painter.line_segment(
            [
                Pos2::new(center.x, center.y - radius),
                Pos2::new(center.x, center.y + radius),
            ],
            axis_stroke,
        );

        painter.circle_filled(center, 3.0, Color32::WHITE);

        painter.text(
            Pos2::new(center.x + radius + 8.0, center.y),
            egui::Align2::LEFT_CENTER,
            "X",
            egui::FontId::monospace(12.0),
            Color32::WHITE,
        );

        painter.text(
            Pos2::new(center.x, center.y - radius - 8.0),
            egui::Align2::CENTER_BOTTOM,
            "Y",
            egui::FontId::monospace(12.0),
            Color32::WHITE,
        );
    }

    // -------------------------------------------------------------------------
    // Complete Lissajous trace
    // -------------------------------------------------------------------------

    fn draw_lissajous_curve(&self, painter: &egui::Painter, center: Pos2, radius: f32) {
        let period = self.mathematical_period();

        let mut previous = None::<Pos2>;

        for i in 0..=CURVE_STEPS {
            let t = period * i as f32 / CURVE_STEPS as f32;

            let (x, y) = self.lissajous_xy(t);

            let current = Pos2::new(center.x + x * radius, center.y - y * radius);

            if let Some(previous) = previous {
                painter.line_segment([previous, current], Stroke::new(1.5, Color32::WHITE));
            }

            previous = Some(current);
        }

        // ---------------------------------------------------------------------
        // Animated point
        // ---------------------------------------------------------------------

        let (x, y) = self.lissajous_xy(self.current_math_time());

        let current = Pos2::new(center.x + x * radius, center.y - y * radius);

        painter.circle_filled(current, 5.0, Color32::WHITE);

        painter.circle_stroke(current, 11.0, Stroke::new(1.0, Color32::WHITE));
    }

    // -------------------------------------------------------------------------
    // Labels
    // -------------------------------------------------------------------------

    fn draw_visualizer_labels(&self, painter: &egui::Painter, rect: Rect) {
        painter.text(
            Pos2::new(rect.left() + 20.0, rect.top() + 18.0),
            egui::Align2::LEFT_TOP,
            "LISSAJOUS",
            egui::FontId::monospace(11.0),
            Color32::WHITE,
        );

        // No a/b label at the top.
        //
        // Current X/Y/a/b data stays in the bottom status line.

        painter.text(
            Pos2::new(rect.left() + 20.0, rect.bottom() - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "X {:.3} Hz · Y {:.3} Hz · a/b {:.6}",
                self.x_hz,
                self.y_hz,
                self.visual_ratio(),
            ),
            egui::FontId::monospace(10.0),
            Color32::from_gray(150),
        );
    }
}

// =============================================================================
// Header
// =============================================================================

impl NoteApp {
    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.add_space(MAIN_MARGIN);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("LISSAJOUS").size(20.0).strong());

                    ui.add_space(8.0);

                    ui.label(egui::RichText::new("OSCILLATOR").size(11.0).monospace());
                });

                ui.add_space(2.0);

                ui.label(
                    egui::RichText::new("2D a/b geometry · synchronized X/Y audio").size(12.0),
                );
            });
        });
    }
}

// =============================================================================
// Main layout
// =============================================================================

impl NoteApp {
    fn draw_main_content(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width() - MAIN_MARGIN * 2.0;

        let available_height = ui.available_height();

        let panel_width = if available_width < 1000.0 {
            (available_width * 0.40).clamp(280.0, 370.0)
        } else {
            (available_width * 0.32).clamp(310.0, 410.0)
        };

        let visualizer_width = (available_width - panel_width - PANEL_GAP).max(260.0);

        let visualizer_height = available_height.max(300.0);

        ui.horizontal(|ui| {
            ui.add_space(MAIN_MARGIN);

            ui.vertical(|ui| {
                ui.horizontal_top(|ui| {
                    // -------------------------------------------------
                    // Left control panel
                    // -------------------------------------------------

                    ui.allocate_ui_with_layout(
                        Vec2::new(panel_width, visualizer_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("left_controls_scroll")
                                .auto_shrink([false, false])
                                .max_height(visualizer_height)
                                .show(ui, |ui| {
                                    self.draw_input_panel(ui, panel_width);
                                });
                        },
                    );

                    ui.add_space(PANEL_GAP);

                    // -------------------------------------------------
                    // Main visualizer
                    // -------------------------------------------------

                    let visualizer_rect = Rect::from_min_size(
                        ui.cursor().min,
                        Vec2::new(visualizer_width, visualizer_height),
                    );

                    ui.allocate_ui(Vec2::new(visualizer_width, visualizer_height), |ui| {
                        self.draw_visualizer(ui, visualizer_rect);
                    });
                });
            });

            ui.add_space(MAIN_MARGIN);
        });
    }

    fn draw_input_panel(&mut self, ui: &mut egui::Ui, width: f32) {
        ui.label(egui::RichText::new("FREQUENCY INPUT").strong().monospace());

        ui.add_space(8.0);

        self.draw_frequency_lock(ui);

        ui.add_space(10.0);

        self.draw_frequency_controls(ui);

        ui.add_space(10.0);

        self.draw_audio_ratio_input(ui);

        ui.add_space(10.0);

        self.draw_lissajous_ratio_control(ui);

        ui.add_space(10.0);

        self.draw_zoom_control(ui);

        ui.add_space(10.0);

        if self.lock_frequency {
            self.draw_note_presets(ui);
        }

        ui.add_space(12.0);

        self.draw_volume_control(ui);

        ui.add_space(12.0);

        self.draw_visual_rate_control(ui);

        ui.add_space(18.0);

        self.draw_play_button(ui, width);

        self.draw_audio_error(ui);
    }
}

// =============================================================================
// Frequency UI
// =============================================================================

impl NoteApp {
    fn draw_frequency_lock(&mut self, ui: &mut egui::Ui) {
        let previous = self.lock_frequency;

        ui.checkbox(
            &mut self.lock_frequency,
            egui::RichText::new("LOCK X/Y FREQUENCY").strong(),
        );

        if self.lock_frequency {
            ui.label(
                egui::RichText::new("ON → Y follows X")
                    .size(10.0)
                    .monospace(),
            );
        } else {
            ui.label(
                egui::RichText::new("OFF → X and Y independent")
                    .size(10.0)
                    .monospace(),
            );
        }

        if previous != self.lock_frequency {
            self.set_frequency_lock(self.lock_frequency);
        }
    }

    fn draw_frequency_controls(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("AUDIO FREQUENCIES")
                .strong()
                .monospace(),
        );

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("X").monospace().strong());

            let mut value = self.x_hz;

            if ui
                .add(
                    egui::DragValue::new(&mut value)
                        .speed(1.0)
                        .range(MIN_FREQUENCY_HZ..=MAX_FREQUENCY_HZ)
                        .suffix(" Hz"),
                )
                .changed()
            {
                self.set_x_frequency(value);
            }
        });

        if !self.lock_frequency {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Y").monospace().strong());

                let mut value = self.y_hz;

                if ui
                    .add(
                        egui::DragValue::new(&mut value)
                            .speed(1.0)
                            .range(MIN_FREQUENCY_HZ..=MAX_FREQUENCY_HZ)
                            .suffix(" Hz"),
                    )
                    .changed()
                {
                    self.y_hz = value.clamp(MIN_FREQUENCY_HZ, MAX_FREQUENCY_HZ);

                    self.sync_lissajous_ratio_from_audio();
                    self.update_frequency_input_from_audio();

                    self.restart_audio_if_playing();
                }
            });
        } else {
            ui.label(
                egui::RichText::new(format!("Y = {:.3} Hz", self.y_hz,))
                    .size(10.0)
                    .monospace(),
            );
        }
    }

    fn draw_audio_ratio_input(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("AUDIO X:Y RATIO").strong().monospace());

        ui.add_space(3.0);

        ui.label(egui::RichText::new("also controls mathematical a/b").size(10.0));

        ui.add_space(5.0);

        if ui
            .add(
                egui::TextEdit::singleline(&mut self.frequency_input)
                    .hint_text("1:1, 1:2, 1:3, 2:3")
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            self.recompute_ratio();
        }

        if !self.parsed_ok {
            ui.add_space(3.0);

            ui.label(egui::RichText::new("Invalid ratio.").size(10.0));
        }

        ui.label(
            egui::RichText::new(format!("X/Y = {:.6}", self.x_hz / self.y_hz,))
                .size(10.0)
                .monospace(),
        );
    }
}

// =============================================================================
// Lissajous ratio UI
// =============================================================================

impl NoteApp {
    fn draw_lissajous_ratio_control(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("LISSAJOUS a/b").strong().monospace());

        ui.add_space(3.0);

        ui.label(egui::RichText::new("X/Y ratio · manually adjustable").size(10.0));

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("0.01").monospace());

            let mut ratio = self.lissajous_ratio;

            if ui
                .add(
                    egui::Slider::new(&mut ratio, MIN_LISSAJOUS_RATIO..=MAX_LISSAJOUS_RATIO)
                        .show_value(false),
                )
                .changed()
            {
                self.set_lissajous_ratio_and_audio(ratio);
            }

            ui.label(egui::RichText::new("1").monospace());
        });

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("a/b").monospace().strong());

            let mut ratio = self.lissajous_ratio;

            if ui
                .add(
                    egui::DragValue::new(&mut ratio)
                        .speed(0.001)
                        .range(MIN_LISSAJOUS_RATIO..=MAX_LISSAJOUS_RATIO)
                        .fixed_decimals(4),
                )
                .changed()
            {
                self.set_lissajous_ratio_and_audio(ratio);
            }

            ui.label(
                egui::RichText::new(format!("X/Y = {:.6}", self.lissajous_ratio,)).monospace(),
            );
        });

        ui.label(
            egui::RichText::new("0.5 = X:Y 1:2 · 0.333 = X:Y 1:3")
                .size(10.0)
                .monospace(),
        );
    }
}

// =============================================================================
// Zoom
// =============================================================================

impl NoteApp {
    fn draw_zoom_control(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("ZOOM").strong().monospace());

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut value = self.zoom;

            if ui
                .add(egui::Slider::new(&mut value, MIN_ZOOM..=MAX_ZOOM).show_value(false))
                .changed()
            {
                self.zoom = value.clamp(MIN_ZOOM, MAX_ZOOM);
            }

            ui.label(format!("{:+.0}x", self.zoom,));
        });

        ui.label(
            egui::RichText::new("-100x close · 0x normal · +100x far")
                .size(10.0)
                .monospace(),
        );

        ui.label(egui::RichText::new("mouse wheel over plot also zooms").size(10.0));
    }
}

// =============================================================================
// Note presets
// =============================================================================

impl NoteApp {
    fn draw_note_presets(&mut self, ui: &mut egui::Ui) {
        let label = if self.note_presets_expanded {
            "NOTE PRESETS  -"
        } else {
            "NOTE PRESETS  +"
        };

        if ui
            .button(egui::RichText::new(label).monospace().strong())
            .clicked()
        {
            self.note_presets_expanded = !self.note_presets_expanded;
        }

        if !self.note_presets_expanded {
            return;
        }

        ui.add_space(6.0);

        ui.label(egui::RichText::new("Equal temperament · Do = 220 Hz").size(10.0));

        ui.add_space(6.0);

        let presets = [
            ("Do", 0_u32),
            ("Re", 2),
            ("Mi", 4),
            ("Fa", 5),
            ("Sol", 7),
            ("La", 9),
            ("Ti", 11),
            ("Do'", 12),
        ];

        ui.horizontal_wrapped(|ui| {
            for (name, semitones) in presets {
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new(name).monospace().strong())
                            .min_size(Vec2::new(52.0, 32.0)),
                    )
                    .clicked()
                {
                    self.apply_note_preset(semitones);
                }
            }
        });

        ui.add_space(4.0);

        ui.label(
            egui::RichText::new("Do  Re  Mi  Fa  Sol  La  Ti  Do'")
                .size(10.0)
                .monospace(),
        );
    }

    fn apply_note_preset(&mut self, semitones: u32) {
        let frequency = DEFAULT_FREQUENCY_HZ * 2.0_f64.powf(semitones as f64 / 12.0);

        self.x_hz = frequency.clamp(MIN_FREQUENCY_HZ, MAX_FREQUENCY_HZ);

        if self.lock_frequency {
            self.y_hz = self.x_hz;
        } else {
            self.y_hz = self.x_hz.clamp(MIN_FREQUENCY_HZ, MAX_FREQUENCY_HZ);
        }

        self.sync_lissajous_ratio_from_audio();
        self.update_frequency_input_from_audio();

        self.restart_audio_if_playing();
    }
}

// =============================================================================
// Volume
// =============================================================================

impl NoteApp {
    fn draw_volume_control(&mut self, ui: &mut egui::Ui) {
        let label = if self.volume_presets_expanded {
            "VOLUME PRESETS  -"
        } else {
            "VOLUME PRESETS  +"
        };

        if ui
            .button(egui::RichText::new(label).monospace().strong())
            .clicked()
        {
            self.volume_presets_expanded = !self.volume_presets_expanded;
        }

        if self.volume_presets_expanded {
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                for (label, value) in [
                    ("0%", 0.0),
                    ("25%", 0.25),
                    ("50%", 0.50),
                    ("75%", 0.75),
                    ("100%", 1.0),
                ] {
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(label).monospace().size(11.0))
                                .min_size(Vec2::new(48.0, 30.0)),
                        )
                        .clicked()
                    {
                        self.set_volume(value);
                    }
                }
            });

            ui.add_space(5.0);
        }

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("LEVEL").monospace());

            let mut value = self.volume;

            if ui
                .add(egui::Slider::new(&mut value, 0.0..=1.0).show_value(false))
                .changed()
            {
                self.set_volume(value);
            }

            ui.label(format!("{:.0}%", self.volume * 100.0,));
        });
    }
}

// =============================================================================
// Visual rate
// =============================================================================

impl NoteApp {
    fn visual_preset_button(&mut self, ui: &mut egui::Ui, preset: VisualRatePreset, label: &str) {
        let selected = self.visual_rate_preset == preset;

        let text = if selected {
            egui::RichText::new(label).color(Color32::BLACK).strong()
        } else {
            egui::RichText::new(label).color(Color32::WHITE).strong()
        };

        if ui.selectable_label(selected, text).clicked() {
            self.apply_visual_rate_preset(preset);
        }
    }

    fn draw_visual_rate_control(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("VISUAL RATE").strong().monospace());

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            self.visual_preset_button(ui, VisualRatePreset::VerySlow, "VERY SLOW");

            self.visual_preset_button(ui, VisualRatePreset::Slow, "SLOW");

            self.visual_preset_button(ui, VisualRatePreset::Normal, "NORMAL");
        });

        ui.add_space(6.0);

        let min_log = MIN_VISUAL_CYCLES_PER_SECOND.log10();

        let max_log = MAX_VISUAL_CYCLES_PER_SECOND.log10();

        let mut log_value = self
            .visual_cycles_per_second
            .clamp(MIN_VISUAL_CYCLES_PER_SECOND, MAX_VISUAL_CYCLES_PER_SECOND)
            .log10();

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("CYCLES / SECOND").monospace());

            if ui
                .add(egui::Slider::new(&mut log_value, min_log..=max_log).show_value(false))
                .changed()
            {
                let value = 10.0_f32.powf(log_value);

                self.set_visual_cycles_per_second(value);
            }

            let mut value = self.visual_cycles_per_second;

            if ui
                .add(
                    egui::DragValue::new(&mut value)
                        .speed(0.01)
                        .range(MIN_VISUAL_CYCLES_PER_SECOND..=MAX_VISUAL_CYCLES_PER_SECOND)
                        .fixed_decimals(3)
                        .suffix(" cps"),
                )
                .changed()
            {
                self.set_visual_cycles_per_second(value);
            }
        });

        ui.label(
            egui::RichText::new(format!("{:.3} cycles/sec", self.visual_cycles_per_second,))
                .size(11.0)
                .monospace(),
        );

        ui.label(egui::RichText::new("controls mathematical time").size(10.0));
    }
}

// =============================================================================
// Play
// =============================================================================

impl NoteApp {
    fn draw_play_button(&mut self, ui: &mut egui::Ui, width: f32) {
        let text = if self.playing {
            "STOP AUDIO"
        } else {
            "PLAY X + Y"
        };

        if ui
            .add_sized(
                [width, 46.0],
                egui::Button::new(egui::RichText::new(text).strong()),
            )
            .clicked()
        {
            self.toggle_play();
        }
    }

    fn draw_audio_error(&self, ui: &mut egui::Ui) {
        if let Some(error) = &self.audio_error {
            ui.add_space(8.0);

            ui.label(egui::RichText::new(error).size(11.0));
        }
    }
}

// =============================================================================
// Style
// =============================================================================

impl NoteApp {
    fn apply_style(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        visuals.override_text_color = Some(Color32::WHITE);

        visuals.panel_fill = Color32::from_rgb(8, 8, 8);

        visuals.window_fill = Color32::from_rgb(8, 8, 8);

        visuals.extreme_bg_color = Color32::from_rgb(4, 4, 4);

        visuals.faint_bg_color = Color32::from_rgb(14, 14, 14);

        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(12, 12, 12);

        visuals.widgets.inactive.bg_fill = Color32::from_rgb(16, 16, 16);

        visuals.widgets.hovered.bg_fill = Color32::from_rgb(28, 28, 28);

        visuals.widgets.active.bg_fill = Color32::from_rgb(36, 36, 36);

        // ---------------------------------------------------------------------
        // White widget outlines.
        // This includes checkbox outlines.
        // ---------------------------------------------------------------------

        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.5, Color32::WHITE);

        visuals.widgets.inactive.fg_stroke = Stroke::new(1.5, Color32::WHITE);

        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

        visuals.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);

        visuals.selection.bg_fill = Color32::WHITE;

        visuals.selection.stroke = Stroke::new(1.0, Color32::BLACK);

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();

        style.spacing.item_spacing = Vec2::new(8.0, 8.0);

        style.spacing.button_padding = Vec2::new(12.0, 8.0);

        ctx.set_style_of(egui::Theme::Dark, style);
    }
}

// =============================================================================
// Footer
// =============================================================================

impl NoteApp {
    fn draw_footer(&self, ui: &mut egui::Ui) {
        ui.add_space(8.0);

        ui.separator();

        ui.add_space(5.0);

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("CURVE").monospace().strong());

            ui.label("x(t)=sin(a t) · y(t)=sin(b t)");
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("DATA").monospace().strong());

            ui.label(format!(
                "X {:.3} Hz · Y {:.3} Hz · a/b {:.6}",
                self.x_hz,
                self.y_hz,
                self.visual_ratio(),
            ));
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("PERIOD").monospace().strong());

            ui.label(format!("{:.3}π", self.mathematical_period() / PI,));
        });
    }
}

// =============================================================================
// App
// =============================================================================

impl App for NoteApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        Self::apply_style(ui.ctx());

        self.update_animation();

        self.draw_header(ui);

        ui.add_space(12.0);

        self.draw_main_content(ui);

        self.draw_footer(ui);

        ui.ctx().request_repaint_after(Duration::from_millis(16));
    }
}

// =============================================================================
// Utility
// =============================================================================

fn pretty_num(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{}", value.round() as i64,)
    } else {
        format!("{:.6}", value,)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT])
            .with_title("Lissajous Oscillator"),

        ..Default::default()
    };

    eframe::run_native(
        "Lissajous Oscillator",
        options,
        Box::new(|_cc| Ok(Box::new(NoteApp::default()))),
    )
}
