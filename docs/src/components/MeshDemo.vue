<script setup>
// Animated hero illustration, drawn entirely on a <canvas>.
//
// Story (16s loop), same beats as the original DOM/keyframe version:
//   Beat A — a zoomed macOS desktop pops out of macbook, the user drags a
//   screenshot selection (marching ants + crosshair + flash), and the
//   captured thumbnail flies through the monitor to every peer.
//   Beat B — a debian desktop pops out with a Claude Code session; the
//   typed-for authorization code is copied and fans out the other way.
// The center window is the `ssh-clipboard monitor` TUI logging it all.
import { onBeforeUnmount, onMounted, ref } from 'vue'

const wrap = ref(null)
const cv = ref(null)

// Page-blending grays match the @theme block in src/style.css; the
// scene colors below them are the illustration's own palette.
const C = {
  line: '#262626',
  lineBright: '#3d3d3d',
  bright: '#f2f2f2',
  dim: '#8b8b8b',
  faint: '#595959',
  mint: '#4ee585',
  tuiBg: '#13151b',
  accent: '#a78bfa',
  cyan: '#22d3ee',
  green: '#34d399',
  yellow: '#fbbf24',
  muted: '#64748b',
  soft: '#cbd5e1',
  tuiPanel: '#3d3d3d', // same neutral gray as lineBright: one border color
  chipBg: '#0f1620',
  bg: '#000000',
}

const MONO = "ui-monospace, 'SF Mono', 'JetBrains Mono', Menlo, Consolas, monospace"
const LOOP = 16000

// ── timeline (fractions of the loop, matching the old keyframes) ──
const clamp01 = (v) => Math.max(0, Math.min(1, v))
const seg = (t, a, b) => clamp01((t - a) / (b - a))
// snappy settle, like the old cubic-bezier(0.22,1,0.36,1) desk-pop
const outQuint = (p) => 1 - (1 - p) ** 5
const inCubic = (p) => p ** 3
// hard attack / hold / hard release, like the old stepped glow keyframes
const trap = (p) => (p <= 0 || p >= 1 ? 0 : Math.min(1, p / 0.12, (1 - p) / 0.18))

// Every effect follows its cause within ~0.3s; the tail after the code is
// accepted is the deliberate resting state before the loop wraps.
const T = {
  // beat A — screenshot on the mac, fans out to everyone.
  // The mac desktop is already open at t=0 (it opened at deskD last loop,
  // showing the accepted code) and closes after the capture.
  sel: [0.04, 0.105], // selection marquee + crosshair drag
  flash: [0.115, 0.132],
  deskAClose: [0.132, 0.162], // the pane dismisses fully…
  flyA: [0.168, 0.205], // …then the shot leaves the node along its wire
  sentTotal: 0.267, // 3 × 8.8 MiB, once all three sends complete
  fanA: [0.215, 0.265], // monitor → all three peers, together
  recvA: [0.265, 0.32],
  // beat A2 — debian uses the image, window closes, that's it
  deskB: [0.285, 0.32, 0.445, 0.475],
  // beat B — fedora types its own request, gets a response with a code
  // (clear air after debian's pane is fully gone)
  deskC: [0.495, 0.53, 0.716, 0.746],
  hlC: [0.676, 0.706], // copy highlight sweep, then the pane dismisses
  flyB: [0.75, 0.79], // code rides fedora's wire back to the monitor
  recvTotal: 0.795,
  fanB: [0.805, 0.855], // monitor → debian, mini, macbook, together
  recvB: [0.855, 0.91],
  // loop closure — the code lands, the mac desktop opens, the pointer
  // clicks into the field, the paste accepts; that end state is frame one
  deskD: [0.875, 0.908],
  macPtr: [0.9, 0.945], // pointer travel to the authorization field
  codeFill: 0.948,
  fade: [0.955, 0.99],
}

const IMG_PV = '<Apple PNG past…'
const TXT_PV = '01923091283'

// Rows append downward as the story advances.
const ROWS = [
  // each row appears when its payload reaches the monitor pane
  { at: 0.205, time: '41.204', flow: '◆ copied here', fc: C.accent, pv: IMG_PV, size: '8.8 MiB', fm: '6' },
  { at: 0.218, time: '41.530', flow: '→ fedora', fc: C.cyan, pv: IMG_PV, size: '8.8 MiB', fm: '6' },
  { at: 0.224, time: '41.530', flow: '→ debian', fc: C.cyan, pv: IMG_PV, size: '8.8 MiB', fm: '6' },
  { at: 0.23, time: '41.530', flow: '→ mini', fc: C.cyan, pv: IMG_PV, size: '8.8 MiB', fm: '6' },
  { at: 0.792, time: '47.214', flow: '← fedora', fc: C.green, pv: TXT_PV, size: '11 B', fm: '1' },
  { at: 0.808, time: '47.215', flow: '→ debian', fc: C.cyan, pv: TXT_PV, size: '11 B', fm: '1' },
  { at: 0.814, time: '47.215', flow: '→ mini', fc: C.cyan, pv: TXT_PV, size: '11 B', fm: '1' },
  { at: 0.82, time: '47.215', flow: '→ macbook', fc: C.cyan, pv: TXT_PV, size: '11 B', fm: '1' },
]

// ── desktop scenes (fixed 440×292 local coordinate space) ─────────
const SCENE_W = 440
const SCENE_H = 292
// screenshot capture region, in scene coords — frames the sunset photo
const CAP = { x: 252, y: 102, w: 116, h: 90 }

function rr(ctx, x, y, w, h, r) {
  ctx.beginPath()
  ctx.roundRect(x, y, w, h, r)
}

// classic arrow pointer, drawn at its hotspot
function drawPointer(ctx, x, y) {
  ctx.save()
  ctx.translate(x, y)
  ctx.beginPath()
  ctx.moveTo(0, 0)
  ctx.lineTo(0, 13)
  ctx.lineTo(3, 10.5)
  ctx.lineTo(5.2, 15)
  ctx.lineTo(7.2, 14)
  ctx.lineTo(5, 9.8)
  ctx.lineTo(9, 9.5)
  ctx.closePath()
  ctx.fillStyle = '#ffffff'
  ctx.fill()
  ctx.strokeStyle = 'rgba(0,0,0,0.8)'
  ctx.lineWidth = 1
  ctx.stroke()
  ctx.restore()
}

