<script setup lang="ts">
interface Props {
  title?: string;
  subTitle?: string;
  glow?: 'cyan' | 'blue' | 'green' | 'red' | 'none';
  padding?: string;
  angled?: boolean;
}

withDefaults(defineProps<Props>(), {
  glow: 'cyan',
  padding: '20px',
  angled: false,
});

const glowColors: Record<string, string> = {
  cyan: 'rgba(0, 212, 255, 0.25)',
  blue: 'rgba(0, 100, 255, 0.25)',
  green: 'rgba(0, 255, 170, 0.2)',
  red: 'rgba(255, 51, 102, 0.2)',
  none: 'transparent',
};

const glowColorsStrong: Record<string, string> = {
  cyan: 'rgba(0, 212, 255, 0.5)',
  blue: 'rgba(0, 100, 255, 0.5)',
  green: 'rgba(0, 255, 170, 0.4)',
  red: 'rgba(255, 51, 102, 0.4)',
  none: 'transparent',
};
</script>

<template>
  <div
    class="cyber-panel-wrapper"
    :class="{ angled: angled }"
    :style="{
      '--panel-glow': glowColors[glow],
      '--panel-glow-strong': glowColorsStrong[glow],
      '--panel-padding': padding,
    }"
  >
    <div v-if="title" class="cyber-panel-header">
      <span class="header-title">{{ title }}</span>
      <span v-if="subTitle" class="header-sub">{{ subTitle }}</span>
    </div>
    <div class="cyber-panel-body">
      <slot />
    </div>
    <!-- Decorative corners -->
    <div class="corner corner-tl"></div>
    <div class="corner corner-tr"></div>
    <div class="corner corner-bl"></div>
    <div class="corner corner-br"></div>
  </div>
</template>

<style scoped>
.cyber-panel-wrapper {
  background: rgba(10, 18, 40, 0.85);
  border: 1px solid var(--panel-glow);
  border-radius: 2px;
  position: relative;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  backdrop-filter: blur(4px);
}

.cyber-panel-wrapper::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(
    110deg,
    transparent 40%,
    rgba(0, 212, 255, 0.04) 50%,
    transparent 60%
  );
  background-size: 200% 100%;
  opacity: 0;
  transition: opacity 0.3s;
  pointer-events: none;
}

.cyber-panel-wrapper:hover {
  border-color: var(--panel-glow-strong);
  box-shadow: 0 0 20px var(--panel-glow), inset 0 0 16px rgba(0, 212, 255, 0.04);
  transform: translateY(-2px);
}

.cyber-panel-wrapper:hover::after {
  opacity: 1;
  animation: panel-sheen 1.8s linear infinite;
}

@keyframes panel-sheen {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

.cyber-panel-wrapper.angled {
  clip-path: polygon(
    0 10px, 10px 0,
    calc(100% - 10px) 0, 100% 10px,
    100% calc(100% - 10px), calc(100% - 10px) 100%,
    10px 100%, 0 calc(100% - 10px)
  );
}

.cyber-panel-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 20px;
  background: linear-gradient(90deg, rgba(0, 180, 255, 0.1), transparent);
  border-bottom: 1px solid var(--panel-glow);
}

.header-title {
  font-size: 14px;
  font-weight: 800;
  letter-spacing: 2px;
  text-transform: uppercase;
  color: var(--cyber-cyan);
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.25);
}

.header-sub {
  font-size: 13px;
  font-weight: 500;
  color: var(--cyber-text-muted);
  letter-spacing: 1px;
}

.cyber-panel-body {
  padding: var(--panel-padding);
}

/* Decorative corners */
.corner {
  position: absolute;
  width: 8px;
  height: 8px;
  border-color: var(--panel-glow-strong);
  border-style: solid;
  pointer-events: none;
  transition: all 0.3s;
}

.corner-tl {
  top: 0;
  left: 0;
  border-width: 2px 0 0 2px;
}

.corner-tr {
  top: 0;
  right: 0;
  border-width: 2px 2px 0 0;
}

.corner-bl {
  bottom: 0;
  left: 0;
  border-width: 0 0 2px 2px;
}

.corner-br {
  bottom: 0;
  right: 0;
  border-width: 0 2px 2px 0;
}

.cyber-panel-wrapper:hover .corner {
  width: 14px;
  height: 14px;
  border-color: var(--cyber-cyan);
  box-shadow: 0 0 8px var(--panel-glow-strong);
}
</style>
