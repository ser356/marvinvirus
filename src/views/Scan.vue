<script setup lang="ts">
import { inject, ref } from 'vue'
import { api } from '../api'
import type { ScanReport } from '../types'

const wizard = inject<any>('wizard')!
const emit = defineEmits<{ (e: 'done'): void }>()

const scanning = ref(false)
const includePrefetch = ref(false)
const error = ref<string | null>(null)

function fmt(bytes: number): string {
  const u = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let v = bytes
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++ }
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${u[i]}`
}

async function run() {
  scanning.value = true
  error.value = null
  try {
    const report: ScanReport = await api.scan(includePrefetch.value)
    wizard.report = report
    wizard.plan.files = report.files.filter((f: any) => f.preselect).map((f: any) => f.id)
    wizard.plan.startup_toggle = []
    wizard.plan.uninstall_ids = []
    emit('done')
  } catch (e: any) {
    error.value = e?.toString?.() ?? 'error desconocido'
  } finally {
    scanning.value = false
  }
}
</script>

<template>
  <section class="col">
    <div class="panel col">
      <h2>Escanear el equipo</h2>
      <p class="dim">
        Buscamos ficheros temporales, cache de navegadores, programas instalados, duplicados y elementos de arranque.
        Nada se borra en este paso.
      </p>
      <label class="row">
        <input type="checkbox" v-model="includePrefetch" />
        Incluir Prefetch (avanzado: puede ralentizar el primer arranque de algunas apps)
      </label>
      <div class="row">
        <button class="primary" :disabled="scanning" @click="run">
          {{ scanning ? 'Escaneando...' : 'Iniciar escaneo' }}
        </button>
        <span v-if="wizard.report" class="dim">
          Ultimo: {{ fmt(wizard.report.reclaimable_bytes) }} recuperables
        </span>
      </div>
      <div v-if="error" class="pill danger">{{ error }}</div>
    </div>
  </section>
</template>