function drawMacBase(ctx) {
  // menubar
  ctx.fillStyle = 'rgba(10,14,20,0.92)'
  ctx.fillRect(0, 0, SCENE_W, 22)
  ctx.fillStyle = '#aab6c4'
  ctx.beginPath()
  ctx.arc(13, 11, 4.5, 0, Math.PI * 2)
  ctx.fill()
  ctx.fillStyle = 'rgba(255,255,255,0.28)'
  for (const [mx, mw] of [[26, 29], [61, 18], [85, 22], [396, 13]]) {
    rr(ctx, mx, 8.5, mw, 4.5, 2)
    ctx.fill()
  }
  ctx.fillStyle = 'rgba(255,255,255,0.6)'
  ctx.font = `9.5px ${MONO}`
  ctx.fillText('18:31', 414, 14.5)
  // wallpaper
  const wp = ctx.createLinearGradient(0, 22, SCENE_W, SCENE_H)
  wp.addColorStop(0, '#1c2c52')
  wp.addColorStop(0.55, '#173a54')
  wp.addColorStop(1, '#0f4a49')
  ctx.fillStyle = wp
  ctx.fillRect(0, 22, SCENE_W, SCENE_H - 22)
  const glow = ctx.createRadialGradient(410, 0, 0, 410, 0, 110)
  glow.addColorStop(0, 'rgba(110,231,183,0.18)')
  glow.addColorStop(1, 'rgba(110,231,183,0)')
  ctx.fillStyle = glow
  ctx.fillRect(300, 22, 140, 120)
  // sunset photo (this is what gets captured)
  const px = 270
  const py = 118
  ctx.save()
  rr(ctx, px, py, 78, 58, 4)
  ctx.clip()
  const sky = ctx.createLinearGradient(0, py, 0, py + 58)
  sky.addColorStop(0, '#2c4a7c')
  sky.addColorStop(0.62, '#7a4a6e')
  sky.addColorStop(1, '#d98a5f')
  ctx.fillStyle = sky
  ctx.fillRect(px, py, 78, 58)
  ctx.fillStyle = '#ffd9a0'
  ctx.shadowColor = 'rgba(255,217,160,0.8)'
  ctx.shadowBlur = 8
  ctx.beginPath()
  ctx.arc(px + 58, py + 15, 6, 0, Math.PI * 2)
  ctx.fill()
  ctx.shadowBlur = 0
  ctx.fillStyle = '#1a2437'
  ctx.beginPath()
  const mtn = [[0, 58], [0, 32], [20, 10.5], [37, 35], [55, 16], [78, 40], [78, 58]]
  mtn.forEach(([mx, my], i) => (i ? ctx.lineTo(px + mx, py + my) : ctx.moveTo(px + mx, py + my)))
  ctx.closePath()
  ctx.fill()
  ctx.restore()
  ctx.strokeStyle = 'rgba(255,255,255,0.3)'
  ctx.lineWidth = 1.5
  rr(ctx, px, py, 78, 58, 4)
  ctx.stroke()
  // dock
  rr(ctx, SCENE_W / 2 - 45, SCENE_H - 22, 90, 16, 5)
  ctx.fillStyle = 'rgba(255,255,255,0.1)'
  ctx.fill()
  const apps = ['#5aa7f7', '#4ee585', '#f7b955', '#d78bea', '#8b97a5']
  apps.forEach((ac, i) => {
    ctx.fillStyle = ac
    rr(ctx, SCENE_W / 2 - 39 + i * 16, SCENE_H - 19, 10, 10, 3)
    ctx.fill()
  })
}

// The mac's second window: an app waiting for the authorization code.
// It closes the loop — end state (code accepted) is also frame one.
function drawMacCodeWindow(ctx, t, now) {
  const success = t < 0.2 || t >= T.codeFill + 0.017
  const filled = success || t >= T.codeFill
  rr(ctx, 30, 44, 216, 66, 5)
  ctx.fillStyle = 'rgba(10,15,22,0.92)'
  ctx.fill()
  ctx.strokeStyle = 'rgba(255,255,255,0.09)'
  ctx.lineWidth = 1
  ctx.stroke()
  ctx.fillStyle = 'rgba(255,255,255,0.05)'
  ctx.fillRect(30, 44, 216, 12)
  for (const [dx, dc] of [[38, '#ff5f57'], [47, '#febc2e'], [56, '#28c840']]) {
    ctx.fillStyle = dc
    ctx.beginPath()
    ctx.arc(dx, 50, 3, 0, Math.PI * 2)
    ctx.fill()
  }
  ctx.font = `8px ${MONO}`
  ctx.fillStyle = 'rgba(255,255,255,0.45)'
  ctx.fillText('authorize', 66, 52.5)
  ctx.font = `9.5px ${MONO}`
  ctx.fillStyle = C.soft
  ctx.fillText('Authorization code:', 40, 70)
  // input field
  ctx.strokeStyle = 'rgba(255,255,255,0.2)'
  rr(ctx, 40, 76, 130, 16, 3)
  ctx.stroke()
  if (filled) {
    // paste highlight fades right after the fill
    const hl = 1 - seg(t, T.codeFill, T.codeFill + 0.04)
    ctx.font = `600 9.5px ${MONO}`
    if (hl > 0 && t >= 0.2) {
      ctx.fillStyle = `rgba(78,229,133,${0.3 * hl})`
      ctx.fillRect(44, 78.5, ctx.measureText(TXT_PV).width + 4, 11)
    }
    ctx.fillStyle = '#7ee7ab'
    ctx.fillText(TXT_PV, 46, 87.5)
  } else if (Math.floor(now / 450) % 2 === 0) {
    ctx.fillStyle = C.soft
    ctx.fillRect(46, 79, 4.5, 10)
  }
  if (success) {
    ctx.font = `600 9.5px ${MONO}`
    ctx.fillStyle = C.mint
    ctx.fillText('✓ accepted', 178, 87.5)
  }
  // end of loop: pointer clicks into the field right before the paste
  if (t > 0.5) {
    const p = 1 - (1 - seg(t, T.macPtr[0], T.macPtr[1])) ** 3
    drawPointer(ctx, 300 + (105 - 300) * p, 215 + (86 - 215) * p)
  }
}

