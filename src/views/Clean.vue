<script setup lang="ts">
import { inject, onMounted, ref } from 'vue'
import { api } from '../api'
import type { CleanResult } from '../types'

const wizard = inject<any>('wizard')!
const emit = defineEmits<{ (e: 'done'): void }>()

const running = ref(true)
const result = ref<CleanResult | null>(null)
const error = ref<string | null>(null)

function fmt(bytes: number): string {
  const u = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0; let v = bytes
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++ }
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${u[i]}`
}

async function run() {
  running.value = true
  error.value = null
  try {
    const r = await api.clean(wizard.plan)
    result.value = r
    wizard.result = r
  } catch (e: any) {
    error.value = e?.toString?.() ?? 'error desconocido'
  } finally {
    running.value = false
  }
}

async function launchUninstallers() {
  for (const id of wizard.plan.uninstall_ids) {
    const u = wizard.report?.uninstalls.find((x: any) => x.id === id)
    if (u) await api.launchUninstaller(u.uninstall_string).catch(() => {})
  }
}

onMounted(async () => {
  await run()
  if (result.value && !error.value && wizard.plan.uninstall_ids.length) {
    await launchUninstallers()
  }
})
</script>

<template>
  <section class="col">
    <div class="panel col">
      <h2>Limpieza en curso</h2>

      <div v-if="running">Ejecutando plan...</div>

      <div v-else-if="error" class="pill danger">{{ error }}</div>

      <div v-else-if="result" class="col">
        <div class="row">
          <div class="grow"><strong>Espacio liberado</strong></div>
          <div class="mono">{{ fmt(result.freed_bytes) }}</div>
        </div>
        <div class="row">
          <div class="grow">Ficheros procesados</div>
          <div class="mono">{{ result.ok.length }}</div>
        </div>
        <div class="row" v-if="result.failed.length">
          <div class="grow">Errores</div>
          <div class="mono">{{ result.failed.length }}</div>
        </div>

        <details v-if="result.failed.length" class="panel">
          <summary>Ver errores</summary>
          <table>
            <thead><tr><th>Ruta</th><th>Motivo</th></tr></thead>
            <tbody>
              <tr v-for="f in result.failed" :key="f.path">
                <td class="mono">{{ f.path }}</td>
                <td class="dim">{{ f.reason }}</td>
              </tr>
            </tbody>
          </table>
        </details>

        <div class="dim">
          Todo lo eliminado esta en la Papelera. Puedes restaurar desde <em>Historial</em>.
        </div>

        <div class="row">
          <div class="grow"></div>
          <button class="primary" @click="$emit('done')">Terminar</button>
        </div>
      </div>
    </div>
  </section>
</template>
