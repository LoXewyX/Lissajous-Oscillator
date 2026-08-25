# Lissajous Oscillator

A native Rust desktop app for exploring Lissajous curves with synchronized X/Y sine waves.

The app connects three things:

* **Audio** — X and Y sine-wave frequencies
* **Geometry** — the Lissajous ratio `a/b`
* **Animation** — how fast the curve is traced visually

The relationship between audio and geometry is:

$$
\frac{a}{b} = \frac{X}{Y}
$$

## Features

* Real-time X/Y sine-wave audio
* Lissajous curve visualizer
* Synchronized X/Y and `a/b` controls
* Expression support for frequency input
* Musical note presets
* Adjustable animation speed
* Logarithmic 2D zoom
* Mouse-wheel zoom
* Minimal dark UI
* Animated point following the rendered curve

## Lissajous curves

The curve is defined as:

$$
x(t) = \sin(at)
$$

$$
y(t) = \sin(bt)
$$

For example:

|      X |      Y | `a/b` |
| -----: | -----: | ----: |
| 220 Hz | 220 Hz |   1:1 |
| 220 Hz | 440 Hz |   1:2 |
| 220 Hz | 660 Hz |   1:3 |
| 440 Hz | 660 Hz |   2:3 |

For a rational ratio

$$
\frac{a}{b} = \frac{p}{q}
$$

the complete curve is rendered over:

$$
T = 2\pi q
$$

This makes sure figures such as `1:2` and `1:3` are rendered as complete curves.

### Near 1:1

An exact `1:1` ratio produces a straight line.

Very small differences can otherwise be difficult to see, so the visualizer adds a small phase offset when the ratio is close to `1`:

$$
y(t) = \sin(bt + \phi)
$$

This only affects the visualization. The audio frequencies remain unchanged.

The exact closure period is also not used for near-equal frequencies. Instead, the visualizer uses a normalized local cycle so small differences remain practical to inspect.

## Audio

Two sine-wave oscillators are generated:

* **X** controls the first oscillator.
* **Y** controls the second oscillator.
* Both are mixed and sent to the default audio output.

Example:

```text
X = 220 Hz
Y = 440 Hz
```

gives:

$$
a/b = 220/440 = 0.5
$$

Changing `a/b` updates Y while keeping X as the reference:

$$
Y = \frac{X}{a/b}
$$

If audio is playing, changing a frequency restarts the current mix.

## Frequency controls

X and Y accept frequencies from:

```text
20 Hz .. 20,000 Hz
```

The fields also accept `meval` expressions:

```text
220
220 * 2
440 / 2
1000 Hz
```

### X/Y lock

When enabled, Y follows X according to the selected ratio.

When disabled, X and Y can be edited independently. Changing either frequency updates `a/b`.

### X:Y ratio

Ratios can be entered directly:

```text
1:1
1:2
1:3
2:3
```

Expressions are also supported:

```text
1:(2+1)
```

which gives `1:3`.

## a/b

The `LISSAJOUS a/b` control has a slider and numeric input.

Current range:

```text
0.01 .. 1.0
```

Changing `a/b` recalculates Y using X as the reference.

Changing X or Y recalculates `a/b`.

The curve itself is not draggable.

### Curve generation

The curve is evaluated parametrically:

$$
x(t) = \sin(at)
$$

$$
y(t) = \sin(bt + \phi)
$$

`φ` is normally zero and is only used for the near-`1:1` visualization case.

### Period calculation

`mathematical_period()` finds the interval required for a complete closed figure for normal rational ratios.

For:

$$
a/b = p/q
$$

the period is:

$$
T = 2\pi q
$$

Near `1:1`, a normalized local cycle is used instead.

### Rendering

The renderer draws:

* Grid
* X/Y axes
* Lissajous curve
* Animated point
* Frequency and ratio information

```

## Limitations

### a/b range

The current UI supports:

```text
0.01 .. 1.0
```

so it mainly covers cases where `Y >= X`.

A future version could support the full positive range:

```text
0 < a/b < ∞
```

## Possible improvements

* Phase control
* Full `a/b` range
* Independent `a` and `b` controls
* Curve trails
* Screenshot export
* Waveform preview
* MIDI input
* Preset save/load
* Custom reference frequency
* Optional stereo output
* GPU rendering

## License

MIT License

See [LICENSE](LICENSE).