function drawMacOverlays(ctx, t, now) {
  // pointer walks from the field to the capture corner (continuing where
  // last loop's paste click left it), then the crosshair takes over
  if (t > 0.004 && t < T.sel[0]) {
    const p = 1 - (1 - seg(t, 0.005, T.sel[0])) ** 3
    drawPointer(ctx, 105 + (CAP.x - 2 - 105) * p, 86 + (CAP.y - 2 - 86) * p)
  }
  const selP = seg(t, T.sel[0], T.sel[1])
  const selDone = t > T.sel[1] && t < T.flash[1] + 0.01
  if ((selP > 0 && t < T.flash[1]) || selDone) {
    const sw = CAP.w * (selDone ? 1 : selP)
    const sh = CAP.h * (selDone ? 1 : selP)
    // dim everything outside the selection
    ctx.save()
    ctx.beginPath()
    ctx.rect(0, 22, SCENE_W, SCENE_H - 22)
    ctx.rect(CAP.x, CAP.y, sw, sh)
    ctx.clip('evenodd')
    ctx.fillStyle = `rgba(0,0,0,${0.35 * Math.min(1, selP * 4)})`
    ctx.fillRect(0, 22, SCENE_W, SCENE_H - 22)
    ctx.restore()
    // marching ants
    ctx.strokeStyle = '#ffffff'
    ctx.lineWidth = 1.4
    ctx.setLineDash([4, 4])
    ctx.lineDashOffset = -(now / 45)
    ctx.strokeRect(CAP.x, CAP.y, sw, sh)
    ctx.setLineDash([])
    // crosshair rides the drag corner
    if (t < T.sel[1] + 0.015) {
      const cx = CAP.x + sw
      const cy = CAP.y + sh
      ctx.strokeStyle = '#ffffff'
      ctx.lineWidth = 1.3
      ctx.beginPath()
      ctx.moveTo(cx - 6, cy)
      ctx.lineTo(cx + 6, cy)
      ctx.moveTo(cx, cy - 6)
      ctx.lineTo(cx, cy + 6)
      ctx.stroke()
    }
  }
  // camera flash: instant full brightness, quick decay
  const flp = seg(t, T.flash[0], T.flash[1])
  const fl = flp <= 0 || flp >= 1 ? 0 : (1 - flp) ** 1.5
  if (fl > 0.01) {
    ctx.fillStyle = `rgba(234,246,255,${fl * 0.6})`
    ctx.fillRect(0, 0, SCENE_W, SCENE_H)
  }
}

// A Linux desktop with a Claude Code terminal. `o` selects the beat:
//   { host, bar, wp: [c0, c1, c2],
//     typeWin | null (null → prompt pre-filled), imgAt, cursorUntil,
//     replyAt | null, codeAt, hlWin }
function drawTermScene(ctx, t, now, o) {
  // menubar
  ctx.fillStyle = o.bar
  ctx.fillRect(0, 0, SCENE_W, 22)
  ctx.fillStyle = 'rgba(255,255,255,0.28)'
  rr(ctx, 12, 8.5, 22, 4.5, 2)
  ctx.fill()
  ctx.fillStyle = 'rgba(255,255,255,0.6)'
  ctx.font = `9.5px ${MONO}`
  ctx.textAlign = 'center'
  ctx.fillText('18:31', SCENE_W / 2, 14.5)
  ctx.textAlign = 'left'
  ctx.fillStyle = 'rgba(255,255,255,0.3)'
  for (let i = 0; i < 3; i++) {
    ctx.beginPath()
    ctx.arc(SCENE_W - 40 + i * 11, 11, 2.5, 0, Math.PI * 2)
    ctx.fill()
  }
  // wallpaper
  const wp = ctx.createLinearGradient(0, 22, SCENE_W, SCENE_H)
  wp.addColorStop(0, o.wp[0])
  wp.addColorStop(0.55, o.wp[1])
  wp.addColorStop(1, o.wp[2])
  ctx.fillStyle = wp
  ctx.fillRect(0, 22, SCENE_W, SCENE_H - 22)
  // claude code terminal
  const wx = 26
  const wy = 46
  const ww = 340
  rr(ctx, wx, wy, ww, 118, 6)
  ctx.fillStyle = 'rgba(12,12,18,0.96)'
  ctx.fill()
  ctx.strokeStyle = 'rgba(255,255,255,0.1)'
  ctx.lineWidth = 1
  ctx.stroke()
  ctx.fillStyle = 'rgba(255,255,255,0.05)'
  ctx.fillRect(wx, wy, ww, 14)
  for (const [dx, dc] of [[wx + 9, '#ff5f57'], [wx + 18, '#febc2e'], [wx + 27, '#28c840']]) {
    ctx.fillStyle = dc
    ctx.beginPath()
    ctx.arc(dx, wy + 7, 3, 0, Math.PI * 2)
    ctx.fill()
  }
  ctx.fillStyle = 'rgba(255,255,255,0.45)'
  ctx.font = `8px ${MONO}`
  ctx.fillText(o.host, wx + 38, wy + 10)
  // prompt box
  const bx = wx + 10
  const by = wy + 24
  ctx.strokeStyle = 'rgba(255,255,255,0.2)'
  rr(ctx, bx, by, ww - 20, 22, 3)
  ctx.stroke()
  ctx.font = `600 10px ${MONO}`
  ctx.fillStyle = '#d97757'
  ctx.fillText('>', bx + 8, by + 15)
  const typed = o.prompt
  const n = o.typeWin ? Math.floor(seg(t, o.typeWin[0], o.typeWin[1]) * typed.length) : typed.length
  ctx.font = `10px ${MONO}`
  ctx.fillStyle = C.soft
  ctx.fillText(typed.slice(0, n), bx + 18, by + 15)
  let cx = bx + 18 + ctx.measureText(typed.slice(0, n)).width
  if (o.imgAt != null && t >= o.imgAt) {
    const chip = '[Image #1]'
    ctx.font = `8.5px ${MONO}`
    const cw = ctx.measureText(chip).width + 8
    ctx.fillStyle = 'rgba(110,168,254,0.16)'
    rr(ctx, cx + 4, by + 5.5, cw, 12, 3)
    ctx.fill()
    ctx.fillStyle = '#8fb8ff'
    ctx.fillText(chip, cx + 8, by + 14.5)
    cx += cw + 8
  }
  // cursor
  if (o.cursorUntil && t < o.cursorUntil && Math.floor(now / 450) % 2 === 0) {
    ctx.fillStyle = C.soft
    ctx.fillRect(cx + 4, by + 5.5, 5, 11)
  }
  // reply
  if (o.replyAt && t >= o.replyAt) {
    ctx.font = `8px ${MONO}`
    ctx.fillStyle = C.mint
    ctx.fillText('⏺', bx + 2, by + 40)
    ctx.font = `10px ${MONO}`
    ctx.fillStyle = C.soft
    ctx.fillText("Here's your authorization code:", bx + 14, by + 40)
  }
  if (o.codeAt && t >= o.codeAt) {
    ctx.font = `600 10.5px ${MONO}`
    const codeW = ctx.measureText(TXT_PV).width
    const hl = seg(t, o.hlWin[0], o.hlWin[1])
    if (hl > 0 && t < o.hlWin[1] + 0.015) {
      ctx.fillStyle = 'rgba(78,229,133,0.3)'
      ctx.fillRect(bx + 12, by + 50, (codeW + 6) * hl, 14)
    }
    ctx.fillStyle = '#7ee7ab'
    ctx.fillText(TXT_PV, bx + 15, by + 61)
  }
  // debian: pointer clicks into the prompt to focus it before typing
  if (o.ptrWin && t >= o.ptrWin[0]) {
    const p = 1 - (1 - seg(t, o.ptrWin[0], o.ptrWin[1])) ** 3
    drawPointer(ctx, 300 + (bx + 60 - 300) * p, 225 + (by + 18 - 225) * p)
  }
  // fedora: pointer performs the copy — approaches the code, then drags
  // across it in lockstep with the selection highlight
  if (o.copyApproach && t >= o.copyApproach[0]) {
    ctx.font = `600 10.5px ${MONO}`
    const codeW = ctx.measureText(TXT_PV).width
    const startX = bx + 13
    const y = by + 63
    const a = 1 - (1 - seg(t, o.copyApproach[0], o.copyApproach[1])) ** 3
    let x = 300 + (startX - 300) * a
    const py = 225 + (y - 225) * a
    if (t >= o.hlWin[0]) x = startX + (codeW + 6) * seg(t, o.hlWin[0], o.hlWin[1])
    drawPointer(ctx, x, py)
  }
}

