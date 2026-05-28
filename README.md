# n-body-simulator

Yoshida Fourth Order integrator with poopy ui and SIMD support (how nice).

Note that the webassembly version uses scalar functions instead of SIMD.
Note possible major performance increase if you compile locally.

TODO:
- [ ] Make it look less like shit
- [ ] Seperate main sources out into smaller files

## Features
- Yoshida 4th order integration for trying to maintain constant K_e efficiently
- Deterministic physics (time reversible!)
- SIMD acceleration (On supported targets)
- Prediction

