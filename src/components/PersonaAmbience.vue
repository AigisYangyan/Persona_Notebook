<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";

const now = ref(new Date());
let timer: number | undefined;

function pad(n: number): string {
  return n.toString().padStart(2, "0");
}

const timeText = computed(() => {
  const d = now.value;
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
});

const dateText = computed(() => {
  const d = now.value;
  const weekdays = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
  return `${d.getFullYear()}.${pad(d.getMonth() + 1)}.${pad(d.getDate())} ${weekdays[d.getDay()]}`;
});

onMounted(() => {
  timer = window.setInterval(() => {
    now.value = new Date();
  }, 1000);
});

onUnmounted(() => {
  if (timer) {
    clearInterval(timer);
  }
});
</script>

<template>
  <div class="persona-ambience" aria-hidden="true">
    <!-- Slow drifting diamond grid -->
    <div class="diamond-grid"></div>

    <!-- Vignette / color wash -->
    <div class="vignette"></div>

    <!-- Floating diamond glints -->
    <div class="glint glint-1"></div>
    <div class="glint glint-2"></div>
    <div class="glint glint-3"></div>
    <div class="glint glint-4"></div>

    <!-- Persona 3 style top-right status strip -->
    <div class="dark-hour-strip">
      <div class="strip-segment">
        <span class="strip-label">DARK HOUR</span>
        <span class="strip-value strip-pulse">ACTIVE</span>
      </div>
      <div class="strip-segment">
        <span class="strip-label">TIME</span>
        <span class="strip-value">{{ timeText }}</span>
      </div>
      <div class="strip-segment">
        <span class="strip-label">DATE</span>
        <span class="strip-value">{{ dateText }}</span>
      </div>
      <div class="strip-moon">
        <div class="moon"></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.persona-ambience {
  position: fixed;
  inset: 0;
  z-index: 2;
  pointer-events: none;
  overflow: hidden;
}

/* Diamond halftone grid, slow drift */
.diamond-grid {
  position: absolute;
  inset: -20%;
  background-image:
    radial-gradient(rgba(0, 212, 255, 0.14) 1px, transparent 1px),
    radial-gradient(rgba(0, 212, 255, 0.08) 1px, transparent 1px);
  background-size: 48px 48px, 24px 24px;
  background-position: 0 0, 24px 24px;
  transform: rotate(0deg);
  animation: grid-drift 30s linear infinite;
  mask-image: radial-gradient(circle at 50% 50%, black 30%, transparent 85%);
  -webkit-mask-image: radial-gradient(circle at 50% 50%, black 30%, transparent 85%);
  opacity: 0.55;
}

@keyframes grid-drift {
  0% {
    transform: translate(0, 0);
  }
  100% {
    transform: translate(24px, 24px);
  }
}

/* Soft vignette to focus attention */
.vignette {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(circle at 50% 35%, transparent 0%, rgba(3, 5, 10, 0.35) 65%, rgba(3, 5, 10, 0.75) 100%),
    linear-gradient(180deg, rgba(0, 40, 90, 0.1) 0%, transparent 35%, transparent 70%, rgba(0, 20, 50, 0.25) 100%);
}

/* Floating diamond glints */
.glint {
  position: absolute;
  width: 8px;
  height: 8px;
  background: var(--cyber-cyan);
  transform: rotate(45deg);
  box-shadow: 0 0 14px var(--cyber-cyan), 0 0 4px #fff;
  opacity: 0.2;
}

.glint-1 {
  top: 18%;
  left: 8%;
  animation: glint-float 7s ease-in-out infinite;
}

.glint-2 {
  top: 66%;
  left: 14%;
  width: 6px;
  height: 6px;
  animation: glint-float 9s ease-in-out infinite 1s;
}

.glint-3 {
  top: 24%;
  right: 12%;
  width: 10px;
  height: 10px;
  animation: glint-float 8s ease-in-out infinite 2s;
}

.glint-4 {
  top: 72%;
  right: 20%;
  width: 5px;
  height: 5px;
  animation: glint-float 6s ease-in-out infinite 0.5s;
}

@keyframes glint-float {
  0%, 100% {
    opacity: 0.15;
    transform: rotate(45deg) translateY(0) scale(1);
  }
  50% {
    opacity: 0.45;
    transform: rotate(45deg) translateY(-16px) scale(1.3);
  }
}

/* Dark Hour status strip - Persona 3 top-right flavor */
.dark-hour-strip {
  position: absolute;
  top: 16px;
  right: 16px;
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 8px 18px;
  background: rgba(5, 10, 22, 0.72);
  border: 1px solid rgba(0, 212, 255, 0.22);
  border-radius: 2px;
  backdrop-filter: blur(4px);
  box-shadow: 0 0 18px rgba(0, 212, 255, 0.08);
}

.strip-segment {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
}

.strip-label {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 2px;
  color: var(--cyber-text-dim);
}

.strip-value {
  font-size: 13px;
  font-weight: 700;
  color: var(--cyber-text-primary);
  letter-spacing: 1px;
  font-family: var(--cyber-font-mono);
}

.strip-pulse {
  color: var(--cyber-cyan);
  animation: active-pulse 2s ease-in-out infinite;
}

@keyframes active-pulse {
  0%, 100% {
    text-shadow: 0 0 6px rgba(0, 212, 255, 0.4);
    opacity: 1;
  }
  50% {
    text-shadow: 0 0 16px rgba(0, 212, 255, 0.8);
    opacity: 0.75;
  }
}

.strip-moon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
}

.moon {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: transparent;
  box-shadow: inset -6px -2px 0 2px var(--cyber-cyan);
  transform: rotate(-30deg);
  filter: drop-shadow(0 0 8px rgba(0, 212, 255, 0.4));
  animation: moon-pulse 4s ease-in-out infinite;
}

@keyframes moon-pulse {
  0%, 100% {
    filter: drop-shadow(0 0 8px rgba(0, 212, 255, 0.4));
  }
  50% {
    filter: drop-shadow(0 0 16px rgba(0, 212, 255, 0.7));
  }
}

@media (max-width: 720px) {
  .dark-hour-strip {
    display: none;
  }
}
</style>