const DEB_SCENE = {
  host: 'claude — debian',
  bar: '#191225',
  wp: ['#2b1436', '#55204a', '#7c2d52'],
  prompt: 'A bit more like this',
  ptrWin: [0.295, 0.35],
  typeWin: [0.355, 0.415],
  imgAt: 0.425,
  cursorUntil: 0.44,
  replyAt: null,
  codeAt: null,
}

// A different machine, its own conversation — no shared context
const FED_SCENE = {
  host: 'claude — fedora',
  bar: '#101c2a',
  wp: ['#0f2a4a', '#14406b', '#0f5a62'],
  prompt: 'log me into the registry',
  copyApproach: [0.638, 0.672],
  typeWin: [0.535, 0.6],
  imgAt: null,
  cursorUntil: 0.625,
  replyAt: 0.625,
  codeAt: 0.633,
  hlWin: [0.676, 0.706],
}

// ── layout ────────────────────────────────────────────────────────
function curve(p0, p1, vertical) {
  const c1 = vertical
    ? { x: p0.x, y: p0.y + (p1.y - p0.y) * 0.45 }
    : { x: p0.x + (p1.x - p0.x) * 0.45, y: p0.y }
  const c2 = vertical
    ? { x: p1.x, y: p1.y - (p1.y - p0.y) * 0.45 }
    : { x: p1.x - (p1.x - p0.x) * 0.45, y: p1.y }
  return {
    at(t) {
      const u = 1 - t
      return {
        x: u * u * u * p0.x + 3 * u * u * t * c1.x + 3 * u * t * t * c2.x + t * t * t * p1.x,
        y: u * u * u * p0.y + 3 * u * u * t * c1.y + 3 * u * t * t * c2.y + t * t * t * p1.y,
      }
    },
    p0,
    c1,
    c2,
    p1,
  }
}

// reserved strip at the canvas bottom for the zero-callouts line
const STAT_H = 48

function monitorHeight(fs) {
  const lh = fs * 1.62
  return Math.round(14 * lh + 108)
}

