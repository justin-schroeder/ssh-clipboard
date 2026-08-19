<script setup>
// All motion lives in @keyframes (src/style.css); everything else is utilities.
const D = '[animation-delay:1.2s]'

const node =
  'relative flex w-[8.5rem] flex-col items-center gap-1 rounded-[14px] border border-linebright px-[0.8rem] pb-2 pt-[0.55rem] text-faint [background:linear-gradient(180deg,#141c26,#0d131b)] shadow-[inset_0_1px_0_rgba(255,255,255,0.05),0_10px_24px_-14px_rgba(0,0,0,0.8)]'

const ping =
  'pointer-events-none absolute -inset-px rounded-[14px] border-[1.5px] border-mint opacity-0'

const mname = 'text-[0.85rem] font-semibold leading-[1.3] text-bright'
const mos = 'text-[0.65rem] leading-[1.3] tracking-[0.06em] text-faint'

const wire =
  'fill-none stroke-linebright [stroke-dasharray:3_6] [stroke-linecap:round] [stroke-width:1.25] [vector-effect:non-scaling-stroke] animate-[dashdrift_1.6s_linear_infinite]'

const bigdesk =
  'absolute z-[6] w-[460px] overflow-hidden rounded-xl border-2 border-[#3d4c60] bg-[#0a0e14] opacity-0 shadow-[0_26px_60px_-20px_rgba(0,0,0,0.85),0_12px_50px_-20px_rgba(78,229,133,0.22)]'

const menubarItem = 'h-[4.5px] rounded-sm bg-white/28'

const dwin =
  'absolute left-[30px] top-[22px] w-[216px] overflow-hidden rounded-md border border-white/9 bg-[rgba(10,15,22,0.92)]'
const dwinBar = 'flex gap-1 bg-white/5 px-2 py-1.5'
const dwinBody = 'flex flex-col gap-1.5 px-2.5 pb-3 pt-2'
const dl = 'h-1 rounded-sm bg-[#33465c]'
const dot1 = 'h-1.5 w-1.5 rounded-full bg-[#ff5f57]'
const dot2 = 'h-1.5 w-1.5 rounded-full bg-[#febc2e]'
const dot3 = 'h-1.5 w-1.5 rounded-full bg-[#28c840]'

const photo =
  'absolute left-[270px] top-[96px] h-[58px] w-[78px] overflow-hidden rounded-[5px] border-[1.5px] border-white/30 shadow-[0_8px_20px_-8px_rgba(0,0,0,0.6)] [background:linear-gradient(180deg,#2c4a7c_0%,#7a4a6e_62%,#d98a5f_100%)] before:absolute before:right-4 before:top-2.5 before:h-3 before:w-3 before:rounded-full before:bg-[#ffd9a0] before:shadow-[0_0_8px_rgba(255,217,160,0.8)] before:content-[""] after:absolute after:inset-x-0 after:bottom-0 after:h-[60%] after:bg-[#1a2437] after:[clip-path:polygon(0_100%,0_55%,26%_18%,48%_60%,70%_28%,100%_70%,100%_100%)] after:content-[""]'

const wpGlow =
  'pointer-events-none absolute -right-[30px] -top-[30px] h-[200px] w-[200px] rounded-full [background:radial-gradient(circle,rgba(110,231,183,0.18),transparent_65%)]'

const flyShot =
  'pointer-events-none absolute left-0 top-0 z-[7] h-[138px] w-[205px] overflow-hidden rounded-[5px] border-[2.5px] border-bright opacity-0 shadow-[0_14px_32px_-8px_rgba(0,0,0,0.75)] [offset-rotate:0deg] motion-reduce:hidden'

const shotInner =
  'absolute left-[-166px] top-[-77px] h-[270px] w-[456px] [background:linear-gradient(135deg,#1c2c52_0%,#173a54_55%,#0f4a49_100%)]'

const flyText =
  'pointer-events-none absolute left-0 top-0 z-[7] w-[112px] rounded-[5px] border-[1.5px] border-[#7ee7ab] bg-[#0f1620] px-2 py-[5px] text-center text-[9.5px] leading-[1.35] text-[#e6edf5] opacity-0 shadow-[0_12px_26px_-8px_rgba(0,0,0,0.75)] [offset-rotate:0deg] motion-reduce:hidden'

const ftSent = 'rounded-sm bg-mint/20 px-0.5'

const tower =
  'M39 8.5h16M39 12.5h16M39 16.5h16'

