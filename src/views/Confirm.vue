<script setup lang="ts">
import { computed, inject } from 'vue'
import type { FileItem } from '../types'

const wizard = inject<any>('wizard')!
defineEmits<{ (e: 'done'): void; (e: 'back'): void }>()

function fmt(bytes: number): string {
  const u = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0; let v = bytes
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++ }
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${u[i]}`
}

const selectedFiles = computed<FileItem[]>(() => {
  const set = new Set<string>(wizard.plan.files)
  const all: FileItem[] = [
    ...(wizard.report?.files ?? []),
    ...(wizard.report?.large_files ?? []),
  ]
  return all.filter(f => set.has(f.id))
})

const totalBytes = computed(() => selectedFiles.value.reduce((s, f) => s + f.size, 0))
const needsUac = computed(() => selectedFiles.value.some(f => f.requires_elevation) || wizard.plan.uninstall_ids.length > 0)
const startupChanges = computed(() => wizard.plan.startup_toggle.length)
const uninstallCount = computed(() => wizard.plan.uninstall_ids.length)
</script>

<template>
  <section class="col">
    <div class="panel col">
      <h2>Confirmar acciones</h2>

      <div class="row">
        <div class="grow">Se van a mover a la Papelera</div>
        <div class="mono">{{ selectedFiles.length }} elementos · {{ fmt(totalBytes) }}</div>
      </div>

      <div class="row">
        <div class="grow">Cambios en arranque</div>
        <div class="mono">{{ startupChanges }}</div>
      </div>

      <div class="row">
        <div class="grow">Programas a desinstalar</div>
        <div class="mono">{{ uninstallCount }}</div>
      </div>

      <div v-if="needsUac" class="panel warn">
        Windows te pedira permiso de administrador (UAC) al continuar.
        Es necesario para tocar rutas del sistema o desinstalar programas.
      </div>

      <div class="dim">
        Todo lo eliminado va a la Papelera de reciclaje. Puedes deshacerlo desde el paso <em>Historial</em>.
      </div>

      <div class="row">
        <button @click="$emit('back')">Volver</button>
        <div class="grow"></div>
        <button class="primary" :disabled="!selectedFiles.length && !startupChanges && !uninstallCount" @click="$emit('done')">
          Ejecutar limpieza
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.panel.warn {
  border-color: rgba(245, 165, 36, 0.4);
  color: var(--warn);
}
</style>