function layout(w) {
  const NW = 118
  const NH = 96
  const wide = w >= 620
  const nodes = {}
  let mon, fs, H, sceneScale
  if (wide) {
    const monW = Math.max(300, Math.min(470, w - 2 * NW - 4 * 26))
    fs = Math.max(9, Math.min(12, monW / 40))
    const monH = monitorHeight(fs)
    const sceneH = Math.max(monH + 56, 476)
    H = sceneH + STAT_H
    mon = { x: (w - monW) / 2, y: (sceneH - monH) / 2 + 14, w: monW, h: monH }
    nodes.macbook = { x: 8, y: sceneH - NH - 6, w: NW, h: NH }
    const rx = w - NW - 8
    const gap = (sceneH - 3 * NH) / 4
    nodes.fedora = { x: rx, y: gap, w: NW, h: NH }
    nodes.debian = { x: rx, y: gap * 2 + NH, w: NW, h: NH }
    nodes.mini = { x: rx, y: gap * 3 + NH * 2, w: NW, h: NH }
    sceneScale = Math.min(1, (w * 0.46) / SCENE_W)
  } else {
    const monW = Math.min(440, w - 8)
    fs = Math.max(8.5, Math.min(11, monW / 40))
    const monH = monitorHeight(fs)
    const gap = 46
    H = NH + gap + monH + gap + NH + 16 + STAT_H
    mon = { x: (w - monW) / 2, y: NH + gap + 8, w: monW, h: monH }
    nodes.macbook = { x: (w - NW) / 2, y: 8, w: NW, h: NH }
    const bw = Math.min(NW, (w - 32) / 3)
    const by = mon.y + monH + gap
    const bgap = (w - 3 * bw) / 4
    nodes.fedora = { x: bgap, y: by, w: bw, h: NH }
    nodes.mini = { x: bgap * 2 + bw, y: by, w: bw, h: NH }
    nodes.debian = { x: bgap * 3 + bw * 2, y: by, w: bw, h: NH }
    sceneScale = Math.min(1, (w - 16) / SCENE_W)
  }
  const cx = (r) => r.x + r.w / 2
  const cy = (r) => r.y + r.h / 2
  const wires = {}
  let deskARect, deskBRect
  if (wide) {
    wires.macbook = curve({ x: nodes.macbook.x + NW, y: cy(nodes.macbook) }, { x: mon.x, y: mon.y + mon.h * 0.72 }, false)
    wires.fedora = curve({ x: mon.x + mon.w, y: mon.y + mon.h * 0.24 }, { x: nodes.fedora.x, y: cy(nodes.fedora) }, false)
    wires.debian = curve({ x: mon.x + mon.w, y: mon.y + mon.h * 0.5 }, { x: nodes.debian.x, y: cy(nodes.debian) }, false)
    wires.mini = curve({ x: mon.x + mon.w, y: mon.y + mon.h * 0.76 }, { x: nodes.mini.x, y: cy(nodes.mini) }, false)
    deskARect = { x: 8, y: 8 }
    deskBRect = { x: w - SCENE_W * sceneScale - 8, y: 8 }
  } else {
    wires.macbook = curve({ x: cx(nodes.macbook), y: nodes.macbook.y + nodes.macbook.h }, { x: cx(mon), y: mon.y }, true)
    for (const n of ['fedora', 'mini', 'debian']) {
      wires[n] = curve({ x: cx(nodes[n]), y: mon.y + mon.h }, { x: cx(nodes[n]), y: nodes[n].y }, true)
    }
    // narrow: pop-up desktops cover the monitor area cleanly
    const dx = (w - SCENE_W * sceneScale) / 2
    const dy = mon.y + (mon.h - SCENE_H * sceneScale) / 2
    deskARect = { x: dx, y: dy }
    deskBRect = { x: dx, y: dy }
  }
  return { w, H, fs, mon, nodes, wires, wide, sceneScale, deskARect, deskBRect }
}

// ── shared drawing helpers ────────────────────────────────────────
function font(ctx, fs, bold) {
  ctx.font = `${bold ? '600 ' : ''}${fs}px ${MONO}`
}

function spans(ctx, x, y, fs, list, maxX) {
  let px = x
  for (const s of list) {
    font(ctx, s.fs || fs, s.b)
    if (maxX && px + ctx.measureText(s.t).width > maxX) break
    ctx.fillStyle = s.c
    ctx.fillText(s.t, px, y)
    px += ctx.measureText(s.t).width
  }
  return px
}

function tuiBlock(ctx, r, title, fs, borderColor, titleColor, bgColor) {
  if (bgColor) {
    ctx.fillStyle = bgColor
    ctx.fillRect(r.x, r.y, r.w, r.h)
  }
  ctx.strokeStyle = borderColor
  ctx.lineWidth = 1
  ctx.strokeRect(r.x + 0.5, r.y + 0.5, r.w - 1, r.h - 1)
  if (title) {
    font(ctx, fs, true)
    const tw = ctx.measureText(title).width
    ctx.fillStyle = bgColor || C.bg
    ctx.fillRect(r.x + 9, r.y - fs * 0.7, tw + 10, fs * 1.4)
    ctx.fillStyle = titleColor
    ctx.fillText(title, r.x + 14, r.y + fs * 0.36)
  }
}

function drawIcon(ctx, kind, cx, cy, led) {
  ctx.save()
  ctx.translate(cx, cy)
  ctx.strokeStyle = C.lineBright
  ctx.lineWidth = 1.4
  ctx.lineJoin = 'round'
  if (kind === 'laptop') {
    ctx.strokeRect(-17, -15, 34, 22)
    ctx.beginPath()
    ctx.moveTo(-24, 11)
    ctx.lineTo(24, 11)
    ctx.stroke()
    ctx.strokeStyle = C.mint
    ctx.globalAlpha = 0.35 + led * 0.65
    ctx.beginPath()
    ctx.moveTo(-12, -9)
    ctx.lineTo(-1, -9)
    ctx.moveTo(-12, -5)
    ctx.lineTo(-5, -5)
    ctx.moveTo(-12, -1)
    ctx.lineTo(-3, -1)
    ctx.stroke()
  } else if (kind === 'tower') {
    ctx.strokeRect(-9, -17, 18, 34)
    ctx.beginPath()
    ctx.moveTo(-5, -11)
    ctx.lineTo(5, -11)
    ctx.moveTo(-5, -7)
    ctx.lineTo(5, -7)
    ctx.moveTo(-5, -1)
    ctx.lineTo(5, -1)
    ctx.stroke()
    ctx.fillStyle = C.mint
    ctx.globalAlpha = 0.35 + led * 0.65
    ctx.fillRect(-5, 7, 2.4, 8)
  } else {
    ctx.strokeRect(-19, -9, 38, 18)
    ctx.beginPath()
    ctx.arc(0, 0, 5, 0, Math.PI * 2)
    ctx.stroke()
    ctx.fillStyle = C.mint
    ctx.globalAlpha = 0.35 + led * 0.65
    ctx.beginPath()
    ctx.arc(13, 4, 1.8, 0, Math.PI * 2)
    ctx.fill()
  }
  ctx.restore()
}

function drawNode(ctx, r, name, backend, kind, recv, copy) {
  const border = recv > 0.02 ? C.mint : copy > 0.02 ? C.accent : C.line
  tuiBlock(ctx, r, name, 11, border, recv > 0.02 ? C.mint : copy > 0.02 ? C.accent : C.dim, C.bg)
  const glow = Math.max(recv, copy)
  if (glow > 0.02) {
    ctx.save()
    ctx.globalAlpha = glow * 0.55
    ctx.strokeStyle = recv >= copy ? C.mint : C.accent
    const g = 3 + glow * 4
    ctx.strokeRect(r.x - g + 0.5, r.y - g + 0.5, r.w + g * 2 - 1, r.h + g * 2 - 1)
    ctx.restore()
  }
  drawIcon(ctx, kind, r.x + r.w / 2, r.y + r.h / 2 + 2, Math.max(recv, copy))
  font(ctx, 9.5, false)
  ctx.fillStyle = C.faint
  ctx.textAlign = 'center'
  ctx.fillText(backend, r.x + r.w / 2, r.y + r.h - 9)
  ctx.textAlign = 'left'
}

