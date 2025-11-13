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
	- Simulation rate is proportional to framerate
- Variable delta time:
	- Step simulation once every render frame using `Time::elapsed` as the delta-time value
	- Non-deterministic
	- Lag spikes or low framerates result in large delta time values, which can destabilize the simulation
- Semi-fixed timestep:
	- Step simulation 1 or more times every render frame using `min(Time::elapsed, constant)` as the delta-time value
	- Non-deterministic
	- Handles lag spikes and low framerates well
	- Can "death spiral" if the simulation itself is too laggy
- Fixed timestep:
	- Step simulation 0 or more times every render frame with a constant delta-time value
	- Handles lag spikes and low framerates well
	- Can "death spiral" if the simulation itself is too laggy
	- Causes noticable visual stuttering

## Types of Visual Smoothing

Fixed timestep causes visual issues, which can be mitigated in a couple of ways:

- None
- Interpolate with previous value
- Interpolate to future value (with previous value but by 1..2 instead of 0..1)
- Extrapolate to future value (how?)

## Safety measures for lag

Semi-fixed and fixed timesteps can "death spiral" if the duration of an update is longer than the configured delta time. This can be mitigated in a couple of ways:

- No mitigation, just let it crash and burn
- Cap the maximum number of updates per frame. Will manifest as the simulation running slower than real-time
- Dynamically increase the configured delta time until the simulation is able to catch back up. Larger delta times may destabilize the simulation (Effectively negating the primary benefit of (semi-)fixed timesteps)

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
