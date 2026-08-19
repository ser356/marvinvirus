<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api } from '../api'
import type { HistoryEntry } from '../types'

defineEmits<{ (e: 'back'): void }>()

const entries = ref<HistoryEntry[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const restoring = ref<string | null>(null)

function fmt(bytes: number): string {
  const u = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0; let v = bytes
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++ }
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${u[i]}`
}

async function load() {
  loading.value = true
  try {
    entries.value = await api.history()
  } catch (e: any) {
    error.value = e?.toString?.() ?? 'error'
  } finally {
    loading.value = false
  }
}

async function restore(id: string) {
  restoring.value = id
  try {
    await api.restore(id)
    await load()
  } catch (e: any) {
    error.value = e?.toString?.() ?? 'error'
  } finally {
    restoring.value = null
  }
}

onMounted(load)
</script>

<template>
  <section class="col">
    <div class="panel col">
      <h2>Historial de limpiezas</h2>
      <p class="dim">Cada limpieza se registra aqui. Puedes restaurar los ficheros desde la Papelera con un clic.</p>

      <div v-if="loading">Cargando...</div>
      <div v-else-if="error" class="pill danger">{{ error }}</div>
      <div v-else-if="!entries.length" class="dim">No hay limpiezas registradas.</div>

      <table v-else>
        <thead><tr><th>Fecha</th><th>Espacio</th><th>Elementos</th><th>Estado</th><th></th></tr></thead>
        <tbody>
          <tr v-for="e in entries" :key="e.id">
            <td>{{ new Date(e.at).toLocaleString() }}</td>
            <td class="mono">{{ fmt(e.freed_bytes) }}</td>
            <td class="mono">{{ e.items.length }}</td>
            <td>
              <span class="pill" :class="e.restored ? 'warn' : 'ok'">
                {{ e.restored ? 'restaurado' : 'aplicado' }}
              </span>
            </td>
            <td>
              <button :disabled="e.restored || restoring === e.id" @click="restore(e.id)">
                {{ restoring === e.id ? 'Restaurando...' : 'Restaurar' }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>

      <div class="row">
        <button @click="$emit('back')">Volver</button>
      </div>
    </div>
  </section>
</template>