function drawMonitor(ctx, L, t, now) {
  const { mon, fs } = L
  const lh = fs * 1.62
  const p = Math.round(fs * 1.2)
  tuiBlock(ctx, mon, ' ssh-clipboard monitor ', fs, C.lineBright, C.dim, C.bg)
  const x = mon.x + p
  const iw = mon.w - p * 2
  let y = mon.y + p + lh * 0.6

  font(ctx, fs, true)
  const head = [
    { t: 'ssh', c: C.accent, b: true },
    { t: ' ◇ ', c: C.soft },
    { t: 'clipboard', c: C.accent, b: true },
    { t: '   ', c: C.soft },
    { t: '● LIVE', c: C.green, b: true },
  ]
  let hw = 0
  for (const s of head) hw += ctx.measureText(s.t).width
  const livePulse = 0.62 + 0.38 * Math.sin((now / 2400) * Math.PI * 2)
  ctx.save()
  spans(ctx, mon.x + (mon.w - hw) / 2, y, fs, head.slice(0, 4))
  ctx.globalAlpha = livePulse
  spans(ctx, mon.x + (mon.w - hw) / 2 + hw - ctx.measureText('● LIVE').width, y, fs, [head[4]])
  ctx.restore()
  y += lh * 0.95
  font(ctx, fs * 0.88, false)
  ctx.fillStyle = C.muted
  ctx.textAlign = 'center'
  ctx.fillText('native clipboard · persistent SSH · zero cloud hops', mon.x + mon.w / 2, y)
  ctx.textAlign = 'left'
  y += lh * 0.9

  const peersR = { x, y, w: iw, h: lh * 2 + 14 }
  tuiBlock(ctx, peersR, 'Peers', fs * 0.82, C.tuiPanel, C.accent)
  let py = y + lh * 0.62 + 7
  spans(
    ctx,
    x + 10,
    py,
    fs * 0.88,
    [
      { t: '● ', c: C.green },
      { t: 'macbook ', c: C.soft, b: true },
      { t: 'this machine', c: C.muted },
      { t: ' │ ', c: C.tuiPanel },
      { t: '● ', c: C.green },
      { t: 'debian ', c: C.soft, b: true },
      { t: 'v0.2.1', c: C.muted },
      { t: ' │ ', c: C.tuiPanel },
      { t: '● ', c: C.green },
      { t: 'fedora ', c: C.soft, b: true },
      { t: 'v0.2.1', c: C.muted },
      { t: ' │ ', c: C.tuiPanel },
      { t: '● ', c: C.yellow },
      { t: 'mini ', c: C.soft, b: true },
      { t: 'outdated', c: C.yellow },
    ],
    x + iw - 8
  )
  py += lh
  spans(
    ctx,
    x + 10,
    py,
    fs * 0.88,
    [
      { t: 'backend ', c: C.muted },
      { t: 'NSPasteboard', c: C.soft },
      { t: '  version ', c: C.muted },
      { t: '0.2.1', c: C.soft },
      { t: '  sent ', c: C.muted },
      { t: t >= T.sentTotal ? '26.4 MiB' : '—', c: t >= T.sentTotal ? C.soft : C.muted },
      { t: '  received ', c: C.muted },
      { t: t >= T.recvTotal ? '11 B' : '—', c: t >= T.recvTotal ? C.soft : C.muted },
    ],
    x + iw - 8
  )
  y += peersR.h + lh * 0.85

  const actR = { x, y, w: iw, h: lh * 9.4 + 14 }
  tuiBlock(ctx, actR, 'Clipboard activity', fs * 0.82, C.tuiPanel, C.accent)
  const rfs = fs * 0.88
  font(ctx, rfs, false)
  const ch = ctx.measureText('0').width
  const showFm = iw > 40 * ch
  const cTime = x + 10
  const cFlow = cTime + ch * 7
  const cFm = x + iw - 10
  const cSize = showFm ? cFm - ch * 4 : cFm
  const cPv = cFlow + ch * 14.5
  const pvMax = cSize - ch * 9 - cPv
  let ry = y + lh * 0.62 + 7
  font(ctx, rfs * 0.92, true)
  ctx.fillStyle = C.muted
  ctx.fillText('TIME', cTime, ry)
  ctx.fillText('FLOW', cFlow, ry)
  ctx.fillText('CONTENT', cPv, ry)
  ctx.textAlign = 'right'
  ctx.fillText('SIZE', cSize, ry)
  if (showFm) ctx.fillText('FMT', cFm, ry)
  ctx.textAlign = 'left'
  ry += lh
  ctx.save()
  ctx.globalAlpha = 1 - seg(t, T.fade[0], T.fade[1])
  for (const row of ROWS) {
    const tp = seg(t, row.at, row.at + 0.02)
    if (tp <= 0) break
    ctx.save()
    ctx.beginPath()
    ctx.rect(x, ry - lh, iw * tp, lh * 1.3)
    ctx.clip()
    font(ctx, rfs, false)
    ctx.fillStyle = C.muted
    ctx.fillText(row.time, cTime, ry)
    font(ctx, rfs, true)
    ctx.fillStyle = row.fc
    ctx.fillText(row.flow, cFlow, ry)
    font(ctx, rfs, false)
    ctx.fillStyle = C.soft
    let pv = row.pv
    while (pv.length > 1 && ctx.measureText(pv).width > pvMax) pv = pv.slice(0, -2) + '…'
    ctx.fillText(pv, cPv, ry)
    ctx.textAlign = 'right'
    ctx.fillStyle = C.muted
    ctx.fillText(row.size, cSize, ry)
    if (showFm) ctx.fillText(row.fm, cFm, ry)
    ctx.textAlign = 'left'
    ctx.restore()
    ry += lh
  }
  ctx.restore()

  const fy = mon.y + mon.h - p - lh * 0.1
  const foot = [
    { t: 'p', c: C.cyan, b: true },
    { t: ' pause · ', c: C.muted },
    { t: 'c', c: C.cyan, b: true },
    { t: ' clear · ', c: C.muted },
    { t: 'q', c: C.cyan, b: true },
    { t: ' close', c: C.muted },
  ]
  let fw = 0
  for (const s of foot) {
    font(ctx, fs * 0.88, s.b)
    fw += ctx.measureText(s.t).width
  }
  spans(ctx, mon.x + (mon.w - fw) / 2, fy, fs * 0.88, foot)
}

