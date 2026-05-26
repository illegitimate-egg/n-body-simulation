# n-body-simulator

Yoshida Fourth Order integrator with poopy ui and SIMD support (how nice).

Note that the webassembly version uses scalar functions instead of SIMD.

TODO:
- [ ] Make simulation rate independent of rendering rate

## Why no releases (Especially when that's how you get to the juicy simd)?
1. mans cba styl, icl 
2. The build features change depending on what your hardware supports, which will affect SIMD performance
3. I do not yet have a good way to build for the big 3 platforms and test them all (Work in progress)
