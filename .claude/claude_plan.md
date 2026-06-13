# Plan: Add Zoom and Pan to the Live Visualization

## Context

The live visualization is a browser page (D3.js + SVG) that shows a behavior tree updating in real time. Right now the SVG just auto-fits the whole tree into the window. Big trees become unreadable, and the user has no way to look closer at one part. We want to let users **zoom in/out with the mouse wheel** and **pan by clicking and dragging**.

All the code lives in one file: [bonsai/src/index.html](bonsai/src/index.html). No Rust changes are needed.

## Approach

Use D3's built-in zoom behavior (`d3.zoom()`). D3 v7 is already loaded, so we don't add any new dependencies. It handles wheel-zoom, drag-pan, and touch gestures out of the box.

The plan:

1. **Wrap content in a stable group.**
   Today [renderTree](bonsai/src/index.html#L181) appends a fresh `<g>` every call. Instead, create the group **once** outside `renderTree` and reuse it. This way the zoom transform stays put when the tree is re-rendered (e.g. on WebSocket reconnect).

2. **Attach `d3.zoom()` to the SVG.**
   - Listen for zoom events and apply `transform` to the wrapper group.
   - Limit scale with `.scaleExtent([0.1, 8])` so users can't zoom into oblivion.
   - Store the zoom behavior on a module variable so we can call it later (for the reset button).

3. **Keep auto-fit behavior on first render.**
   Compute the existing viewBox bounds as today, but instead of setting `viewBox` directly, compute an initial transform (translate + scale) that fits the tree, and apply it via `svg.call(zoom.transform, initialTransform)`. Only do this the **first** time a tree is rendered, not on every reconnect — otherwise we'd snap the user's view back.

4. **Add a small "Reset view" button.**
   Place it next to the existing tick counter / legend. Clicking it re-runs the fit-to-view transform.

5. **Disable zoom on double-click.**
   D3's default double-click-to-zoom is annoying for this UI. Disable with `.on('dblclick.zoom', null)`.

## Files to Change

- [bonsai/src/index.html](bonsai/src/index.html) — only file touched.
  - Around line 200: pull the `g` out of `renderTree` into a one-time setup block.
  - Around line 197: replace direct `viewBox` setting with the initial zoom transform.
  - Add a button in the top bar HTML.
  - Add zoom setup + reset handler in the script section.

## Verification

1. Run the example:
   ```
   cargo run --bin visualizer_smoke
   ```
2. Open `http://127.0.0.1:8910/` in a browser.
3. Check:
   - Mouse wheel zooms in and out, centered on the cursor.
   - Click + drag pans the tree.
   - Tree ticks (color changes) keep working — zoom level does not reset on each tick.
   - "Reset view" button snaps back to fit-to-screen.
   - Double-click does not trigger zoom.
4. Refresh the page mid-run: tree re-renders and view still fits correctly.