// ── component ─────────────────────────────────────────────────────
onMounted(() => {
  const canvas = cv.value
  const ctx = canvas.getContext('2d')
  // dev hook: ?demot=0.6 freezes the loop at that point of the timeline
  const demot = parseFloat(new URLSearchParams(window.location.search).get('demot'))
  const frozen = Number.isFinite(demot)
  const reduced = !frozen && window.matchMedia('(prefers-reduced-motion: reduce)').matches
  let L = null
  let dots = null
  let macBase = null
  let raf = 0
  let visible = true
  const start = performance.now()

  function resize() {
    const w = wrap.value.clientWidth
    if (!w) return
    L = layout(w)
    const dpr = Math.min(window.devicePixelRatio || 1, 2)
    canvas.width = Math.round(w * dpr)
    canvas.height = Math.round(L.H * dpr)
    canvas.style.height = `${L.H}px`
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    // static mac desktop, pre-rendered at 2x for crisp thumbnails
    macBase = document.createElement('canvas')
    macBase.width = SCENE_W * 2
    macBase.height = SCENE_H * 2
    const mctx = macBase.getContext('2d')
    mctx.setTransform(2, 0, 0, 2, 0, 0)
    drawMacBase(mctx)
    // ambient dot grid
    dots = document.createElement('canvas')
    dots.width = canvas.width
    dots.height = canvas.height
    const dctx = dots.getContext('2d')
    dctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    const cx = w / 2
    const cy = L.H / 2
    const maxD = Math.hypot(cx, cy)
    dctx.fillStyle = C.line
    for (let gy = 8; gy < L.H; gy += 22) {
      for (let gx = 8; gx < w; gx += 22) {
        const d = Math.hypot(gx - cx, gy - cy) / maxD
        dctx.globalAlpha = Math.max(0, 0.9 - d * 1.4)
        dctx.fillRect(gx, gy, 1.6, 1.6)
      }
    }
    if (frozen) draw(demot, demot * LOOP)
    else if (reduced) draw(0.9, 0)
  }

  // scale/alpha of a popped desktop; null when hidden
  function popState(t, [a1, a2, b1, b2]) {
    if (t < a1 || t > b2) return null
    const grow = outQuint(seg(t, a1, a2))
    const shrink = inCubic(seg(t, b1, b2))
    return { s: 0.14 + 0.86 * (grow - grow * shrink), a: Math.min(1, grow * 6) * (1 - shrink) }
  }

  // desktop frame + wedge back to its node, then scene content
  function drawDesk(pop, rect, origin, sceneDraw) {
    const s = L.sceneScale
    ctx.save()
    ctx.globalAlpha = pop.a
    // collapse toward the owning node
    ctx.translate(origin.x, origin.y)
    ctx.scale(pop.s, pop.s)
    ctx.translate(-origin.x, -origin.y)
    // callout wedge
    ctx.fillStyle = 'rgba(78,229,133,0.035)'
    ctx.strokeStyle = 'rgba(78,229,133,0.3)'
    ctx.lineWidth = 1
    ctx.setLineDash([3, 4])
    ctx.beginPath()
    ctx.moveTo(rect.x + 4, rect.y + SCENE_H * s)
    ctx.lineTo(origin.x - 20, origin.y)
    ctx.lineTo(origin.x + 20, origin.y)
    ctx.lineTo(rect.x + SCENE_W * s - 4, rect.y + SCENE_H * s)
    ctx.closePath()
    ctx.fill()
    ctx.stroke()
    ctx.setLineDash([])
    // frame
    ctx.translate(rect.x, rect.y)
    ctx.scale(s, s)
    ctx.save()
    rr(ctx, -2, -2, SCENE_W + 4, SCENE_H + 4, 10)
    ctx.fillStyle = '#0a0e14'
    ctx.fill()
    ctx.strokeStyle = '#3d4c60'
    ctx.lineWidth = 2
    ctx.stroke()
    rr(ctx, 0, 0, SCENE_W, SCENE_H, 8)
    ctx.clip()
    sceneDraw()
    ctx.restore()
    ctx.restore()
  }

  // the captured screenshot as a flying thumbnail
  function drawShot(pos, scale, alpha) {
    ctx.save()
    ctx.globalAlpha = alpha
    ctx.translate(pos.x, pos.y)
    ctx.scale(scale, scale)
    ctx.fillStyle = '#0a0e14'
    ctx.fillRect(-CAP.w / 2 - 2, -CAP.h / 2 - 2, CAP.w + 4, CAP.h + 4)
    ctx.save()
    ctx.beginPath()
    ctx.rect(-CAP.w / 2, -CAP.h / 2, CAP.w, CAP.h)
    ctx.clip()
    ctx.drawImage(macBase, CAP.x * 2, CAP.y * 2, CAP.w * 2, CAP.h * 2, -CAP.w / 2, -CAP.h / 2, CAP.w, CAP.h)
    ctx.restore()
    ctx.strokeStyle = C.bright
    ctx.lineWidth = 2.5
    ctx.strokeRect(-CAP.w / 2, -CAP.h / 2, CAP.w, CAP.h)
    ctx.restore()
  }

  function drawTxtChip(pos, scale, alpha) {
    ctx.save()
    ctx.globalAlpha = alpha
    ctx.translate(pos.x, pos.y)
    ctx.scale(scale, scale)
    font(ctx, 10.5, true)
    const w = ctx.measureText(TXT_PV).width + 16
    ctx.fillStyle = C.chipBg
    ctx.fillRect(-w / 2, -11, w, 22)
    ctx.strokeStyle = '#7ee7ab'
    ctx.lineWidth = 1.4
    ctx.strokeRect(-w / 2 + 0.5, -10.5, w - 1, 21)
    ctx.fillStyle = C.bright
    ctx.textAlign = 'center'
    ctx.fillText(TXT_PV, 0, 3.6)
    ctx.restore()
    ctx.textAlign = 'left'
  }

  // constant-speed travel, absorbed on arrival — like the old linear
  // offset-distance keyframes
  function flight(wire, win, t, drawFn, s0, s1, fromEnd) {
    const p = seg(t, win[0], win[1])
    if (p <= 0 || p >= 1) return
    const pos = wire.at(fromEnd ? 1 - p : p)
    drawFn(pos, s0 + (s1 - s0) * p, 1)
  }

  function draw(t, now) {
    const { w, H, mon, nodes, wires, sceneScale: s } = L
    ctx.clearRect(0, 0, w, H)
    ctx.drawImage(dots, 0, 0, w, H)
    ctx.strokeStyle = C.lineBright
    ctx.lineWidth = 1.2
    ctx.setLineDash([3, 6])
    ctx.lineDashOffset = reduced ? 0 : -(now / 90)
    for (const k of Object.keys(wires)) {
      const wire = wires[k]
      ctx.beginPath()
      ctx.moveTo(wire.p0.x, wire.p0.y)
      ctx.bezierCurveTo(wire.c1.x, wire.c1.y, wire.c2.x, wire.c2.y, wire.p1.x, wire.p1.y)
      ctx.stroke()
    }
    ctx.setLineDash([])

    drawNode(ctx, nodes.macbook, 'macbook', 'pasteboard', 'laptop',
      trap(seg(t, T.recvB[0], T.recvB[1])), trap(seg(t, T.flash[0], T.flash[1] + 0.06)))
    drawNode(ctx, nodes.fedora, 'fedora', 'wayland', 'tower',
      trap(seg(t, T.recvA[0], T.recvA[1])), trap(seg(t, T.hlC[0], T.hlC[1] + 0.04)))
    drawNode(ctx, nodes.debian, 'debian', 'x11', 'tower',
      Math.max(trap(seg(t, T.recvA[0], T.recvA[1])), trap(seg(t, T.recvB[0], T.recvB[1]))), 0)
    drawNode(ctx, nodes.mini, 'mini', 'pasteboard', 'mini',
      Math.max(trap(seg(t, T.recvA[0], T.recvA[1])), trap(seg(t, T.recvB[0], T.recvB[1]))), 0)

    drawMonitor(ctx, L, t, now)

    // zero callouts, part of the illustration — sized to the monitor width
    const segs = [
      { t: '0', c: C.bright, b: true },
      { t: ' relays', c: C.dim },
      { t: '  ·  ', c: C.faint },
      { t: '0', c: C.bright, b: true },
      { t: ' accounts', c: C.dim },
      { t: '  ·  ', c: C.faint },
      { t: '0', c: C.bright, b: true },
      { t: ' new ports', c: C.dim },
    ]
    const probe = 20
    let probeW = 0
    for (const sg of segs) {
      font(ctx, probe, sg.b)
      probeW += ctx.measureText(sg.t).width
    }
    const statFs = Math.max(11, Math.min(21, (probe * (mon.w - 28)) / probeW))
    let statW = 0
    for (const sg of segs) {
      font(ctx, statFs, sg.b)
      statW += ctx.measureText(sg.t).width
    }
    spans(ctx, mon.x + mon.w / 2 - statW / 2, H - STAT_H / 2 + statFs * 0.35, statFs, segs)

    if (reduced) return

    // popped desktops
    const nodeC = (r) => ({ x: r.x + r.w / 2, y: r.y + r.h / 2 })
    // the mac desktop stays open across the loop boundary
    let popA = null
    if (t >= T.deskD[0]) {
      const grow = outQuint(seg(t, T.deskD[0], T.deskD[1]))
      popA = { s: 0.14 + 0.86 * grow, a: Math.min(1, grow * 6) }
    } else if (t <= T.deskAClose[1]) {
      const shrink = inCubic(seg(t, T.deskAClose[0], T.deskAClose[1]))
      popA = { s: 1 - 0.86 * shrink, a: 1 - shrink }
    }
    if (popA) {
      drawDesk(popA, L.deskARect, nodeC(nodes.macbook), () => {
        ctx.drawImage(macBase, 0, 0, SCENE_W * 2, SCENE_H * 2, 0, 0, SCENE_W, SCENE_H)
        drawMacCodeWindow(ctx, t, now)
        drawMacOverlays(ctx, t, now)
      })
    }
    const popB = popState(t, T.deskB)
    if (popB) {
      drawDesk(popB, L.deskBRect, nodeC(nodes.debian), () => drawTermScene(ctx, t, now, DEB_SCENE))
    }
    const popC = popState(t, T.deskC)
    if (popC) {
      drawDesk(popC, L.deskBRect, nodeC(nodes.fedora), () => drawTermScene(ctx, t, now, FED_SCENE))
    }

    // beat A: the desktop has dismissed; the shot leaves the node on its wire
    flight(wires.macbook, T.flyA, t, drawShot, 0.42, 0.36, false)
    // fan out to all three peers at once
    flight(wires.fedora, T.fanA, t, drawShot, 0.32, 0.32, false)
    flight(wires.debian, T.fanA, t, drawShot, 0.32, 0.32, false)
    flight(wires.mini, T.fanA, t, drawShot, 0.32, 0.32, false)

    // beat B: the code rides fedora's wire back, then out to the rest
    flight(wires.fedora, T.flyB, t, drawTxtChip, 1, 0.75, true)
    flight(wires.debian, T.fanB, t, drawTxtChip, 0.6, 0.6, false)
    flight(wires.mini, T.fanB, t, drawTxtChip, 0.6, 0.6, false)
    flight(wires.macbook, T.fanB, t, drawTxtChip, 0.6, 0.6, true)
  }

  function tick(now) {
    raf = 0
    if (!visible) return
    draw(((now - start) % LOOP) / LOOP, now)
    raf = requestAnimationFrame(tick)
  }

  const ro = new ResizeObserver(resize)
  ro.observe(wrap.value)
  const io = new IntersectionObserver(([entry]) => {
    visible = entry.isIntersecting
    if (visible && !reduced && !frozen && !raf) raf = requestAnimationFrame(tick)
  })
  io.observe(canvas)
  resize()
  if (!reduced && !frozen) raf = requestAnimationFrame(tick)

  onBeforeUnmount(() => {
    if (raf) cancelAnimationFrame(raf)
    ro.disconnect()
    io.disconnect()
  })
})
</script>

<template>
  <div ref="wrap" class="w-full" aria-hidden="true">
    <canvas ref="cv" class="block w-full"></canvas>
  </div>
</template>
