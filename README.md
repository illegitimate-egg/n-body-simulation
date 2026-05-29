# Polaris: Efficient 𝑛-body Simulator

Polaris is a deterministic 2D abstraction of gravitational systems. 
Yoshida Fourth Order integrator with a UI and SIMD support (how nice).

> [!NOTE]
> The webassembly version uses scalar functions instead of SIMD, this is because webassembly SIMD support is incomplete.

> [!NOTE]
> Note possible major performance increase if you compile locally, as arch specific SIMD instructions may become available.

## Features
- [Yoshida 4th order integration](https://en.wikipedia.org/wiki/Leapfrog_integration#4th_order_Yoshida_integrator) for trying to maintain constant $E_k$ efficiently and [symplectically](https://en.wikipedia.org/wiki/Symplectic_integrator).
- [Deterministic](https://en.wikipedia.org/wiki/Deterministic_system) physics.
- Time acceleration (including reversal).
- [SIMD](https://en.wikipedia.org/wiki/Single_instruction,_multiple_data) acceleration (On supported targets).
- Prediction of the future and of [the](https://en.wikipedia.org/wiki/The) past.

## Usage
Fire up the webassembly version, or download a release. If you want to maximize performance, compiling from source is also and option. Since this project is built with rust cargo it couldn't be easier. Head over to [rustup.rs](https://rustup.rs/) to get a copy of the **nightly** toolchain and then enter `cargo run --release`.

Once you're in the program, you can set up a system by dragging points around, or right clicking them to change their velocity or precisely change their position. By default you should be seeing a solution to the three body problem, found in [this paper](https://arxiv.org/abs/math/0511219)

## Releases
Releases are automatically built when a new tag is pushed. See the github workflow file for details.

## See also
- The [todo list](TODO.md)
- The [build workflow](.github/workflows/release.yml)