/* monitor */
const muted = 'text-tuimuted'
const soft = 'text-tuisoft'
const panel = 'relative rounded-lg border border-tuipanel px-3.5 pb-2 pt-2.5'
const panelTitle =
  'absolute -top-[0.72em] left-3 bg-tuibg px-2 text-[0.68rem] font-bold tracking-[0.02em] text-tuiaccent'
const peerB = 'font-bold text-tuisoft'
const sep = 'text-tuipanel'
const rowGrid =
  'grid grid-cols-[4em_7.6em_minmax(0,1fr)_4.6em_4.4em] gap-x-2.5 whitespace-nowrap'
const trow = rowGrid + ' h-0 overflow-hidden opacity-0 [clip-path:inset(0_100%_0_0)]'
const cellPv = 'overflow-hidden text-ellipsis text-tuisoft'
const num = 'text-right text-tuimuted'
</script>

<template>
  <div class="@container w-full" aria-hidden="true">
    <div
      class="relative mt-6 h-[calc(480px*var(--s))] [--s:0.95] @max-[1069px]:[--s:0.92] @max-[1000px]:[--s:0.88] @max-[940px]:[--s:0.82] @max-[880px]:[--s:0.77] @max-[820px]:[--s:0.72] @max-[760px]:[--s:0.66] @max-[700px]:[--s:0.62] @max-[660px]:h-[calc(900px*var(--s))] @max-[660px]:[--s:1] @max-[540px]:[--s:0.96] @max-[500px]:[--s:0.9] @max-[460px]:[--s:0.83] @max-[420px]:[--s:0.76] @max-[380px]:[--s:0.69] @max-[340px]:[--s:0.62] @max-[300px]:[--s:0.55]"
    >
      <div
        class="absolute left-1/2 top-0 w-[63.5rem] origin-top [transform:translateX(-50%)_scale(var(--s))] @max-[660px]:w-[520px] before:pointer-events-none before:absolute before:-inset-x-4 before:-inset-y-12 before:content-[''] before:[background-image:radial-gradient(circle,#1c2836_1px,transparent_1px)] before:[background-size:22px_22px] before:[mask-image:radial-gradient(60%_70%_at_50%_50%,black_30%,transparent_75%)] after:pointer-events-none after:absolute after:-inset-x-8 after:-inset-y-16 after:opacity-55 after:content-[''] after:[background:radial-gradient(40rem_18rem_at_50%_50%,rgba(78,229,133,0.08),transparent_70%)] after:animate-[breathe_16s_ease-in-out_infinite] after:[animation-delay:1.2s]"
      >
        <div
          class="relative grid h-[480px] grid-cols-[16rem_4.5rem_30rem_4.5rem_8.5rem] items-stretch justify-center @max-[660px]:h-[900px] @max-[660px]:grid-cols-[1fr] @max-[660px]:grid-rows-[130px_70px_480px_70px_150px]"
        >
          <!-- callout: the big desktop projects out of macbook -->
          <svg
            class="pointer-events-none absolute left-0 top-0 h-full w-[460px] opacity-0 animate-[callout-show_16s_infinite] @max-[660px]:hidden"
            :class="D"
            viewBox="0 0 460 480"
          >
            <polygon points="12,298 448,298 186,368 70,368" fill="rgba(78,229,133,0.035)" />
            <path d="M12,298 L70,368" stroke="rgba(78,229,133,0.3)" stroke-width="1" stroke-dasharray="3 4" />
            <path d="M448,298 L186,368" stroke="rgba(78,229,133,0.3)" stroke-width="1" stroke-dasharray="3 4" />
          </svg>

          <!-- ── zoomed macOS desktop ── -->
          <div
            :class="[bigdesk, D]"
            class="left-[2px] top-0 [transform:scale(0.14)] [transform-origin:128px_420px] animate-[desk-pop_16s_cubic-bezier(0.22,1,0.36,1)_infinite] @max-[660px]:left-[28px] @max-[660px]:[transform-origin:232px_65px]"
          >
            <div class="flex h-[22px] items-center gap-2.5 border-b border-white/5 bg-[rgba(10,14,20,0.85)] px-3">
              <span class="h-[9px] w-[9px] rounded-full [background:linear-gradient(180deg,#cbd5e1,#7b8a9a)]"></span>
              <span :class="menubarItem" style="width: 1.8rem"></span>
              <span :class="menubarItem" style="width: 1.1rem"></span>
              <span :class="menubarItem" style="width: 1.4rem"></span>
              <span class="ml-auto flex items-center gap-2">
                <span :class="menubarItem" style="width: 0.8rem"></span>
                <span class="text-[0.6rem] tracking-[0.05em] text-white/60">18:31</span>
              </span>
            </div>
            <div
              class="relative h-[270px] overflow-hidden [background:linear-gradient(135deg,#1c2c52_0%,#173a54_55%,#0f4a49_100%)] before:absolute before:inset-0 before:z-[2] before:bg-black before:opacity-0 before:content-[''] before:animate-[sel-dim_16s_infinite] before:[animation-delay:1.2s]"
            >
              <span :class="wpGlow"></span>
              <div :class="dwin" class="shadow-[0_10px_24px_-8px_rgba(0,0,0,0.6)]">
                <div :class="dwinBar"><i :class="dot1"></i><i :class="dot2"></i><i :class="dot3"></i></div>
                <div :class="dwinBody">
                  <span :class="dl" style="width: 55%; background: #4ee585"></span>
                  <span :class="dl" style="width: 82%"></span>
                  <span :class="dl" style="width: 64%"></span>
                  <span :class="dl" style="width: 42%"></span>
                  <span :class="dl" style="width: 72%"></span>
                </div>
              </div>
              <div :class="photo"></div>
              <div
                :class="D"
                class="absolute left-[165px] top-[76px] z-[3] h-0 w-0 bg-white/10 bg-no-repeat opacity-0 [background-image:repeating-linear-gradient(90deg,#fff_0_4px,transparent_4px_8px),repeating-linear-gradient(90deg,#fff_0_4px,transparent_4px_8px),repeating-linear-gradient(0deg,#fff_0_4px,transparent_4px_8px),repeating-linear-gradient(0deg,#fff_0_4px,transparent_4px_8px)] [background-position:0_0,0_100%,0_0,100%_0] [background-size:100%_1.5px,100%_1.5px,1.5px_100%,1.5px_100%] animate-[sel-grow_16s_linear_infinite,ants_0.6s_linear_infinite]"
              ></div>
              <div
                :class="D"
                class="absolute left-[165px] top-[76px] z-[4] -m-[5.5px] h-[11px] w-[11px] opacity-0 [background:linear-gradient(#fff,#fff)_center/100%_1.3px_no-repeat,linear-gradient(#fff,#fff)_center/1.3px_100%_no-repeat] [filter:drop-shadow(0_0_3px_rgba(0,0,0,0.8))] animate-[crosshair_16s_linear_infinite] motion-reduce:hidden"
              ></div>
              <div
                :class="D"
                class="pointer-events-none absolute inset-0 z-[6] bg-[#eaf6ff] opacity-0 animate-[cam-flash_16s_infinite]"
              ></div>
              <div class="absolute bottom-2 left-1/2 flex -translate-x-1/2 gap-1.5 rounded-lg bg-white/10 px-2 py-1">
                <i class="h-2.5 w-2.5 rounded-[3.5px] bg-[#5aa7f7]"></i>
                <i class="h-2.5 w-2.5 rounded-[3.5px] bg-[#4ee585]"></i>
                <i class="h-2.5 w-2.5 rounded-[3.5px] bg-[#f7b955]"></i>
                <i class="h-2.5 w-2.5 rounded-[3.5px] bg-[#d78bea]"></i>
                <i class="h-2.5 w-2.5 rounded-[3.5px] bg-[#8b97a5]"></i>
              </div>
            </div>
          </div>

          <!-- ── debian desktop: Claude Code session ── -->
          <div
            :class="[bigdesk, D]"
            class="right-[2px] top-0 [transform:scale(0.14)] [transform-origin:396px_240px] animate-[desk-pop-deb_16s_cubic-bezier(0.22,1,0.36,1)_infinite] @max-[660px]:left-[28px] @max-[660px]:right-auto @max-[660px]:top-[380px] @max-[660px]:[transform-origin:232px_440px]"
          >
            <div class="relative flex h-[22px] items-center border-b border-white/5 bg-[#191225] px-3">
              <span :class="menubarItem" style="width: 1.4rem"></span>
              <span class="absolute left-1/2 -translate-x-1/2 text-[0.6rem] tracking-[0.05em] text-white/60">18:31</span>
              <span class="ml-auto flex gap-1.5">
                <span class="h-[5px] w-[5px] rounded-full bg-white/30"></span>
                <span class="h-[5px] w-[5px] rounded-full bg-white/30"></span>
                <span class="h-[5px] w-[5px] rounded-full bg-white/30"></span>
              </span>
            </div>
            <div class="relative h-[270px] overflow-hidden [background:linear-gradient(135deg,#2b1436_0%,#55204a_55%,#7c2d52_100%)]">
              <div class="absolute left-[26px] top-[24px] w-[330px] overflow-hidden rounded-md border border-white/10 bg-[rgba(12,12,18,0.95)] shadow-[0_12px_28px_-8px_rgba(0,0,0,0.65)]">
                <div :class="dwinBar">
                  <i :class="dot1"></i><i :class="dot2"></i><i :class="dot3"></i>
                  <span class="ml-1.5 text-[8px] leading-[6px] text-white/45">claude — debian</span>
                </div>
                <div class="flex flex-col gap-2 px-3 pb-3.5 pt-2.5 text-[9.5px] leading-[1.4]">
                  <div class="flex items-center gap-1.5 overflow-hidden whitespace-nowrap rounded border border-white/20 px-2 py-[5px]">
                    <span class="font-bold text-[#d97757]">&gt;</span>
                    <span
                      :class="D"
                      class="inline-block w-0 overflow-hidden whitespace-nowrap align-bottom text-tuisoft animate-[cc-type_16s_infinite]"
                    >A bit more like this</span>
                    <span
                      :class="D"
                      class="-ml-1.5 max-w-0 overflow-hidden whitespace-nowrap rounded-[3px] bg-[rgba(110,168,254,0.16)] text-[8.5px] text-[#8fb8ff] opacity-0 [padding:1px_0] animate-[cc-img_16s_infinite]"
                    >[Image #1]</span>
                    <span
                      class="h-[11px] w-[5px] bg-tuisoft animate-[cc-blink_0.9s_steps(1,end)_infinite,cc-cursor-gate_16s_infinite] [animation-delay:0s,1.2s]"
                    ></span>
                  </div>
                  <div :class="D" class="flex items-center gap-1.5 opacity-0 animate-[cc-reveal-line_16s_infinite]">
                    <span class="text-[8px] text-mint">⏺</span>
                    <span class="text-tuisoft">Here's your authorization code:</span>
                  </div>
                  <div :class="D" class="relative self-start text-[#e6edf5] opacity-0 animate-[cc-reveal-code_16s_infinite]">
                    <span
                      :class="D"
                      class="absolute -bottom-px -left-0.5 -top-px w-0 rounded-sm bg-mint/30 opacity-0 animate-[cc-hl_16s_linear_infinite]"
                    ></span>
                    <span class="relative z-[1] font-bold tracking-[0.04em] text-[#7ee7ab]">01923091283</span>
                  </div>
                </div>
              </div>
              <div
                :class="D"
                class="absolute left-[38px] top-[108px] z-[4] h-3 w-[3px] opacity-0 [background:linear-gradient(#fff,#fff)_center/1.4px_100%_no-repeat] [filter:drop-shadow(0_0_3px_rgba(0,0,0,0.9))] animate-[deb-cursor_16s_linear_infinite] before:absolute before:left-0 before:top-0 before:h-[1.4px] before:w-[3px] before:bg-white before:content-[''] after:absolute after:bottom-0 after:left-0 after:h-[1.4px] after:w-[3px] after:bg-white after:content-[''] motion-reduce:hidden"
              ></div>
            </div>
          </div>

          <!-- THE screenshot in flight -->
          <div
            :class="[flyShot, D]"
            class="[offset-path:path('M271,169_C300,300_128,330_128,415_C180,428_270,320_330,240')] animate-[fly-main_16s_infinite] @max-[660px]:[offset-path:path('M297,169_C315,130_295,80_260,65_C260,112_260,158_260,205')]"
          >
            <div :class="shotInner">
              <div :class="dwin">
                <div :class="dwinBar"><i :class="dot1"></i><i :class="dot2"></i><i :class="dot3"></i></div>
                <div :class="dwinBody">
                  <span :class="dl" style="width: 55%; background: #4ee585"></span>
                  <span :class="dl" style="width: 82%"></span>
                  <span :class="dl" style="width: 64%"></span>
                  <span :class="dl" style="width: 42%"></span>
                  <span :class="dl" style="width: 72%"></span>
                </div>
              </div>
              <div :class="photo"></div>
              <span :class="wpGlow"></span>
            </div>
          </div>

          <!-- screenshot copies fan out -->
          <div
            :class="[flyShot, D]"
            class="[transform:scale(0.22)] [offset-path:path('M810,202_C850,202_900,80_942,80')] animate-[fly-out-a_16s_infinite] @max-[660px]:[offset-path:path('M260,680_C260,725_87,748_87,820')]"
          >
            <div :class="shotInner">
              <div :class="dwin">
                <div :class="dwinBar"><i :class="dot1"></i><i :class="dot2"></i><i :class="dot3"></i></div>
                <div :class="dwinBody">
                  <span :class="dl" style="width: 55%; background: #4ee585"></span>
                  <span :class="dl" style="width: 82%"></span>
                  <span :class="dl" style="width: 64%"></span>
                </div>
              </div>
              <div :class="photo"></div>
            </div>
          </div>
          <div
            :class="[flyShot, D]"
            class="[transform:scale(0.22)] [offset-path:path('M810,240_C845,228_900,252_942,240')] animate-[fly-out-b_16s_infinite] @max-[660px]:[offset-path:path('M260,680_C260,725_260,748_260,820')]"
          >
            <div :class="shotInner">
              <div :class="dwin">
                <div :class="dwinBar"><i :class="dot1"></i><i :class="dot2"></i><i :class="dot3"></i></div>
                <div :class="dwinBody">
                  <span :class="dl" style="width: 55%; background: #4ee585"></span>
                  <span :class="dl" style="width: 82%"></span>
                  <span :class="dl" style="width: 64%"></span>
                </div>
              </div>
              <div :class="photo"></div>
            </div>
          </div>
          <div
            :class="[flyShot, D]"
            class="[transform:scale(0.22)] [offset-path:path('M810,278_C850,278_900,400_942,400')] animate-[fly-out-a_16s_infinite] @max-[660px]:[offset-path:path('M260,680_C260,725_433,748_433,820')]"
          >
            <div :class="shotInner">
              <div :class="dwin">
                <div :class="dwinBar"><i :class="dot1"></i><i :class="dot2"></i><i :class="dot3"></i></div>
                <div :class="dwinBody">
                  <span :class="dl" style="width: 55%; background: #4ee585"></span>
                  <span :class="dl" style="width: 82%"></span>
                  <span :class="dl" style="width: 64%"></span>
                </div>
              </div>
              <div :class="photo"></div>
            </div>
          </div>

          <!-- THE copied code, riding the pipes -->
          <div
            :class="[flyText, D]"
            class="[offset-path:path('M623,137_C760,160_948,155_948,235_C914,252_860,228_810,240')] animate-[fly-text-main_16s_infinite] @max-[660px]:[offset-path:path('M101,517_C180,630_260,720_260,818_C260,770_260,724_260,684')]"
          >
            <span :class="ftSent">01923091283</span>
          </div>
          <div
            :class="[flyText, D]"
            class="[transform:scale(0.42)] [offset-path:path('M330,240_C290,240_160,300_128,412')] animate-[fly-text-out-b_16s_infinite] @max-[660px]:[offset-path:path('M260,200_C260,160_260,110_260,70')]"
          >
            <span :class="ftSent">01923091283</span>
          </div>
          <div
            :class="[flyText, D]"
            class="[transform:scale(0.42)] [offset-path:path('M810,202_C850,202_900,80_942,80')] animate-[fly-text-out-a_16s_infinite] @max-[660px]:[offset-path:path('M260,680_C260,728_87,748_87,820')]"
          >
            <span :class="ftSent">01923091283</span>
          </div>
          <div
            :class="[flyText, D]"
            class="[transform:scale(0.42)] [offset-path:path('M810,278_C850,278_900,400_942,400')] animate-[fly-text-out-a_16s_infinite] @max-[660px]:[offset-path:path('M260,680_C260,728_433,748_433,820')]"
          >
            <span :class="ftSent">01923091283</span>
          </div>

          <!-- ── macbook card ── -->
          <div class="grid items-end justify-items-center pb-1.5 @max-[660px]:items-center @max-[660px]:pb-0">
            <div :class="[node, D]" class="animate-[glow-macbook_16s_infinite]">
              <span :class="[ping, D]" class="animate-[ping-macbook_16s_ease-out_infinite]"></span>
              <svg class="w-[76px] flex-none" viewBox="0 0 100 60">
                <defs>
                  <linearGradient id="alu-nb" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0" stop-color="#3b485c" />
                    <stop offset="1" stop-color="#212b39" />
                  </linearGradient>
                  <linearGradient id="wall-nb" x1="0" y1="0" x2="1" y2="1">
                    <stop offset="0" stop-color="#1c2c52" />
                    <stop offset="0.55" stop-color="#173a54" />
                    <stop offset="1" stop-color="#0f4a49" />
                  </linearGradient>
                </defs>
                <rect x="24" y="4" width="52" height="36" rx="3" fill="#0a0e14" stroke="#3d4c60" stroke-width="1.2" />
                <rect x="27" y="7" width="46" height="30" rx="1.5" fill="url(#wall-nb)" />
                <rect x="30" y="10" width="18" height="10" rx="1.5" fill="rgba(10,15,22,0.92)" />
                <path d="M32 13h9M32 15.5h6.5M32 18h8" stroke="#4ee585" stroke-width="1" stroke-linecap="round" opacity="0.7" />
                <path d="M14 44 h72 l4.5 7 a2.5 2.5 0 0 1 -2.4 3.2 H12 a2.5 2.5 0 0 1 -2.4 -3.2 z" fill="url(#alu-nb)" />
                <rect x="42" y="44" width="16" height="2.4" rx="1.2" fill="#18212d" />
              </svg>
              <div class="flex flex-col items-center text-center">
                <span :class="mname">macbook</span>
                <span :class="mos">pasteboard</span>
              </div>
            </div>
          </div>

          <!-- ── wire: macbook ⇄ monitor ── -->
          <svg class="h-full w-full overflow-visible" viewBox="0 0 72 480" preserveAspectRatio="none">
            <path
              :class="wire"
              class="@max-[660px]:[d:path('M36,-90_C36,160_36,320_36,480')]"
              d="M-58,420 C10,420 30,240 70,240"
            />
          </svg>

          <!-- ── the monitor ── -->
          <div
            class="flex min-w-0 flex-col overflow-hidden rounded-xl border border-white/9 bg-tuibg shadow-[inset_0_1px_0_rgba(255,255,255,0.06),0_40px_90px_-30px_rgba(0,0,0,0.85),0_25px_70px_-35px_rgba(78,229,133,0.22)] @max-[660px]:w-[500px] @max-[660px]:justify-self-center"
          >
            <div class="relative flex h-[2.1rem] flex-none items-center border-b border-black/55 px-3.5 [background:linear-gradient(180deg,#1b212c,#161c25)]">
              <span class="flex gap-2">
                <i class="h-3 w-3 rounded-full bg-[#ff5f57] shadow-[inset_0_0_1px_rgba(0,0,0,0.35)]"></i>
                <i class="h-3 w-3 rounded-full bg-[#febc2e] shadow-[inset_0_0_1px_rgba(0,0,0,0.35)]"></i>
                <i class="h-3 w-3 rounded-full bg-[#28c840] shadow-[inset_0_0_1px_rgba(0,0,0,0.35)]"></i>
              </span>
              <span class="absolute left-1/2 -translate-x-1/2 text-[0.75rem] font-semibold tracking-[0.01em] text-[#8b97a5]">ssh-clipboard monitor</span>
            </div>
            <div class="flex min-h-0 flex-1 flex-col gap-3.5 px-3.5 pb-2.5 pt-3.5 text-[0.7rem] leading-[1.5]">
              <div class="text-center leading-[1.4]">
                <div>
                  <span class="font-bold text-tuiaccent">ssh</span><span :class="soft"> ◇ </span><span class="font-bold text-tuiaccent">clipboard</span>
                  <span class="inline-block w-[1.2em]"></span>
                  <span class="font-bold text-tuigreen animate-[live-pulse_2.4s_ease-in-out_infinite]">● LIVE</span>
                </div>
                <div class="text-[0.65rem] text-tuimuted">native clipboard&nbsp; •&nbsp; persistent SSH&nbsp; •&nbsp; zero cloud hops</div>
              </div>

              <div :class="panel">
                <span :class="panelTitle">Peers</span>
                <div class="flex flex-wrap gap-x-3 gap-y-0.5 whitespace-nowrap">
                  <span><i class="not-italic text-tuigreen">●</i> <b :class="peerB">macbook</b> <span :class="muted">this machine</span></span>
                  <span :class="sep">│</span>
                  <span><i class="not-italic text-tuigreen">●</i> <b :class="peerB">debian</b> <span :class="muted">connected · v0.2.1</span></span>
                  <span :class="sep">│</span>
                  <span><i class="not-italic text-tuigreen">●</i> <b :class="peerB">fedora</b> <span :class="muted">connected · v0.2.1</span></span>
                  <span :class="sep">│</span>
                  <span><i class="not-italic text-tuiyellow">●</i> <b :class="peerB">mini</b> <span class="text-tuiyellow">outdated · 0.2.0</span></span>
                </div>
                <div class="mt-1.5 flex flex-wrap gap-x-2 whitespace-nowrap">
                  <span :class="muted">backend</span> <span :class="soft">NSPasteboard</span>
                  <span :class="muted" class="ml-2">version</span> <span :class="soft">0.2.1</span>
                  <span :class="muted" class="ml-2">sent</span>
                  <span :class="[soft, D]" class="opacity-0 animate-[show-at-28_16s_infinite]">26.4 MiB</span>
                  <span :class="muted" class="ml-2">received</span>
                  <span :class="[soft, D]" class="opacity-0 animate-[show-at-69_16s_infinite]">11 B</span>
                </div>
              </div>

              <div :class="panel" class="flex min-h-0 flex-1 flex-col">
                <span :class="panelTitle">Clipboard activity</span>
                <div class="flex flex-col overflow-hidden tabular-nums">
                  <div :class="rowGrid" class="mb-1.5 text-[0.62rem] font-bold tracking-[0.06em] text-tuimuted">
                    <span>TIME</span><span>FLOW</span><span>CONTENT</span><span class="text-right">SIZE</span><span class="text-right">FORMATS</span>
                  </div>
                  <div :class="[trow, D]" class="animate-[type-t3_16s_infinite]">
                    <span :class="muted">47.218</span>
                    <span class="font-bold text-tuicyan">→ macbook</span>
                    <span :class="cellPv">01923091283</span>
                    <span :class="num">11 B</span>
                    <span :class="num">1</span>
                  </div>
                  <div :class="[trow, D]" class="animate-[type-t2_16s_infinite]">
                    <span :class="muted">47.217</span>
                    <span class="font-bold text-tuicyan">→ mini</span>
                    <span :class="cellPv">01923091283</span>
                    <span :class="num">11 B</span>
                    <span :class="num">1</span>
                  </div>
                  <div :class="[trow, D]" class="animate-[type-t1_16s_infinite]">
                    <span :class="muted">47.216</span>
                    <span class="font-bold text-tuicyan">→ fedora</span>
                    <span :class="cellPv">01923091283</span>
                    <span :class="num">11 B</span>
                    <span :class="num">1</span>
                  </div>
                  <div :class="[trow, D]" class="animate-[type-recv_16s_infinite]">
                    <span :class="muted">47.214</span>
                    <span class="font-bold text-tuigreen">← debian</span>
                    <span :class="cellPv">01923091283</span>
                    <span :class="num">11 B</span>
                    <span :class="num">1</span>
                  </div>
                  <div :class="[trow, D]" class="animate-[type-s3_16s_infinite]">
                    <span :class="muted">41.532</span>
                    <span class="font-bold text-tuicyan">→ mini</span>
                    <span :class="cellPv">&lt;Apple PNG past…</span>
                    <span :class="num">8.8 MiB</span>
                    <span :class="num">6</span>
                  </div>
                  <div :class="[trow, D]" class="animate-[type-s2_16s_infinite]">
                    <span :class="muted">41.531</span>
                    <span class="font-bold text-tuicyan">→ debian</span>
                    <span :class="cellPv">&lt;Apple PNG past…</span>
                    <span :class="num">8.8 MiB</span>
                    <span :class="num">6</span>
                  </div>
                  <div :class="[trow, D]" class="animate-[type-s1_16s_infinite]">
                    <span :class="muted">41.530</span>
                    <span class="font-bold text-tuicyan">→ fedora</span>
                    <span :class="cellPv">&lt;Apple PNG past…</span>
                    <span :class="num">8.8 MiB</span>
                    <span :class="num">6</span>
                  </div>
                  <div :class="[trow, D]" class="animate-[type-local_16s_infinite]">
                    <span :class="muted">41.204</span>
                    <span class="font-bold text-tuiaccent">◆ copied here</span>
                    <span :class="cellPv">&lt;Apple PNG past…</span>
                    <span :class="num">8.8 MiB</span>
                    <span :class="num">6</span>
                  </div>
                </div>
              </div>

              <div class="flex-none text-center text-[0.65rem]">
                <span class="font-bold text-tuicyan">p</span><span :class="muted"> pause&nbsp; •&nbsp; </span><span class="font-bold text-tuicyan">c</span><span :class="muted"> clear&nbsp; •&nbsp; </span><span class="font-bold text-tuicyan">q</span><span :class="muted"> close</span>
              </div>
            </div>
          </div>

          <!-- ── wires: monitor ⇄ 3 machines ── -->
          <svg class="h-full w-full overflow-visible" viewBox="0 0 72 480" preserveAspectRatio="none">
            <path
              :class="wire"
              class="@max-[660px]:[d:path('M36,0_C36,250_12,250_12,630')]"
              d="M2,202 C40,202 30,80 70,80"
            />
            <path
              :class="wire"
              class="@max-[660px]:[d:path('M36,0_C36,250_36,250_36,630')]"
              d="M2,240 C34,228 38,252 70,240"
            />
            <path
              :class="wire"
              class="@max-[660px]:[d:path('M36,0_C36,250_60,250_60,630')]"
              d="M2,278 C40,278 30,400 70,400"
            />
          </svg>

          <!-- ── receiving machines ── -->
          <div class="grid grid-rows-3 items-center justify-items-center @max-[660px]:grid-cols-3 @max-[660px]:grid-rows-none">
            <div :class="[node, D]" class="animate-[glow-recv-both_16s_infinite]">
              <span :class="[ping, D]" class="animate-[ping-recv_16s_ease-out_infinite]"></span>
              <svg class="w-[76px] flex-none" viewBox="0 0 100 60">
                <defs>
                  <linearGradient id="case-fed" x1="0" y1="0" x2="1" y2="0">
                    <stop offset="0" stop-color="#2d3949" />
                    <stop offset="1" stop-color="#19222e" />
                  </linearGradient>
                </defs>
                <ellipse cx="50" cy="56" rx="22" ry="2.4" fill="rgba(0,0,0,0.45)" />
                <rect x="34" y="3" width="32" height="52" rx="3.5" fill="url(#case-fed)" stroke="#46566c" stroke-width="1" />
                <path :d="tower" stroke="#141d28" stroke-width="1.7" stroke-linecap="round" />
                <rect x="39" y="23" width="22" height="3" rx="1.5" fill="#141d28" />
                <rect :class="D" class="animate-[led-recv_16s_infinite]" x="39" y="31" width="2.6" height="15" rx="1.3" fill="#4ee585" />
                <circle class="animate-[led-idle_3.1s_ease-in-out_infinite]" cx="58" cy="47" r="2.7" fill="none" stroke="#4ee585" stroke-width="1.1" />
              </svg>
              <div class="flex flex-col items-center text-center">
                <span :class="mname">fedora</span>
                <span :class="mos">wayland</span>
              </div>
            </div>
            <div :class="[node, D]" class="animate-[glow-send-b_16s_infinite]">
              <span :class="[ping, D]" class="animate-[ping-send-b_16s_ease-out_infinite]"></span>
              <svg class="w-[76px] flex-none" viewBox="0 0 100 60">
                <defs>
                  <linearGradient id="case-deb" x1="0" y1="0" x2="1" y2="0">
                    <stop offset="0" stop-color="#2d3949" />
                    <stop offset="1" stop-color="#19222e" />
                  </linearGradient>
                </defs>
                <ellipse cx="50" cy="56" rx="22" ry="2.4" fill="rgba(0,0,0,0.45)" />
                <rect x="34" y="3" width="32" height="52" rx="3.5" fill="url(#case-deb)" stroke="#46566c" stroke-width="1" />
                <path :d="tower" stroke="#141d28" stroke-width="1.7" stroke-linecap="round" />
                <rect x="39" y="23" width="22" height="3" rx="1.5" fill="#141d28" />
                <rect :class="D" class="animate-[led-send-b_16s_infinite]" x="39" y="31" width="2.6" height="15" rx="1.3" fill="#4ee585" />
                <circle class="animate-[led-idle_3.1s_ease-in-out_infinite]" cx="58" cy="47" r="2.7" fill="none" stroke="#4ee585" stroke-width="1.1" />
              </svg>
              <div class="flex flex-col items-center text-center">
                <span :class="mname">debian</span>
                <span :class="mos">x11</span>
              </div>
            </div>
            <div :class="[node, D]" class="animate-[glow-recv-both_16s_infinite]">
              <span :class="[ping, D]" class="animate-[ping-recv_16s_ease-out_infinite]"></span>
              <svg class="w-[76px] flex-none" viewBox="0 0 100 60">
                <defs>
                  <linearGradient id="alu-mini" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0" stop-color="#3b485c" />
                    <stop offset="1" stop-color="#212b39" />
                  </linearGradient>
                </defs>
                <ellipse cx="50" cy="45" rx="28" ry="2.6" fill="rgba(0,0,0,0.45)" />
                <rect x="22" y="16" width="56" height="26" rx="6" fill="url(#alu-mini)" stroke="#46566c" stroke-width="1" />
                <rect x="24" y="17.4" width="52" height="2" rx="1" fill="rgba(255,255,255,0.1)" />
                <circle cx="50" cy="29" r="6" fill="none" stroke="#1c2734" stroke-width="1.5" />
                <rect x="30" y="36" width="12" height="1.6" rx="0.8" fill="#1b2634" />
                <circle :class="D" class="animate-[led-recv_16s_infinite]" cx="70" cy="36" r="1.9" fill="#4ee585" />
              </svg>
              <div class="flex flex-col items-center text-center">
                <span :class="mname">mini</span>
                <span :class="mos">pasteboard</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
