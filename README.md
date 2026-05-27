# n-body-simulator

Yoshida Fourth Order integrator with poopy ui and SIMD support (how nice).

Note that the webassembly version uses scalar functions instead of SIMD.

TODO:
- [ ] Fix past line being foolish when simulating forwards


## Why are the releases shit? (Especially when that's how you get to the juicy simd)?
1. mans cba styl, icl 
2. The build features change depending on what your hardware supports, which will affect SIMD performance
3. I do not yet have a good way to build for the big 3 platforms and test them all (Work in progress)

## How can I stage webassembly for local testing?
I was lazy while setting up the wasm for the first time. It is a cross I bear myself. To be honest, the gigantitude of my laziness does inconvience me in ways that reduce the gigantitude of my laziness. Although this does not benefit me. Regardless, when I test staging I create the same environment the github action creates. I know that I should move all of that out to some seperate files which I can then use something like an xtask with for staging. But pleas refer to "Why no releases" section 1.

The html file should look like this:
```html
          <!DOCTYPE html>
          <html lang="en">
          <head>
              <meta charset="utf-8">
              <title>n-body gravs</title>
              <style>
                  html, body, canvas {
                      margin: 0px;
                      padding: 0px;
                      width: 100%;
                      height: 100%;
                      overflow: hidden;
                      position: absolute;
                      background: black;
                      z-index: 0;
                  }
              </style>
          </head>
          <body>
              <canvas id="glcanvas" tabindex='1'></canvas>
              <script src="assets/mq_js_bundle.js"></script>
              <script src="assets/quad-url.js"></script>
              <script src="assets/sapp_jsutils.js"></script>
              <script src="assets/butt-gravity.js"></script>
              <script>load("butt-gravity.wasm");</script>
          </body>
          </html>
```

and there should be a folder called assets with the 4 js files present in .github/workflows/assets in it.

Once this is assembled next to the wasm binary you must use a live-sever to view it. (I don't know the exact details but wasm will not load directly from the fs for security reasons).
```
cargo install basic-http-server
basic-http-server .
```

If you want to see an example of it all assembled together, the [gh-pages branch](https://github.com/illegitimate-egg/n-body-simulation/tree/gh-pages) on this repo contains a complete version.
