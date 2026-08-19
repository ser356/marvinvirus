<script setup lang="ts">
import { provide, reactive, ref, onMounted } from 'vue'
import type { CleanPlan, CleanResult, ScanReport, Step } from './types'
import { api } from './api'
import ScanView from './views/Scan.vue'
import ReviewView from './views/Review.vue'
import ConfirmView from './views/Confirm.vue'
import CleanView from './views/Clean.vue'
import RestoreView from './views/Restore.vue'

type Wizard = {
  step: Step
  report: ScanReport | null
  plan: CleanPlan
  result: CleanResult | null
  platform: { os: string; supported: boolean }
}

const wizard = reactive<Wizard>({
  step: 'scan',
  report: null,
  plan: { files: [], startup_toggle: [], uninstall_ids: [] },
  result: null,
  platform: { os: 'unknown', supported: false },
})

const showRestore = ref(false)

provide('wizard', wizard)

function go(step: Step) {
  wizard.step = step
  showRestore.value = step === 'restore'
}

onMounted(async () => {
  wizard.platform = await api.platform()
})

const steps: { id: Step; label: string }[] = [
  { id: 'scan', label: '1. Escanear' },
  { id: 'review', label: '2. Revisar' },
  { id: 'confirm', label: '3. Confirmar' },
  { id: 'clean', label: '4. Limpiar' },
]
</script>

<template>
  <div class="app">
    <header class="topbar">
      <div class="brand">marvinvirus</div>
      <nav class="steps">
        <button
          v-for="s in steps"
          :key="s.id"
          :class="{ active: wizard.step === s.id, primary: wizard.step === s.id }"
          @click="go(s.id)"
        >{{ s.label }}</button>
      </nav>
      <button @click="go('restore')" :class="{ active: showRestore }">Historial</button>
    </header>

    <main class="content">
      <div v-if="!wizard.platform.supported" class="panel warn">
        <strong>Aviso:</strong>
        Estas en <span class="mono">{{ wizard.platform.os }}</span>. Los modulos de limpieza solo funcionan en Windows.
        En este SO la UI corre pero los escaneres devuelven listas vacias (solo desarrollo).
      </div>

      <ScanView v-if="wizard.step === 'scan'" @done="go('review')" />
      <ReviewView v-else-if="wizard.step === 'review'" @done="go('confirm')" @back="go('scan')" />
      <ConfirmView v-else-if="wizard.step === 'confirm'" @done="go('clean')" @back="go('review')" />
      <CleanView v-else-if="wizard.step === 'clean'" @done="go('scan')" />
      <RestoreView v-else-if="wizard.step === 'restore'" @back="go('scan')" />
    </main>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
.topbar {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--panel);
}
.brand {
  font-weight: 700;
  letter-spacing: 0.02em;
  margin-right: 12px;
}
.steps { display: flex; gap: 6px; flex: 1; }
.content {
  padding: 16px;
  overflow: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.panel.warn {
  border-color: rgba(245, 165, 36, 0.4);
  color: var(--warn);
}
button.active { outline: 2px solid var(--accent); outline-offset: -2px; }
</style>
