# Timestep Tests

Visually demonstrate the difference between timestep strategies.

Features several different example simulations, as well as a bunch of configuration options for experimentation.

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
- Interpolate with previous value (i.e. `lerp` between positions by `Time::overstep_fraction`)
- Extrapolate to future value (Same as interpolation, but with the `t` parameter in the range `1..2` instead of `0..1`)

For more information, see:
- https://gafferongames.com/post/fix_your_timestep/
- https://cbournhonesque.github.io/lightyear/book/concepts/advanced_replication/visual_interpolation.html

## Safety measures for lag (Unimplemented)

Semi-fixed and fixed timesteps can "death spiral" if the duration of an update is longer than the configured delta time. This can be mitigated in a couple of ways:

- No mitigation, just let it crash and burn
- Cap the maximum number of updates per frame. Will manifest as the simulation running slower than real-time
- Dynamically increase the configured delta time until the simulation is able to catch back up. Larger delta times may destabilize the simulation (Effectively negating the primary benefit of (semi-)fixed timesteps)

## Render presentation modes

The update rate of the application is not inherently tied to the refresh rate of the monitor. A new frame being rendered mid-refresh causes "screen tearing", which generally looks terrible. Different presentation modes exist to remedy this.

| Presentation Mode | Input Latency | Screen Tearing | Framerate capped by Refresh Rate |
|-|-|-|-|
| Fifo ("vsync") | Yes | No | Yes |
| FifoRelaxed ("adaptive vsync") | Yes | Yes | Yes |
| Immediate | No | Yes | No |
| Mailbox ("fast vsync") | No | No | No |

For more information, see [`wgpu::PresentMode`](https://docs.rs/wgpu/latest/wgpu/enum.PresentMode.html)
