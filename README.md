# Lissajous Oscillator

The application combines:

- Real-time X/Y audio generation
- A mathematically driven 2D Lissajous visualizer
- Synchronized `a/b` and X/Y frequency ratios
- Adjustable visual animation speed
- 2D logarithmic zoom
- Musical note presets
- A minimal dark interface built with `egui` and `eframe`

## Features

### Lissajous curve

The visualizer is based on the standard parametric form:

\[
x(t) = \sin(at)
\]

\[
y(t) = \sin(bt)
\]

The application synchronizes the mathematical ratio with the audio frequencies:

\[
\frac{a}{b} = \frac{X}{Y}
\]

Examples:

| X | Y | a/b | Shape |
|---:|---:|---:|---|
| 220 Hz | 220 Hz | 1.000000 | Straight diagonal |
| 220 Hz | 220.5 Hz | ≈ 0.997732 | Narrow ellipse |
| 220 Hz | 221 Hz | ≈ 0.995475 | Wider ellipse |
| 220 Hz | 440 Hz | 0.500000 | Complete 1:2 Lissajous figure |
| 220 Hz | 660 Hz | ≈ 0.333333 | Complete 1:3 figure |

The exact `1:1` case remains a straight line. Ratios close to `1:1` receive a small visual phase enhancement so very small frequency differences are visible as ellipses rather than becoming visually indistinguishable from a line.

### Complete curve periods

The renderer calculates a useful mathematical period before drawing the curve.

For rational ratios:

\[
\frac{a}{b} = \frac{p}{q}
\]

the complete common period is:

\[
T = 2\pi q
\]

Examples:

- `1:1` → `2π`
- `1:2` → `4π`
- `1:3` → `6π`
- `2:3` → `6π`
- `1:4` → `8π`

This prevents the common problem where a `1:2` figure is rendered as only half of its ribbon.

## Audio

The application generates two independent sine-wave oscillators:

- **X frequency** controls the first oscillator.
- **Y frequency** controls the second oscillator.
- The two oscillators are mixed and sent to the default audio output.

The relationship between audio and the mathematical curve is:

```text
a/b = X/Y
```

Changing either side keeps the other representation synchronized.

For example:

```text
X = 220 Hz
Y = 440 Hz

a/b = 220 / 440
    = 0.5
```

Changing `a/b` performs the inverse operation. X remains the reference frequency and Y is recalculated:

```text
Y = X / (a/b)
```

So:

```text
X = 220 Hz
a/b = 0.5

Y = 440 Hz
```

If audio is currently playing, frequency changes restart the current audio mix automatically.

## Frequency controls

### X and Y frequency

Frequencies are limited to:

```text
20 Hz .. 20,000 Hz
```

The application accepts mathematical expressions through the frequency fields, including expressions supported by `meval`.

Examples:

```text
220
220 * 2
440 / 2
1000 Hz
```

### Lock X/Y frequency

When enabled:

```text
Y follows X
```

Changing X changes Y according to the currently selected audio ratio.

When disabled:

```text
X and Y are independently editable
```

Changing either frequency automatically updates `a/b`.

### X:Y ratio input

The audio ratio field accepts values such as:

```text
1:1
1:2
1:3
2:3
```

It can also accept expressions.

For example:

```text
1:(2+1)
```

results in a `1:3` relationship.

## a/b control

The `LISSAJOUS a/b` control contains:

- A slider for continuous adjustment
- A numeric field for precise manual input

The current range is:

```text
0.01 .. 1.0
```

The numeric field can be edited directly with fine increments.

Changing `a/b` changes the Y frequency while keeping X as the reference.

Changing X or Y changes `a/b` automatically.

There is **no drag-to-change-a/b interaction** in the plot itself.

## Visual animation

The animated point travels along the currently rendered curve.

Visual speed is independent from audio pitch.

Available presets:

- `VERY SLOW` — `0.10 cycles/sec`
- `SLOW` — `0.50 cycles/sec`
- `NORMAL` — `1.00 cycles/sec`

There is also a logarithmic continuous control spanning:

```text
0.01 .. 2.0 cycles/sec
```

The animation uses a normalized time value, so the moving point always remains synchronized with the exact curve interval currently being rendered.

## Zoom

The plot has ordinary 2D zoom.

