# Timestep Tests

Visually demonstrate the difference between timestep strategies.

A Lorenz Attractor is used as a sensitive chaotic system to test determinism and simulation accuracy.

Visual fidelity is tested with high-frequency player-controlled motion.

## Sources

- https://gafferongames.com/post/fix_your_timestep/
- https://cbournhonesque.github.io/lightyear/book/concepts/advanced_replication/visual_interpolation.html

## Types of Timestep

From most-to-least framerate dependent:

- No delta time:
	- Step simulation once every render frame with a constant delta-time value
- Variable delta time:
	- Step simulation once every render frame using `Time::elapsed` as the delta-time value
- Semi-fixed timestep:
	- Step simulation 1 or more times every render frame using `min(Time::elapsed, constant)` as the delta-time value
- Fixed timestep:
	- Step simulation 0 or more times every render frame with a constant delta-time value

## Types of Visual Smoothing

- None
- Interpolate with previous value
- Extrapolate to future value

## Safety measures for lag

- None
- Cap sim rate (slow down)

## Frequency configurations

Render framerate in FPS, simulation update rate in Hz

- Sim much faster (60 FPS / 128 Hz)
- Sim slightly faster (60 FPS / 64 Hz)
- Sim slightly slower (60 FPS / 56 Hz)
- Sim much slower (60 FPS / 24 Hz)

## Render presentation modes

| Presentation Mode | Input Latency | Screen Tearing | Framerate capped by Refresh Rate |
|-|-|-|-|
| Fifo ("vsync") | Yes | No | Yes |
| FifoRelaxed ("adaptive vsync") | Yes | Yes | Yes |
| Immediate | No | Yes | No |
| Mailbox ("fast vsync") | No | No | No |

## Behaviour with a laggy simulation

| Type of Timestep | Behaviour | Header |
|--------|--------|--------|
| No delta time | Cell | Cell |
| Variable delta time | Cell | Cell |
| Semi-fixed timestep | Cell | Cell | 
| Fixed timestep | Cell | Cell | 

## Results table

| Type of Timestep | Determinism | Visual accuracy |
|--------|--------|--------|
| No delta time | Cell | Cell |
| Variable delta time | Cell | Cell |
| Semi-fixed timestep | Cell | Cell | 
| Fixed timestep | Cell | Cell | 
