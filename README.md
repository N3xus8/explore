# **WGPU implementing a 3D Mirror effect** 🦀



*Date: Dec 2025*

## **What is it:** 

The goal is to build on the [**Learn WGPU**](https://sotrh.github.io/learn-wgpu/) tutorial and add some new functionalities and explore 🔥.

Here we explore the *Stencil buffer* commonly found in graphics backend.

The high level description: 

The aim is to use the *Stencil buffer* to mask the area where the mirror is placed and reflect the camera/world.

This is achieved with 4 render pass basically:

1. Pass to mask the mirror refletive area.
2. Pass to reflect the 3D camera in the mirror
3. Pass to render the rest of the world
4. Pass to render the mirror surface with texture/tint/blending (optional)

The cool thing about [WGPU](https://github.com/gfx-rs/wgpu) ([WEBGPU](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API)) is that the same code works for both desktops and web browsers (WASM).
It requires a heavy setup upfront but later on it is almost painless(not 100% though).

I used **Rust** *1.92 nightly*.

Main crates: 📦

* **WGPU** version 0.30.
* **Winit** version 27.0.1

Note: I use [Caddy](https://caddyserver.com/) to serve the static files with [CORS.](https://developer.mozilla.org/fr/docs/Web/HTTP/Guides/CORS)

Developped on [Rocky Linux 10.1](https://rockylinux.org) tested on Windows11.

How to run it:

For desktop:
```bash
> cargo run
```

For web browser:

```bash
> trunk serve 
```
and I use **Caddy** to serve static files with CORS.

So on *Linux*:

```bash
> sudo caddy run --config Caddyfile --adapter caddyfile 
```

### **Check this out:** 



[![Watch the video](mirror.gif)]