The slider range is:

```text
-100x .. +100x
```

with:

```text
0x    = normal
-100x = very close
+100x = very far
```

Zoom is logarithmic so the large range remains usable.

The mouse wheel can also zoom while the pointer is over the plot.

Zoom changes only the 2D scale. It does not modify:

- X frequency
- Y frequency
- `a/b`
- visual rate

## Curve rendering

The curve is sampled with:

```rust
const CURVE_STEPS: usize = 6000;
```

That means the selected mathematical period is approximated with 6000 line segments.

This is a sampling resolution, not a frequency or cycle count.

The trace does not use an artificial segment-length cutoff, so the complete mathematical interval is connected continuously.

## Musical note presets

The application includes equal-temperament note presets using:

```text
Do = 220 Hz
```

Available notes:

```text
Do
Re
Mi
Fa
Sol
La
Ti
Do'
```

Selecting a preset updates X and then synchronizes the corresponding ratio and Y frequency.

## User interface

The application uses a dark, monochrome interface with:

- White typography
- White curve rendering
- Compact monospace labels
- Responsive two-column layout
- Main content margins
- Collapsible note presets
- Volume presets
- A synchronized frequency/ratio display on the visualizer

The main visualizer displays:

```text
X frequency
Y frequency
a/b
```

directly in its lower status line.

## Build requirements

You need:

- Rust
- Cargo
- A working system audio output

The project uses:

- `eframe`
- `egui`
- `rodio`
- `meval`
- `anyhow`

A typical `Cargo.toml` dependency section is:

```toml
[dependencies]
anyhow = "1"
eframe = "0.36"
meval = "0.2"
rodio = "0.21"
```

Adjust dependency versions to match the versions used by the project.

## Running

From the project directory:

```bash
cargo run
```

For an optimized build:

```bash
cargo run --release
```

Or build the executable:

```bash
cargo build --release
```

## Architecture

The application is intentionally separated into several conceptual layers.

### Audio

`AudioState` is responsible for:

- Opening the default output device
- Creating the X sine wave
- Creating the Y sine wave
- Mixing the sources
- Controlling playback volume
- Stopping and restarting playback

### Mathematical state

`NoteApp` stores:

```text
x_hz
y_hz
lissajous_ratio
```

with the synchronization rule:

```text
a/b = X/Y
```

### Curve generation

The curve is evaluated parametrically:

```rust
x(t) = sin(a * t)
y(t) = sin(b * t + phase)
```

The near-`1:1` visual enhancement uses a small phase adjustment so tiny differences such as:

```text
220 / 220.5
```

remain visible.

Exactly `1:1` uses zero phase and remains a straight line.

### Period calculation

`mathematical_period()` determines the interval needed to render a complete closed figure for normal rational ratios.

Near `1:1`, the application intentionally uses a normalized local cycle instead of requiring the extremely long exact beat period of a near-equal but non-identical frequency pair.

### Rendering

Rendering is pure 2D:

- X/Y axes
- Grid
- Lissajous trace
- Animated point
- Status data
- 2D zoom

There is no 3D coordinate system, Z axis, camera rotation, or rotation-speed control.

## Design principles

The application keeps the following concepts separate:

```text
Audio frequency
    ↓
X / Y

Mathematical geometry
    ↓
a / b = X / Y

Animation
    ↓
visual cycles per second

View
    ↓
2D zoom
```

This separation makes it possible to change visual speed or zoom without changing the pitch relationship.

## Known limitation

The current `a/b` control is normalized to:

```text
0.01 .. 1.0
```

This is convenient for exploring cases where `Y >= X`.

A future version could allow the full positive ratio range:

```text
0 < a/b < ∞
```

and automatically normalize equivalent curves such as `2:1` versus `1:2` while preserving their orientation.

## Possible future improvements

Potential extensions include:

- Phase control from `0°` to `360°`
- Full unrestricted `a/b` range
- Frequency-independent mathematical `a` and `b` controls
- Curve persistence / trail mode
- Screenshot export
- Audio waveform preview
- MIDI note input
- Preset save/load
- Custom note reference frequency
- Optional dual-frequency stereo output
- GPU rendering for extremely dense curves

## License

MIT License

See [LICENSE](LICENSE).
```
